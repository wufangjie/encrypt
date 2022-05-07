//! 只实现 128 bit 密钥的情况

use crate::aes_const::{EXP_TABLE, LOG_TABLE, MIX_MAT, MIX_MAT_INV, RND_CON, SUB_BOX, SUB_BOX_INV};
use std::fmt;
use std::ops::{Deref, DerefMut};

const N: usize = 4;

#[derive(Clone, Copy, PartialEq)]
pub struct ByteSquare {
    pub(crate) data: [[u8; N]; N],
}

impl fmt::Display for ByteSquare {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "ByteSquare {{")?;
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

impl Deref for ByteSquare {
    type Target = [[u8; N]; N];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for ByteSquare {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl ByteSquare {
    pub fn new() -> Self {
        Self { data: [[0; N]; N] }
    }

    // fn from_row(row: &[u8]) -> Self {
    //     // TODO: 处理长度不够的情况
    //     let mut data = [[0; N]; N];
    //     for (i, v) in row.iter().take(N * N).enumerate() {
    //         data[i / N][i % N] = *v;
    //     }
    //     Self { data }
    // }

    pub fn from_col(col: &[u8]) -> Self {
        let mut data = [[0; N]; N];
        for (i, v) in col.iter().take(N * N).enumerate() {
            data[i % N][i / N] = *v;
        }
        Self { data }
    }

    pub fn from_rows(mat: &[[u8; N]]) -> Self {
        let mut data = [[0; N]; N];
        for (i, data_i) in data.iter_mut().enumerate() {
            *data_i = mat[i];
        }
        Self { data }
    }

    pub fn from_cols(mat: &[[u8; N]]) -> Self {
        let mut data = [[0; N]; N];
        for (i, data_i) in data.iter_mut().enumerate() {
            for (j, data_i_j) in data_i.iter_mut().enumerate() {
                *data_i_j = mat[j][i];
            }
        }
        Self { data }
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut res = Vec::with_capacity(N * N);
        for j in 0..N {
            for i in 0..N {
                res.push(self[i][j]);
            }
        }
        res
    }
}

impl ByteSquare {
    fn add(&self, other: &Self) -> Self {
        let mut new = Self::new();
        for i in 0..N {
            for j in 0..N {
                new[i][j] = self[i][j] ^ other[i][j];
            }
        }
        new
    }

    /// 密钥加法 (异或操作, 加密解密同)
    fn add_(&mut self, other: &Self) {
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
                    //res ^= galois_mul(MIX_MAT[i][k], self[k][j]);
                    res ^= log_sum_exp(MIX_MAT[i][k], self[k][j]);
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
                    //res ^= galois_mul(MIX_MAT_INV[i][k], self[k][j]);
                    res ^= log_sum_exp(MIX_MAT_INV[i][k], self[k][j]);
                }
                new[i][j] = res;
            }
        }
        self.data = new;
    }
}

/// polynomial version of GF(2^8) multiplication
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

/// table lookup version of GF(2^8) multiplication
fn log_sum_exp(lhs: u8, rhs: u8) -> u8 {
    if lhs == 0 || rhs == 0 {
        0
    } else {
        let log_sum = LOG_TABLE[lhs as usize] as usize + LOG_TABLE[rhs as usize] as usize;
        EXP_TABLE[log_sum % 0xff] // 0xff loop
    }
}

#[derive(Debug)]
pub struct AES {
    round: usize,
    pub(crate) keys: Vec<ByteSquare>,
}

impl AES {
    /// deprecated
    pub fn new2(key: ByteSquare) -> Self {
        let round = 10;
        let mut keys = Vec::<ByteSquare>::with_capacity(11);
        keys.push(key);

        for r in 0..round {
            let mut new = ByteSquare::new();

            for i in 0..N {
                new[i][0] = SUB_BOX[keys[r][(i + 1) % N][N - 1] as usize] ^ keys[r][i][0];
            }
            new[0][0] ^= RND_CON[r]; //  + 1

            for j in 1..N {
                for i in 0..N {
                    new[i][j] = new[i][j - 1] ^ keys[r][i][j];
                }
            }
            keys.push(new);
        }

        dbg!(&keys[1]);
        Self { round, keys }
    }

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
            for i in 0..N {
                new[i] = row[i];
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

        let mut keys = Vec::<ByteSquare>::with_capacity(1 + round);
        for r in 0..=round {
            let mut key = [[0; N]; N];
            for j in 0..N {
                for (i, key_i) in key.iter_mut().enumerate() {
                    key_i[j] = key_manager[r * N + j][i];
                }
            }
            keys.push(ByteSquare::from_rows(&key));
        }
        Self { round, keys }
    }

    pub fn encode_ecb(&self, msg: &[u8]) -> Vec<u8> {
        // ECB 可以并行计算, CBC 每个 block 开始加密前要先和之前的加密结果 XOR
        let mut res = Vec::new();
        for m in msg.chunks(16) {
            let mut block = ByteSquare::from_col(m);
            self.encode_block(&mut block);
            res.extend(block.to_bytes());
        }
        res
    }

    pub fn decode_ecb(&self, msg: &[u8]) -> Vec<u8> {
        let mut res = Vec::new();
        for m in msg.chunks(16) {
            let mut block = ByteSquare::from_col(m);
            self.decode_block(&mut block);
            res.extend(block.to_bytes());
        }
        res
    }

    pub fn encode_cbc(&self, msg: &[u8], mut iv: ByteSquare) -> Vec<u8> {
        // iv means init vector
        let mut res = Vec::new();
        for m in msg.chunks(16) {
            let mut block = ByteSquare::from_col(m);
            block.add_(&iv);
            self.encode_block(&mut block);
            iv = block;
            res.extend(block.to_bytes());
        }
        res
    }

    pub fn decode_cbc(&self, msg: &[u8], mut iv: ByteSquare) -> Vec<u8> {
        let mut res = Vec::new();
        for m in msg.chunks(16) {
            let mut block = ByteSquare::from_col(m);
            let block_bak = block;
            self.decode_block(&mut block);
            block.add_(&iv);
            iv = block_bak;
            res.extend(block.to_bytes());
        }
        res
    }

    /// decode ige mode (for telegram)
    pub fn encode_ige(&self, msg: &[u8], ivs: (ByteSquare, ByteSquare)) -> Vec<u8> {
        let (mut y_prev, mut x_prev) = ivs;
        let mut res = Vec::new();
        for m in msg.chunks(16) {
            let mut block = ByteSquare::from_col(m);
            let block_bak = block;
            block.add_(&y_prev);
            self.encode_block(&mut block);
            block.add_(&x_prev);
            y_prev = block;
            x_prev = block_bak;
            res.extend(block.to_bytes());
        }
        res
    }

    /// decode ige mode (for telegram)
    pub fn decode_ige(&self, msg: &[u8], ivs: (ByteSquare, ByteSquare)) -> Vec<u8> {
        let (mut y_prev, mut x_prev) = ivs;
        let mut res = Vec::new();
        for m in msg.chunks(16) {
            let mut block = ByteSquare::from_col(m);
            let block_bak = block;
            block.add_(&x_prev);
            self.decode_block(&mut block);
            block.add_(&y_prev);
            x_prev = block;
            y_prev = block_bak;
            res.extend(block.to_bytes());
        }
        res
    }

    fn encode_block(&self, msg: &mut ByteSquare) {
        msg.add_(&self.keys[0]);
        for i in 1..self.round {
            msg.sub();
            msg.shift_row();
            msg.mix_col();
            msg.add_(&self.keys[i])
        }
        msg.sub();
        msg.shift_row();
        msg.add_(&self.keys[self.round]);
    }

    fn decode_block(&self, msg: &mut ByteSquare) {
        msg.add_(&self.keys[self.round]);
        msg.shift_row_inv();
        msg.sub_inv();
        for i in (1..self.round).into_iter().rev() {
            msg.add_(&self.keys[i]);
            msg.mix_col_inv();
            msg.shift_row_inv();
            msg.sub_inv();
        }
        msg.add_(&self.keys[0]);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::conv::hex_to_bytes;

    #[test]
    fn test_mix_col() {
        let e = ByteSquare::from_rows(&[[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]]);
        let mut m = ByteSquare::from_rows(&MIX_MAT);
        let mut n = ByteSquare::from_rows(&MIX_MAT_INV);

        n.mix_col();
        assert_eq!(e, n);
        m.mix_col_inv();
        assert_eq!(e, m);

        n.mix_col_inv();
        assert_eq!(ByteSquare::from_rows(&MIX_MAT_INV), n);
    }

    #[test]
    fn test_mul() {
        let a = 123;
        let b = 111;
        let c = 23;
        assert_eq!(galois_mul(a, b), log_sum_exp(a, b));
        assert_eq!(galois_mul(a, c), log_sum_exp(c, a));
        assert_eq!(galois_mul(b, 0), log_sum_exp(0, b));
    }

    #[test]
    fn test_key_manager() {
        // 128 bits
        let a = AES::new(&(1..17).into_iter().collect::<Vec<u8>>());

        assert_eq!(
            ByteSquare::from_rows(&[
                [0xBC, 0x6F, 0xA1, 0xB1],
                [0xC4, 0x1A, 0x81, 0xB1],
                [0x14, 0x5C, 0x62, 0x40],
                [0x42, 0x73, 0x65, 0x87]
            ]),
            a.keys[a.round]
        );

        // 256 bits
        let a = AES::new(&(1..33).into_iter().collect::<Vec<u8>>());

        assert_eq!(
            ByteSquare::from_rows(&[
                [0xAF, 0x45, 0xAF, 0x95],
                [0x06, 0xED, 0x70, 0x76],
                [0x48, 0x58, 0x0C, 0xC8],
                [0x99, 0x3A, 0xCF, 0xB2]
            ]),
            a.keys[a.round]
        );
    }

    #[test]
    fn test_ecb() {
        let a = AES::new(&(1..33).into_iter().collect::<Vec<u8>>());

        let m = "The Advanced Encryption Standard (AES), also known by its original name Rijndael (Dutch pronunciation: [ˈrɛindaːl]),[3] is a specification for the encryption of electronic data established by the U.S. National Institute of Standards and Technology (NIST) in 2001.";
        //dbg!(m.as_bytes());

        let n = m.len();
        let c = a.encode_ecb(m.as_bytes());

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
        let ivs = (
            ByteSquare::from_col(&(0..16).into_iter().collect::<Vec<u8>>()),
            ByteSquare::from_col(&(16..32).into_iter().collect::<Vec<u8>>()),
        );
        let block = vec![0; 32];

        let cipher = a.encode_ige(&block, ivs);
        assert_eq!(
            format_hex4(&cipher),
            "1A8519A6 557BE652 E9DA8E43 DA4EF445 3CF456B4 CA488AA3 83C79C98 B34797CB"
        );

        let origin = a.decode_ige(&cipher, ivs);
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

        let cipher = a.encode_ige(&block, (iv1, iv2));
        assert_eq!(
            format_hex4(&cipher),
            "4C2E204C 65742773 20686F70 65204265 6E20676F 74206974 20726967 6874210A"
        );

        let origin = a.decode_ige(&cipher, (iv1, iv2));
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
// 为什么最后一轮, 不需要 mix_col
// 为什么第一轮需要 add_ (漂白)
