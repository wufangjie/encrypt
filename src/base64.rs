use crate::error::DecodeBase64Error;

const BASE64_CHARS: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const INVALID_VALUE: u8 = 0xff;

// copy from base64 crate
pub const STANDARD_DECODE: &[u8; 256] = &[
    INVALID_VALUE, // input 0 (0x0)
    INVALID_VALUE, // input 1 (0x1)
    INVALID_VALUE, // input 2 (0x2)
    INVALID_VALUE, // input 3 (0x3)
    INVALID_VALUE, // input 4 (0x4)
    INVALID_VALUE, // input 5 (0x5)
    INVALID_VALUE, // input 6 (0x6)
    INVALID_VALUE, // input 7 (0x7)
    INVALID_VALUE, // input 8 (0x8)
    INVALID_VALUE, // input 9 (0x9)
    INVALID_VALUE, // input 10 (0xA)
    INVALID_VALUE, // input 11 (0xB)
    INVALID_VALUE, // input 12 (0xC)
    INVALID_VALUE, // input 13 (0xD)
    INVALID_VALUE, // input 14 (0xE)
    INVALID_VALUE, // input 15 (0xF)
    INVALID_VALUE, // input 16 (0x10)
    INVALID_VALUE, // input 17 (0x11)
    INVALID_VALUE, // input 18 (0x12)
    INVALID_VALUE, // input 19 (0x13)
    INVALID_VALUE, // input 20 (0x14)
    INVALID_VALUE, // input 21 (0x15)
    INVALID_VALUE, // input 22 (0x16)
    INVALID_VALUE, // input 23 (0x17)
    INVALID_VALUE, // input 24 (0x18)
    INVALID_VALUE, // input 25 (0x19)
    INVALID_VALUE, // input 26 (0x1A)
    INVALID_VALUE, // input 27 (0x1B)
    INVALID_VALUE, // input 28 (0x1C)
    INVALID_VALUE, // input 29 (0x1D)
    INVALID_VALUE, // input 30 (0x1E)
    INVALID_VALUE, // input 31 (0x1F)
    INVALID_VALUE, // input 32 (0x20)
    INVALID_VALUE, // input 33 (0x21)
    INVALID_VALUE, // input 34 (0x22)
    INVALID_VALUE, // input 35 (0x23)
    INVALID_VALUE, // input 36 (0x24)
    INVALID_VALUE, // input 37 (0x25)
    INVALID_VALUE, // input 38 (0x26)
    INVALID_VALUE, // input 39 (0x27)
    INVALID_VALUE, // input 40 (0x28)
    INVALID_VALUE, // input 41 (0x29)
    INVALID_VALUE, // input 42 (0x2A)
    62,            // input 43 (0x2B char '+') => 62 (0x3E)
    INVALID_VALUE, // input 44 (0x2C)
    INVALID_VALUE, // input 45 (0x2D)
    INVALID_VALUE, // input 46 (0x2E)
    63,            // input 47 (0x2F char '/') => 63 (0x3F)
    52,            // input 48 (0x30 char '0') => 52 (0x34)
    53,            // input 49 (0x31 char '1') => 53 (0x35)
    54,            // input 50 (0x32 char '2') => 54 (0x36)
    55,            // input 51 (0x33 char '3') => 55 (0x37)
    56,            // input 52 (0x34 char '4') => 56 (0x38)
    57,            // input 53 (0x35 char '5') => 57 (0x39)
    58,            // input 54 (0x36 char '6') => 58 (0x3A)
    59,            // input 55 (0x37 char '7') => 59 (0x3B)
    60,            // input 56 (0x38 char '8') => 60 (0x3C)
    61,            // input 57 (0x39 char '9') => 61 (0x3D)
    INVALID_VALUE, // input 58 (0x3A)
    INVALID_VALUE, // input 59 (0x3B)
    INVALID_VALUE, // input 60 (0x3C)
    0,             // input 61 (0x3D)
    INVALID_VALUE, // input 62 (0x3E)
    INVALID_VALUE, // input 63 (0x3F)
    INVALID_VALUE, // input 64 (0x40)
    0,             // input 65 (0x41 char 'A') => 0 (0x0)
    1,             // input 66 (0x42 char 'B') => 1 (0x1)
    2,             // input 67 (0x43 char 'C') => 2 (0x2)
    3,             // input 68 (0x44 char 'D') => 3 (0x3)
    4,             // input 69 (0x45 char 'E') => 4 (0x4)
    5,             // input 70 (0x46 char 'F') => 5 (0x5)
    6,             // input 71 (0x47 char 'G') => 6 (0x6)
    7,             // input 72 (0x48 char 'H') => 7 (0x7)
    8,             // input 73 (0x49 char 'I') => 8 (0x8)
    9,             // input 74 (0x4A char 'J') => 9 (0x9)
    10,            // input 75 (0x4B char 'K') => 10 (0xA)
    11,            // input 76 (0x4C char 'L') => 11 (0xB)
    12,            // input 77 (0x4D char 'M') => 12 (0xC)
    13,            // input 78 (0x4E char 'N') => 13 (0xD)
    14,            // input 79 (0x4F char 'O') => 14 (0xE)
    15,            // input 80 (0x50 char 'P') => 15 (0xF)
    16,            // input 81 (0x51 char 'Q') => 16 (0x10)
    17,            // input 82 (0x52 char 'R') => 17 (0x11)
    18,            // input 83 (0x53 char 'S') => 18 (0x12)
    19,            // input 84 (0x54 char 'T') => 19 (0x13)
    20,            // input 85 (0x55 char 'U') => 20 (0x14)
    21,            // input 86 (0x56 char 'V') => 21 (0x15)
    22,            // input 87 (0x57 char 'W') => 22 (0x16)
    23,            // input 88 (0x58 char 'X') => 23 (0x17)
    24,            // input 89 (0x59 char 'Y') => 24 (0x18)
    25,            // input 90 (0x5A char 'Z') => 25 (0x19)
    INVALID_VALUE, // input 91 (0x5B)
    INVALID_VALUE, // input 92 (0x5C)
    INVALID_VALUE, // input 93 (0x5D)
    INVALID_VALUE, // input 94 (0x5E)
    INVALID_VALUE, // input 95 (0x5F)
    INVALID_VALUE, // input 96 (0x60)
    26,            // input 97 (0x61 char 'a') => 26 (0x1A)
    27,            // input 98 (0x62 char 'b') => 27 (0x1B)
    28,            // input 99 (0x63 char 'c') => 28 (0x1C)
    29,            // input 100 (0x64 char 'd') => 29 (0x1D)
    30,            // input 101 (0x65 char 'e') => 30 (0x1E)
    31,            // input 102 (0x66 char 'f') => 31 (0x1F)
    32,            // input 103 (0x67 char 'g') => 32 (0x20)
    33,            // input 104 (0x68 char 'h') => 33 (0x21)
    34,            // input 105 (0x69 char 'i') => 34 (0x22)
    35,            // input 106 (0x6A char 'j') => 35 (0x23)
    36,            // input 107 (0x6B char 'k') => 36 (0x24)
    37,            // input 108 (0x6C char 'l') => 37 (0x25)
    38,            // input 109 (0x6D char 'm') => 38 (0x26)
    39,            // input 110 (0x6E char 'n') => 39 (0x27)
    40,            // input 111 (0x6F char 'o') => 40 (0x28)
    41,            // input 112 (0x70 char 'p') => 41 (0x29)
    42,            // input 113 (0x71 char 'q') => 42 (0x2A)
    43,            // input 114 (0x72 char 'r') => 43 (0x2B)
    44,            // input 115 (0x73 char 's') => 44 (0x2C)
    45,            // input 116 (0x74 char 't') => 45 (0x2D)
    46,            // input 117 (0x75 char 'u') => 46 (0x2E)
    47,            // input 118 (0x76 char 'v') => 47 (0x2F)
    48,            // input 119 (0x77 char 'w') => 48 (0x30)
    49,            // input 120 (0x78 char 'x') => 49 (0x31)
    50,            // input 121 (0x79 char 'y') => 50 (0x32)
    51,            // input 122 (0x7A char 'z') => 51 (0x33)
    INVALID_VALUE, // input 123 (0x7B)
    INVALID_VALUE, // input 124 (0x7C)
    INVALID_VALUE, // input 125 (0x7D)
    INVALID_VALUE, // input 126 (0x7E)
    INVALID_VALUE, // input 127 (0x7F)
    INVALID_VALUE, // input 128 (0x80)
    INVALID_VALUE, // input 129 (0x81)
    INVALID_VALUE, // input 130 (0x82)
    INVALID_VALUE, // input 131 (0x83)
    INVALID_VALUE, // input 132 (0x84)
    INVALID_VALUE, // input 133 (0x85)
    INVALID_VALUE, // input 134 (0x86)
    INVALID_VALUE, // input 135 (0x87)
    INVALID_VALUE, // input 136 (0x88)
    INVALID_VALUE, // input 137 (0x89)
    INVALID_VALUE, // input 138 (0x8A)
    INVALID_VALUE, // input 139 (0x8B)
    INVALID_VALUE, // input 140 (0x8C)
    INVALID_VALUE, // input 141 (0x8D)
    INVALID_VALUE, // input 142 (0x8E)
    INVALID_VALUE, // input 143 (0x8F)
    INVALID_VALUE, // input 144 (0x90)
    INVALID_VALUE, // input 145 (0x91)
    INVALID_VALUE, // input 146 (0x92)
    INVALID_VALUE, // input 147 (0x93)
    INVALID_VALUE, // input 148 (0x94)
    INVALID_VALUE, // input 149 (0x95)
    INVALID_VALUE, // input 150 (0x96)
    INVALID_VALUE, // input 151 (0x97)
    INVALID_VALUE, // input 152 (0x98)
    INVALID_VALUE, // input 153 (0x99)
    INVALID_VALUE, // input 154 (0x9A)
    INVALID_VALUE, // input 155 (0x9B)
    INVALID_VALUE, // input 156 (0x9C)
    INVALID_VALUE, // input 157 (0x9D)
    INVALID_VALUE, // input 158 (0x9E)
    INVALID_VALUE, // input 159 (0x9F)
    INVALID_VALUE, // input 160 (0xA0)
    INVALID_VALUE, // input 161 (0xA1)
    INVALID_VALUE, // input 162 (0xA2)
    INVALID_VALUE, // input 163 (0xA3)
    INVALID_VALUE, // input 164 (0xA4)
    INVALID_VALUE, // input 165 (0xA5)
    INVALID_VALUE, // input 166 (0xA6)
    INVALID_VALUE, // input 167 (0xA7)
    INVALID_VALUE, // input 168 (0xA8)
    INVALID_VALUE, // input 169 (0xA9)
    INVALID_VALUE, // input 170 (0xAA)
    INVALID_VALUE, // input 171 (0xAB)
    INVALID_VALUE, // input 172 (0xAC)
    INVALID_VALUE, // input 173 (0xAD)
    INVALID_VALUE, // input 174 (0xAE)
    INVALID_VALUE, // input 175 (0xAF)
    INVALID_VALUE, // input 176 (0xB0)
    INVALID_VALUE, // input 177 (0xB1)
    INVALID_VALUE, // input 178 (0xB2)
    INVALID_VALUE, // input 179 (0xB3)
    INVALID_VALUE, // input 180 (0xB4)
    INVALID_VALUE, // input 181 (0xB5)
    INVALID_VALUE, // input 182 (0xB6)
    INVALID_VALUE, // input 183 (0xB7)
    INVALID_VALUE, // input 184 (0xB8)
    INVALID_VALUE, // input 185 (0xB9)
    INVALID_VALUE, // input 186 (0xBA)
    INVALID_VALUE, // input 187 (0xBB)
    INVALID_VALUE, // input 188 (0xBC)
    INVALID_VALUE, // input 189 (0xBD)
    INVALID_VALUE, // input 190 (0xBE)
    INVALID_VALUE, // input 191 (0xBF)
    INVALID_VALUE, // input 192 (0xC0)
    INVALID_VALUE, // input 193 (0xC1)
    INVALID_VALUE, // input 194 (0xC2)
    INVALID_VALUE, // input 195 (0xC3)
    INVALID_VALUE, // input 196 (0xC4)
    INVALID_VALUE, // input 197 (0xC5)
    INVALID_VALUE, // input 198 (0xC6)
    INVALID_VALUE, // input 199 (0xC7)
    INVALID_VALUE, // input 200 (0xC8)
    INVALID_VALUE, // input 201 (0xC9)
    INVALID_VALUE, // input 202 (0xCA)
    INVALID_VALUE, // input 203 (0xCB)
    INVALID_VALUE, // input 204 (0xCC)
    INVALID_VALUE, // input 205 (0xCD)
    INVALID_VALUE, // input 206 (0xCE)
    INVALID_VALUE, // input 207 (0xCF)
    INVALID_VALUE, // input 208 (0xD0)
    INVALID_VALUE, // input 209 (0xD1)
    INVALID_VALUE, // input 210 (0xD2)
    INVALID_VALUE, // input 211 (0xD3)
    INVALID_VALUE, // input 212 (0xD4)
    INVALID_VALUE, // input 213 (0xD5)
    INVALID_VALUE, // input 214 (0xD6)
    INVALID_VALUE, // input 215 (0xD7)
    INVALID_VALUE, // input 216 (0xD8)
    INVALID_VALUE, // input 217 (0xD9)
    INVALID_VALUE, // input 218 (0xDA)
    INVALID_VALUE, // input 219 (0xDB)
    INVALID_VALUE, // input 220 (0xDC)
    INVALID_VALUE, // input 221 (0xDD)
    INVALID_VALUE, // input 222 (0xDE)
    INVALID_VALUE, // input 223 (0xDF)
    INVALID_VALUE, // input 224 (0xE0)
    INVALID_VALUE, // input 225 (0xE1)
    INVALID_VALUE, // input 226 (0xE2)
    INVALID_VALUE, // input 227 (0xE3)
    INVALID_VALUE, // input 228 (0xE4)
    INVALID_VALUE, // input 229 (0xE5)
    INVALID_VALUE, // input 230 (0xE6)
    INVALID_VALUE, // input 231 (0xE7)
    INVALID_VALUE, // input 232 (0xE8)
    INVALID_VALUE, // input 233 (0xE9)
    INVALID_VALUE, // input 234 (0xEA)
    INVALID_VALUE, // input 235 (0xEB)
    INVALID_VALUE, // input 236 (0xEC)
    INVALID_VALUE, // input 237 (0xED)
    INVALID_VALUE, // input 238 (0xEE)
    INVALID_VALUE, // input 239 (0xEF)
    INVALID_VALUE, // input 240 (0xF0)
    INVALID_VALUE, // input 241 (0xF1)
    INVALID_VALUE, // input 242 (0xF2)
    INVALID_VALUE, // input 243 (0xF3)
    INVALID_VALUE, // input 244 (0xF4)
    INVALID_VALUE, // input 245 (0xF5)
    INVALID_VALUE, // input 246 (0xF6)
    INVALID_VALUE, // input 247 (0xF7)
    INVALID_VALUE, // input 248 (0xF8)
    INVALID_VALUE, // input 249 (0xF9)
    INVALID_VALUE, // input 250 (0xFA)
    INVALID_VALUE, // input 251 (0xFB)
    INVALID_VALUE, // input 252 (0xFC)
    INVALID_VALUE, // input 253 (0xFD)
    INVALID_VALUE, // input 254 (0xFE)
    INVALID_VALUE, // input 255 (0xFF)
];

pub fn encode(msg: &[u8]) -> Vec<u8> {
    let n = msg.len();
    let n_block = n / 3;
    let m = match n % 3 {
        0 => n_block << 2,
        _ => (n_block + 1) << 2,
    };
    let mut res = vec![0; m];

    let mut msg_i = 0;
    let mut res_i = 0;
    for _ in 0..n_block {
        // NOTE: [lo..hi] 这种形式可以减少 bound check, 提高速度
        encode_3bytes(&msg[msg_i..msg_i + 3], &mut res[res_i..res_i + 4]);
        msg_i += 3;
        res_i += 4;
    }

    match n % 3 {
        0 => (),
        x => {
            let mut last = [0; 3];
            last[0..x].copy_from_slice(&msg[n - x..n]);
            encode_3bytes(&last, &mut res[n_block * 4..m]);
            for i in x..3 {
                res[m - 3 + i] = b'=';
            }
        }
    }
    res
}

#[inline]
fn encode_3bytes(msg: &[u8], res: &mut [u8]) {
    let num = (msg[0] as usize) << 16 | (msg[1] as usize) << 8 | msg[2] as usize;
    res[0] = BASE64_CHARS[(num >> 18) & 0x3f];
    res[1] = BASE64_CHARS[(num >> 12) & 0x3f];
    res[2] = BASE64_CHARS[(num >> 6) & 0x3f];
    res[3] = BASE64_CHARS[num & 0x3f];
}

pub fn decode(msg: &[u8]) -> Result<Vec<u8>, DecodeBase64Error> {
    let n = msg.len();
    if n % 4 != 0 || n == 0 {
        return Err(DecodeBase64Error::InvalidLength(n));
    }
    let n_block = n / 4;
    let mut res = vec![0; n / 4 * 3 + 1];

    let mut msg_i = 0;
    let mut res_i = 0;
    for _ in 0..n_block {
        decode_4bytes(&msg[msg_i..msg_i + 4], &mut res[res_i..res_i + 4], msg_i)?;
        msg_i += 4;
        res_i += 3;
    }

    if b'=' == msg[n - 2] {
        res.pop();
    }
    if b'=' == msg[n - 1] {
        res.pop();
    }
    res.pop();
    Ok(res)
}

#[inline]
fn decode_4bytes(msg: &[u8], res: &mut [u8], i: usize) -> Result<(), DecodeBase64Error> {
    let num = decode_char(msg[0], i)? << 26
        | decode_char(msg[1], i + 1)? << 20
        | decode_char(msg[2], i + 2)? << 14
        | decode_char(msg[3], i + 3)? << 8;
    res[..4].copy_from_slice(&num.to_be_bytes());
    Ok(())
}

#[inline]
fn decode_char(v: u8, idx: usize) -> Result<u32, DecodeBase64Error> {
    match STANDARD_DECODE[v as usize] {
        INVALID_VALUE => Err(DecodeBase64Error::InvalidChar { c: v as char, idx }),
        d => Ok(d as u32),
    }
    // NOTE: match is slower than table lookup
    // Ok((match v {
    //     b'A'..=b'Z' => v - b'A',
    //     b'a'..=b'z' => v - b'a' + 26,
    //     b'0'..=b'9' => v - b'0' + 52,
    //     b'+' => 62,
    //     b'/' => 63,
    //     b'=' => 0, // actually only tailing `=`s are allowed
    //     _ => return Err(DecodeBase64Error::InvalidChar { c: v as char, idx }),
    // }) as usize)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::conv;

    #[test]
    fn test_base64() {
        assert_eq!(
            "aGVsbG8gd29ybGQ=",
            conv::bytes_to_string(&encode(b"hello world"))
        );
        assert_eq!(
            "aGVsbG8gd29ybGQh",
            conv::bytes_to_string(&encode(b"hello world!"))
        );
        assert_eq!(
            "aGVsbG8gd29ybGQhIQ==",
            conv::bytes_to_string(&encode(b"hello world!!"))
        );

        assert_eq!(
            "hello world",
            conv::bytes_to_string(&decode(b"aGVsbG8gd29ybGQ=").unwrap())
        );
        assert_eq!(
            "hello world!",
            conv::bytes_to_string(&decode(b"aGVsbG8gd29ybGQh").unwrap())
        );
        assert_eq!(
            "hello world!!",
            conv::bytes_to_string(&decode(b"aGVsbG8gd29ybGQhIQ==").unwrap())
        );
    }
}
