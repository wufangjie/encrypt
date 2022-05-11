//! 用一维数组加速: 不到 10 倍

use crate::aes_const::{
    EXP_TABLE, LOG_TABLE, MIX_MAT_LOG, MIX_MAT_LOG_INV, RND_CON, SUB_BOX, SUB_BOX_INV,
};
use std::fmt;
//use std::ops::{Deref, DerefMut};
//use std::slice::rotate;
//use rayon::prelude::*;

const N: usize = 4;
const N2: usize = N * N;

#[derive(Clone, Copy, PartialEq)]
pub struct ByteSquare {
    pub(crate) data: [u8; N2],
}

impl ByteSquare {
    pub fn new() -> Self {
        Self { data: [0; N2] }
    }

    pub fn from_rows(rows: &[[u8; N]]) -> Self {
        let mut data = [0; N2];
        for i in 0..N {
            for j in 0..N {
                data[j * N + i] = rows[i][j];
            }
        }
        Self { data }
    }

    pub fn from_col(col: &[u8]) -> Self {
        let data = col.try_into().unwrap();
        Self { data }
    }

    pub fn to_bytes(self) -> [u8; N2] {
        self.data
    }
}

impl From<[u8; N2]> for ByteSquare {
    fn from(data: [u8; N2]) -> Self {
        Self { data }
    }
}

// impl ByteSquare {
//     fn add(&self, other: &Self) -> Self {
//         let mut new = [0; N2];
//         for (i, new_i) in new.iter_mut().enumerate() {
//             *new_i = self.data[i] ^ other.data[i];
//         }
//         new.into()
//     }
// }

impl ByteSquare {
    // /// 密钥加法 (异或操作, 加密解密同)
    // fn add_(&mut self, other: &Self) {
    //     // for i in 0..N2 {
    //     //     self.data[i] ^= other.data[i];
    //     // }
    //     for (data_i, other_i) in self.data.iter_mut().zip(other.data.iter()) {
    //         *data_i ^= *other_i;
    //     }
    // }

    /// 密钥加法 (异或操作, 加密解密同)
    fn add_bytes(&mut self, other: &[u8]) {
        // for i in 0..N2 {
        //     self.data[i] ^= other[i];
        // }
        for (data_i, other_i) in self.data.iter_mut().zip(other.iter()) {
            *data_i ^= *other_i;
        }
    }

    /// 字节代换
    fn sub(&mut self) {
        for i in 0..N2 {
            self.data[i] = SUB_BOX[self.data[i] as usize];
        }
        // for p in self.data.iter_mut() {
        //     *p = SUB_BOX[*p as usize];
        // }

        // self.data
        //     .par_iter_mut()
        //     .for_each(|p| *p = SUB_BOX[*p as usize]);
        // use std::ptr;
        // unsafe {
        //     for i in 0..N2 {
        //         ptr::copy_nonoverlapping(
        // 	    SUB_BOX.get_unchecked(*self.data.get_unchecked(i) as usize),
        //             self.data.get_unchecked_mut(i),
        // 	    1
        //         );
        //     }
        // }
    }

    /// 字节代换 (解密)
    fn sub_inv(&mut self) {
        for i in 0..N2 {
            self.data[i] = SUB_BOX_INV[self.data[i] as usize];
        }
        // for p in self.data.iter_mut() {
        //     *p = SUB_BOX_INV[*p as usize];
        // }
    }

    /// 行位移
    fn shift_rows(&mut self) {
        let tmp = self.data[1];
        self.data[1] = self.data[5];
        self.data[5] = self.data[9];
        self.data[9] = self.data[13];
        self.data[13] = tmp;

        self.data.swap(2, 10);
        self.data.swap(6, 14);

        let tmp = self.data[15];
        self.data[15] = self.data[11];
        self.data[11] = self.data[7];
        self.data[7] = self.data[3];
        self.data[3] = tmp;
    }

    /// 行位移 (解密)
    fn shift_rows_inv(&mut self) {
        let tmp = self.data[13];
        self.data[13] = self.data[9];
        self.data[9] = self.data[5];
        self.data[5] = self.data[1];
        self.data[1] = tmp;

        self.data.swap(2, 10);
        self.data.swap(6, 14);

        let tmp = self.data[3];
        self.data[3] = self.data[7];
        self.data[7] = self.data[11];
        self.data[11] = self.data[15];
        self.data[15] = tmp;
    }

    /// 列混淆
    fn mix_cols(&mut self, v: &mut [usize]) {
        // let mut v = [0u8; N];
        // for p in self.data.chunks_mut(N) {
        //     v[0] = mat_mul(&MIX_MAT[0], &p);
        //     v[1] = mat_mul(&MIX_MAT[1], &p);
        //     v[2] = mat_mul(&MIX_MAT[2], &p);
        //     v[3] = mat_mul(&MIX_MAT[3], &p);
        //     p.copy_from_slice(&v);
        // }
        // let mut v = [0usize; N];
        for p in self.data.chunks_mut(N) {
            v[0] = LOG_TABLE[p[0] as usize];
            v[1] = LOG_TABLE[p[1] as usize];
            v[2] = LOG_TABLE[p[2] as usize];
            v[3] = LOG_TABLE[p[3] as usize];
            p[0] = mat_mul_2(&MIX_MAT_LOG[0], v);
            p[1] = mat_mul_2(&MIX_MAT_LOG[1], v);
            p[2] = mat_mul_2(&MIX_MAT_LOG[2], v);
            p[3] = mat_mul_2(&MIX_MAT_LOG[3], v);
        }
    }

    /// 列混淆 (解密)
    fn mix_cols_inv(&mut self, v: &mut [usize]) {
        // let mut v = [0u8; N];
        // for p in self.data.chunks_mut(N) {
        //     v[0] = mat_mul(&MIX_MAT_INV[0], &p);
        //     v[1] = mat_mul(&MIX_MAT_INV[1], &p);
        //     v[2] = mat_mul(&MIX_MAT_INV[2], &p);
        //     v[3] = mat_mul(&MIX_MAT_INV[3], &p);
        //     p.copy_from_slice(&v);
        // }
        //let mut v = [0usize; N];
        for p in self.data.chunks_mut(N) {
            v[0] = LOG_TABLE[p[0] as usize];
            v[1] = LOG_TABLE[p[1] as usize];
            v[2] = LOG_TABLE[p[2] as usize];
            v[3] = LOG_TABLE[p[3] as usize];
            p[0] = mat_mul_2(&MIX_MAT_LOG_INV[0], v);
            p[1] = mat_mul_2(&MIX_MAT_LOG_INV[1], v);
            p[2] = mat_mul_2(&MIX_MAT_LOG_INV[2], v);
            p[3] = mat_mul_2(&MIX_MAT_LOG_INV[3], v);
        }
    }
}

impl fmt::Display for ByteSquare {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "ByteSquare {{")?;
        for i in 0..N {
            write!(f, "[")?;
            for j in 0..N - 1 {
                write!(f, "{}, ", self.data[j * N + i])?;
            }
            writeln!(f, "{}],", self.data[(N - 1) * N + i])?;
        }
        write!(f, "}}")
    }
}

impl fmt::Debug for ByteSquare {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Default for ByteSquare {
    fn default() -> Self {
        Self::new()
    }
}

// /// polynomial version of GF(2^8) multiplication
// fn galois_mul(mut lhs: u8, mut rhs: u8) -> u8 {
//     let mut res = 0u8;
//     while lhs != 0 {
//         if (lhs & 0x01) != 0 {
//             res ^= rhs;
//         }
//         lhs >>= 1;

//         if (rhs & 0x80) != 0 {
//             // 0b10000000
//             rhs <<= 1;
//             rhs ^= 0x1B; // 0b00011011
//         } else {
//             rhs <<= 1;
//         }
//     }
//     res
// }

/// table lookup version of GF(2^8) multiplication
#[inline]
fn log_sum_exp(lhs: u8, rhs: u8) -> u8 {
    if lhs == 0 || rhs == 0 {
        0
    } else {
        // loop size: 0xff
        //EXP_TABLE[(LOG_TABLE[lhs as usize] + LOG_TABLE[rhs as usize]) % 0xff]
        EXP_TABLE[LOG_TABLE[lhs as usize] + LOG_TABLE[rhs as usize]]
    }
}

/// table lookup version of GF(2^8) multiplication
#[inline]
fn mat_mul(row: &[u8], col: &[u8]) -> u8 {
    log_sum_exp(row[0], col[0])
        ^ log_sum_exp(row[1], col[1])
        ^ log_sum_exp(row[2], col[2])
        ^ log_sum_exp(row[3], col[3])
}

#[inline]
fn mat_mul_2(row: &[usize], col: &[usize]) -> u8 {
    // row[0] will not equal to 0
    (if col[0] == 0 {
        0
    } else {
        EXP_TABLE[row[0] + col[0]]
    } ^ if col[1] == 0 {
        0
    } else {
        EXP_TABLE[row[1] + col[1]]
    } ^ if col[2] == 0 {
        0
    } else {
        EXP_TABLE[row[2] + col[2]]
    } ^ if col[3] == 0 {
        0
    } else {
        EXP_TABLE[row[3] + col[3]]
    })
}

#[derive(Debug)]
pub struct AES {
    round: usize,
    pub(crate) keys: Vec<[u8; N2]>, //ByteSquare>,
}

impl AES {
    /// row style key
    pub fn new(key: &[u8]) -> Self {
        //fn gen(&self) -> Vec<ByteSquare> {
        // NOTE: 这个密钥的生成, 没有所谓的逆过程, 就是提前算好, 然后逆序解密
        let key_len = key.len() / N;

        let round = match key_len {
            4 => 10,
            6 => 12,
            8 => 14,
            _ => panic!("AES only support 128/192/256 bits key!"),
        };

        let mut key_manager = vec![];
        for row in key.chunks(N) {
            let mut new = [0; N];
            for (i, new_i) in new.iter_mut().enumerate() {
                *new_i = row[i];
            }
            key_manager.push(new);
        }

        let mut i = key_len;
        let mut r = 0;
        let nrow = N * (round + 1);

        while i < nrow {
            let mut new = key_manager[i - 1];
            if i % N == 0 {
                for j in 0..N {
                    new[j] = SUB_BOX[new[j] as usize];
                }
            }

            if i % key_len == 0 {
                new.rotate_left(1);
                new[0] ^= RND_CON[r];
                r += 1;
            }

            for (j, new_j) in new.iter_mut().enumerate() {
                *new_j ^= key_manager[i - key_len][j];
            }
            i += 1;
            key_manager.push(new);
        }

        let mut keys = Vec::<[u8; N2]>::with_capacity(1 + round);
        for r in 0..=round {
            let mut key = [0; N2];
            for j in 0..N {
                for i in 0..N {
                    key[j * N + i] = key_manager[r * N + j][i];
                }
            }
            keys.push(key); // ByteSquare { data: key });
        }
        Self { round, keys }
    }

    pub fn encode_ecb(&self, msg: &[u8]) -> Vec<u8> {
        // ECB 可以并行计算, CBC 每个 block 开始加密前要先和之前的加密结果 XOR
        let mut res = Vec::with_capacity(msg.len());
        let mut cache = [0; N2];
        for m in msg.chunks(N2) {
            let mut block = ByteSquare::from_col(m);
            self.encode_block(&mut block, &mut cache);
            res.extend(block.to_bytes());
        }
        res
    }

    pub fn decode_ecb(&self, msg: &[u8]) -> Vec<u8> {
        let mut res = Vec::with_capacity(msg.len());
        let mut cache = [0; N2];
        for m in msg.chunks(N2) {
            let mut block = ByteSquare::from_col(m);
            self.decode_block(&mut block, &mut cache);
            res.extend(block.to_bytes());
        }
        res
    }

    pub fn encode_cbc(&self, msg: &[u8], mut iv: ByteSquare) -> Vec<u8> {
        // iv means init vector
        let mut res = Vec::with_capacity(msg.len());
        let mut cache = [0; N2];
        for m in msg.chunks(N2) {
            iv.add_bytes(m);
            self.encode_block(&mut iv, &mut cache);
            res.extend(iv.to_bytes());
        }
        res
    }

    pub fn decode_cbc(&self, msg: &[u8], iv: ByteSquare) -> Vec<u8> {
        let mut res = Vec::with_capacity(msg.len());
        let mut iv_ref = &iv.data[..];
        let mut cache = [0; N2];
        for m in msg.chunks(N2) {
            let mut block = ByteSquare::from_col(m);
            self.decode_block(&mut block, &mut cache);
            block.add_bytes(iv_ref);
            res.extend(block.to_bytes());
            iv_ref = m;
        }
        res
    }

    /// decode ige mode (for telegram)
    pub fn encode_ige(&self, msg: &[u8], mut y_prev: ByteSquare, x_prev: ByteSquare) -> Vec<u8> {
        let mut res = Vec::with_capacity(msg.len());
        let mut x_prev_ref = &x_prev.data[..];
        let mut cache = [0; N2];
        for m in msg.chunks(N2) {
            y_prev.add_bytes(m);
            self.encode_block(&mut y_prev, &mut cache);
            y_prev.add_bytes(x_prev_ref);
            x_prev_ref = m;
            res.extend(y_prev.to_bytes());
        }
        res
    }

    /// decode ige mode (for telegram)
    pub fn decode_ige(&self, msg: &[u8], y_prev: ByteSquare, mut x_prev: ByteSquare) -> Vec<u8> {
        // NOTE: 把 y_prev 和 x_prev 换一下, 就和 encode_ige 完全一样
        let mut res = Vec::with_capacity(msg.len());
        let mut y_prev_ref = &y_prev.data[..];
        let mut cache = [0; N2];
        for m in msg.chunks(N2) {
            x_prev.add_bytes(m);
            self.decode_block(&mut x_prev, &mut cache);
            x_prev.add_bytes(y_prev_ref);
            y_prev_ref = m;
            res.extend(x_prev.to_bytes());
        }
        res
    }

    #[inline(always)]
    fn encode_block(&self, msg: &mut ByteSquare, cache: &mut [usize]) {
        msg.add_bytes(&self.keys[0]);
        for i in 1..self.round {
            msg.sub();
            msg.shift_rows();
            msg.mix_cols(cache);
            msg.add_bytes(&self.keys[i])
        }
        msg.sub();
        msg.shift_rows();
        msg.add_bytes(&self.keys[self.round]);
    }

    #[inline(always)]
    fn decode_block(&self, msg: &mut ByteSquare, cache: &mut [usize]) {
        msg.add_bytes(&self.keys[self.round]);
        msg.shift_rows_inv();
        msg.sub_inv();
        for i in (1..self.round).into_iter().rev() {
            msg.add_bytes(&self.keys[i]);
            msg.mix_cols_inv(cache);
            msg.shift_rows_inv();
            msg.sub_inv();
        }
        msg.add_bytes(&self.keys[0]);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::conv::hex_to_bytes;

    // #[test]
    // fn print_log_m() {
    // 	use crate::aes_const::{MIX_MAT, MIX_MAT_INV};
    // 	for row in MIX_MAT {
    //         println!(
    //             "[0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X}],",
    //             LOG_TABLE[row[0] as usize],
    //             LOG_TABLE[row[1] as usize],
    //             LOG_TABLE[row[2] as usize],
    //             LOG_TABLE[row[3] as usize],
    //         );
    //     }

    //     println!();
    //     for row in MIX_MAT_INV {
    //         println!(
    //             "[0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X}],",
    //             LOG_TABLE[row[0] as usize],
    //             LOG_TABLE[row[1] as usize],
    //             LOG_TABLE[row[2] as usize],
    //             LOG_TABLE[row[3] as usize],
    //         );
    //     }
    // }

    // #[test]
    // fn test_mix_cols() {
    //     let e = ByteSquare::from_rows(&[[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]]);
    //     let mut m = ByteSquare::from_rows(&MIX_MAT);
    //     let mut n = ByteSquare::from_rows(&MIX_MAT_INV);
    //     n.mix_cols();
    //     assert_eq!(e, n);
    //     m.mix_cols_inv();
    //     assert_eq!(e, m);
    //     n.mix_cols_inv();
    //     assert_eq!(ByteSquare::from_rows(&MIX_MAT_INV), n);
    // }

    #[test]
    fn test_shift() {
        let mut e = ByteSquare::from_rows(&[
            [1, 2, 3, 4],
            [5, 6, 7, 8],
            [9, 10, 11, 12],
            [13, 14, 15, 16],
        ]);

        e.shift_rows();
        assert_eq!(
            e,
            ByteSquare::from_rows(&[
                [1, 2, 3, 4],
                [6, 7, 8, 5],
                [11, 12, 9, 10],
                [16, 13, 14, 15]
            ])
        );

        e.shift_rows_inv();
        assert_eq!(
            e,
            ByteSquare::from_rows(&[
                [1, 2, 3, 4],
                [5, 6, 7, 8],
                [9, 10, 11, 12],
                [13, 14, 15, 16]
            ])
        );
    }

    // #[test]
    // fn test_mul() {
    //     let a = 123;
    //     let b = 111;
    //     let c = 23;
    //     assert_eq!(galois_mul(a, b), log_sum_exp(a, b));
    //     assert_eq!(galois_mul(a, c), log_sum_exp(c, a));
    //     assert_eq!(galois_mul(b, 0), log_sum_exp(0, b));
    // }

    #[test]
    fn test_key_manager() {
        // 128 bits
        let a = AES::new(&(1..17).into_iter().collect::<Vec<u8>>());
        assert_eq!(
            [
                0xBC, 0xC4, 0x14, 0x42, 0x6F, 0x1A, 0x5C, 0x73, 0xA1, 0x81, 0x62, 0x65, 0xB1, 0xB1,
                0x40, 0x87
            ],
            a.keys[a.round]
        );
        // 256 bits
        let a = AES::new(&(1..33).into_iter().collect::<Vec<u8>>());
        assert_eq!(
            [
                0xAF, 0x06, 0x48, 0x99, 0x45, 0xED, 0x58, 0x3A, 0xAF, 0x70, 0x0C, 0xCF, 0x95, 0x76,
                0xC8, 0xB2
            ],
            a.keys[a.round]
        );
    }

    #[test]
    fn test_ecb() {
        let a = AES::new(&(1..33).into_iter().collect::<Vec<u8>>());

        let m = "The Advanced Encryption Standard (AES), also known by its original name Rijndael (Dutch pronunciation: [ˈrɛindaːl]),[3] is a specification for the encryption of electronic data established by the U.S. National Institute of Standards and Technology (NIST) in 2001.";
        let mut ms = m.to_string();

        let n = m.len();
        let mut i = n;
        while i % N2 != 0 {
            ms.push('\0');
            i += 1;
        }
        let c = a.encode_ecb(ms.as_bytes());

        assert_eq!(
            &c[256..],
            &[22, 173, 32, 53, 47, 237, 153, 96, 7, 5, 110, 246, 221, 14, 68, 209]
        );
        //dbg!(&c);
        let m2 = a.decode_ecb(&c);
        assert_eq!(String::from_utf8_lossy(&m2[..n]), m);
    }

    fn format_hex4(bytes: &[u8]) -> String {
        let mut res = String::new();
        for (i, v) in bytes.iter().enumerate() {
            if i != 0 && i % 4 == 0 {
                res.push(' ');
            }
            res.push_str(&format!("{:02X}", v));
        }
        res
    }

    #[test]
    fn test_ige() {
        // see https://mgp25.com/AESIGE/
        let a = AES::new(&(0..16).into_iter().collect::<Vec<u8>>());
        let iv1 = ByteSquare::from_col(&(0..16).into_iter().collect::<Vec<u8>>());
        let iv2 = ByteSquare::from_col(&(16..32).into_iter().collect::<Vec<u8>>());
        let block = vec![0; 32];

        let cipher = a.encode_ige(&block, iv1, iv2);
        assert_eq!(
            format_hex4(&cipher),
            "1A8519A6 557BE652 E9DA8E43 DA4EF445 3CF456B4 CA488AA3 83C79C98 B34797CB"
        );

        let origin = a.decode_ige(&cipher, iv1, iv2);
        assert_eq!(
            format_hex4(&origin),
            "00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000"
        );

        let a = AES::new(
            &hex_to_bytes("54686973 20697320 616E2069 6D706C65".replace(' ', "")).unwrap(),
        );

        let iv1 = ByteSquare::from_col(
            &hex_to_bytes("6D656E74 6174696F 6E206F66 20494745".replace(' ', "")).unwrap(),
        );
        let iv2 = ByteSquare::from_col(
            &hex_to_bytes("206D6F64 6520666F 72204F70 656E5353".replace(' ', "")).unwrap(),
        );

        let block = hex_to_bytes(
            "99706487 A1CDE613 BC6DE0B6 F24B1C7A A448C8B9 C3403E34 67A8CAD8 9340F53B"
                .replace(' ', ""),
        )
        .unwrap();

        let cipher = a.encode_ige(&block, iv1, iv2);
        assert_eq!(
            format_hex4(&cipher),
            "4C2E204C 65742773 20686F70 65204265 6E20676F 74206974 20726967 6874210A"
        );

        let origin = a.decode_ige(&cipher, iv1, iv2);
        assert_eq!(
            format_hex4(&origin),
            "99706487 A1CDE613 BC6DE0B6 F24B1C7A A448C8B9 C3403E34 67A8CAD8 9340F53B"
        )
    }

    #[test]
    fn test_cbc() {
        let a = AES::new(
            &hex_to_bytes("54686973 20697320 616E2069 6D706C65".replace(' ', "")).unwrap(),
        );

        let iv = ByteSquare::from_col(
            &hex_to_bytes("6D656E74 6174696F 6E206F66 20494745".replace(' ', "")).unwrap(),
        );

        let block = hex_to_bytes(
            "99706487 A1CDE613 BC6DE0B6 F24B1C7A A448C8B9 C3403E34 67A8CAD8 9340F53B"
                .replace(' ', ""),
        )
        .unwrap();

        let cipher = a.encode_cbc(&block, iv);
        let origin = a.decode_cbc(&cipher, iv);
        assert_eq!(
            format_hex4(&origin),
            "99706487 A1CDE613 BC6DE0B6 F24B1C7A A448C8B9 C3403E34 67A8CAD8 9340F53B"
        );
    }
}

// TODO:
// 为什么是 10 轮
// 为什么最后一轮, 不需要 mix_cols
// 为什么第一轮需要 add_ (漂白)
