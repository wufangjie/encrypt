//! 只实现 128 bit 密钥的情况

use crate::aes_const::{MIX_MAT, MIX_MAT_INV, RND_CON, RND_NUM, SUB_BOX, SUB_BOX_INV};
use std::fmt;
use std::ops::{Deref, DerefMut};

const N: usize = 4;

#[derive(Clone, Copy, PartialEq)]
pub struct BitSquare {
    data: [[u8; N]; N],
}

impl fmt::Display for BitSquare {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "BitSquare {{")?;
        for i in 0..N {
            //self.data[i].map(|x|
            let s = self.data[i]
                .iter()
                .map(|x| format!("{:02X}", x))
                .collect::<Vec<String>>()
                .join(", ");
            writeln!(f, "    [{}]{}", s, if i == N - 1 { "" } else { "," })?;
        }
        write!(f, "}}")
    }
}

impl fmt::Debug for BitSquare {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Default for BitSquare {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BitSquare {
    type Target = [[u8; N]; N];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for BitSquare {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl BitSquare {
    fn new() -> Self {
        Self { data: [[0; N]; N] }
    }

    fn from_row(row: &[u8]) -> Self {
        // TODO: 处理长度不够的情况
        let mut data = [[0; N]; N];
        for (i, v) in row.iter().take(N * N).enumerate() {
            data[i / N][i % N] = *v;
        }
        Self { data }
    }

    fn from_col(col: &[u8]) -> Self {
        let mut data = [[0; N]; N];
        for (i, v) in col.iter().take(N * N).enumerate() {
            data[i % N][i / N] = *v;
        }
        Self { data }
    }

    fn from_mat(mat: [[u8; N]; N]) -> Self {
        Self { data: mat }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(N * N);
        for j in 0..N {
            for i in 0..N {
                res.push(self[i][j]);
            }
        }
        res
    }
}

impl BitSquare {
    /// 密钥加法 (异或操作, 加密解密同)
    fn add(&mut self, other: Self) {
        for i in 0..N {
            for j in 0..N {
                self[i][j] ^= other[i][j];
            }
        }
    }

    /// 字节代换
    fn sub(&mut self) {
        for i in 0..N {
            for j in 0..N {
                self[i][j] = SUB_BOX[self[i][j] as usize];
            }
        }
    }

    /// 字节代换 (解密)
    fn sub_inv(&mut self) {
        for i in 0..N {
            for j in 0..N {
                self[i][j] = SUB_BOX_INV[self[i][j] as usize];
            }
        }
    }

    /// 行位移
    fn shift_row(&mut self) {
        for (i, row) in self.data.iter_mut().enumerate().skip(1) {
            row.rotate_left(i);
        }
    }

    /// 行位移 (解密)
    fn shift_row_inv(&mut self) {
        for (i, row) in self.data.iter_mut().enumerate().skip(1) {
            row.rotate_left(N - i);
        }
    }

    /// 列混淆
    fn mix_col(&mut self) {
        let mut new: [[u8; N]; N] = Default::default();
        for i in 0..N {
            for j in 0..N {
                let mut res = 0u8;
                for k in 0..N {
                    res ^= galois_mul(MIX_MAT[i][k], self[k][j]);
                }
                new[i][j] = res;
            }
        }
        self.data = new;
    }

    /// 列混淆 (解密)
    fn mix_col_inv(&mut self) {
        let mut new: [[u8; N]; N] = Default::default();
        for i in 0..N {
            for j in 0..N {
                let mut res = 0u8;
                for k in 0..N {
                    res ^= galois_mul(MIX_MAT_INV[i][k], self[k][j]);
                }
                new[i][j] = res;
            }
        }
        self.data = new;
    }
}

fn galois_mul(mut lhs: u8, mut rhs: u8) -> u8 {
    let mut res = 0u8;
    while lhs != 0 {
        if (lhs & 0x01) != 0 {
            res ^= rhs;
        }
        lhs >>= 1;

        if (rhs & 0x80) != 0 {
            // 0b10000000
            rhs <<= 1;
            rhs ^= 0x1B; // 0b00011011
        } else {
            rhs <<= 1;
        }
    }
    res
}

#[derive(Debug)]
pub struct AES {
    round: usize,
    keys: Vec<BitSquare>,
}

impl AES {
    pub fn new(key: BitSquare, round: usize) -> Self {
        //fn gen(&self) -> Vec<BitSquare> {
        // NOTE: 这个密钥的生成, 没有所谓的逆过程, 就是提前算好, 然后逆序解密

        let mut keys = Vec::<BitSquare>::with_capacity(11);
        keys.push(key);

        for r in 0..round {
            let mut new = BitSquare::new();

            for i in 0..N {
                new[i][0] = SUB_BOX[keys[r][(i + 1) % N][N - 1] as usize] ^ keys[r][i][0];
            }
            new[0][0] ^= RND_CON[r + 1];

            for j in 1..N {
                for i in 0..N {
                    new[i][j] = new[i][j - 1] ^ keys[r][i][j];
                }
            }
            keys.push(new);
        }

        Self { round, keys }
    }

    /// encode msg (bytes)
    pub fn encode(&self, msg: &str) -> Vec<u8> {
        // EBC 可以并行计算, CBC 每个 block 开始加密前要先和之前的加密结果 XOR
        let mut res = Vec::new();
        //let mut iter = msg.as_bytes().iter();
        for s in msg.as_bytes().chunks(16) {
            let mut block = BitSquare::from_col(s);
            self.encode_block(&mut block);
            res.extend(block.to_bytes());
        }

        // while !iter.is_empty() {
        //     self.encode_block(iter.take(N * N));
        // }
        res
    }

    /// decode msg (bytes)
    pub fn decode(&self, msg: &[u8]) -> Vec<u8> {
        let mut res = Vec::new();
        for s in msg.chunks(16) {
            let mut block = BitSquare::from_col(s);
            self.decode_block(&mut block);
            res.extend(block.to_bytes());
        }
        res
    }

    fn encode_block(&self, msg: &mut BitSquare) {
        msg.add(self.keys[0]);
        for i in 1..self.round {
            msg.sub();
            msg.shift_row();
            msg.mix_col();
            msg.add(self.keys[i])
        }
        msg.sub();
        msg.shift_row();
        msg.add(self.keys[self.round]);
    }

    fn decode_block(&self, msg: &mut BitSquare) {
        msg.add(self.keys[self.round]);
        msg.shift_row_inv();
        msg.sub_inv();
        for i in (1..self.round).into_iter().rev() {
            msg.add(self.keys[i]);
            msg.mix_col_inv();
            msg.shift_row_inv();
            msg.sub_inv();
        }
        msg.add(self.keys[0]);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::aes_const::{S_TABLE, S_TABLE_INV};

    impl BitSquare {
        fn sub2(&mut self) {
            for i in 0..N {
                for j in 0..N {
                    self[i][j] = S_TABLE[(self[i][j] >> N) as usize][(self[i][j] & 0xf) as usize];
                }
            }
        }

        fn sub_inv2(&mut self) {
            for i in 0..N {
                for j in 0..N {
                    self[i][j] =
                        S_TABLE_INV[(self[i][j] >> N) as usize][(self[i][j] & 0x0F) as usize];
                }
            }
        }
    }

    #[test]
    fn test2() {
        // let b = BitSquare::from_col(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        // dbg!(b.gen());
        let a = AES::new(
            BitSquare::from_col(&(1..17).into_iter().collect::<Vec<u8>>()),
            10,
        );
        dbg!(&a);

        let mut m =
            BitSquare::from_col(&[81, 72, 33, 84, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);

        a.encode_block(&mut m);
        dbg!(&m);
        dbg!(a.decode_block(&mut m));
        dbg!(&m);
    }

    #[test]
    fn test3() {
        let a = AES::new(
            BitSquare::from_col(&(1..17).into_iter().collect::<Vec<u8>>()),
            10,
        );

        let m = "The Advanced Encryption Standard (AES), also known by its original name Rijndael (Dutch pronunciation: [ˈrɛindaːl]),[3] is a specification for the encryption of electronic data established by the U.S. National Institute of Standards and Technology (NIST) in 2001.";

        let n = m.len();
        let c = a.encode(m);

        //dbg!(&c);
        let m2 = a.decode(&c);
        //dbg!(&m2);
        //a.encode_group();

        dbg!(String::from_utf8_lossy(&m2[..n]).to_string());
    }

    #[test]
    fn test() {
        let mut b = BitSquare::from_mat([
            [101, 187, 113, 232],
            [31, 249, 224, 13],
            [84, 77, 68, 73],
            [97, 236, 39, 55],
        ]);

        dbg!(&b.to_bytes());

        // b.shift_row();
        // dbg!(&b);
        // b.shift_row_inv();
        // dbg!(&b);

        let mut c = b; //.clone();
        b.sub();
        c.sub2();
        assert_eq!(b, c);

        b.sub_inv();
        c.sub_inv2();
        assert_eq!(b, c);

        let e = BitSquare::from_mat([[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]]);
        let mut m = BitSquare::from_mat(MIX_MAT);
        let mut n = BitSquare::from_mat(MIX_MAT_INV);

        n.mix_col();
        assert_eq!(e, n);
        m.mix_col_inv();
        assert_eq!(e, m);

        n.mix_col_inv();
        assert_eq!(BitSquare::from_mat(MIX_MAT_INV), n);

        // for i in 0..N {
        //     for j in 0..N {
        //         let sum: i32 = (0..N)
        //             .into_iter()
        //             .map(|k| (MIX_MAT[i][k] as i32) * (MIX_MAT_INV[k][j] as i32))
        //             .sum();
        //         dbg!(sum);
        //     }
        // }

        // let mut a = [1, 2, 3, 4, 5, 6, 7];
        // a.rotate_left(2);
        // dbg!(a);

        // dbg!(BitSquare::from_row(&[
        //     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
        // ]));
        // dbg!(BitSquare::from_col(&[
        //     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
        // ]));
    }
}

// BitSquare {
//     [101, 187, 113, 232],
//     [31, 249, 224, 13],
//     [84, 77, 68, 73],
//     [97, 236, 39, 55]
// }

// TODO:
// 为什么是 10 轮
// 为什么最后一轮, 不需要 mix_col
// 为什么第一轮需要 add (漂白)
// 用查表代替 galois_mul
