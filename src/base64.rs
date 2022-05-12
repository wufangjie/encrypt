use crate::error::DecodeBase64Error;

const BASE64_CHARS: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub struct Base64;

impl Base64 {
    pub fn encode(msg: &[u8]) -> Vec<u8> {
        let n = msg.len();
        let n_block = n / 3;
        let m = match n % 3 {
            0 => n_block << 2,
            _ => (n_block + 1) << 2,
        };
        let mut res = vec![0; m];

        for i in 0..n_block {
            Self::encode_3bytes(&msg[i * 3..], &mut res[i << 2..]);
        }

        match n % 3 {
            0 => (),
            x => {
                let mut last = [0; 3];
                last[0..x].copy_from_slice(&msg[n - x..]);
                Self::encode_3bytes(&last, &mut res[n_block << 2..]);
                for i in x..3 {
                    res[m - 3 + i] = b'=';
                }
            }
        }
        res
    }

    #[inline(always)]
    fn encode_3bytes(msg: &[u8], res: &mut [u8]) {
        let i3 = (msg[0] as usize) << 16 | (msg[1] as usize) << 8 | msg[2] as usize;
        res[0] = BASE64_CHARS[(i3 >> 18) & 0x3f];
        res[1] = BASE64_CHARS[(i3 >> 12) & 0x3f];
        res[2] = BASE64_CHARS[(i3 >> 6) & 0x3f];
        res[3] = BASE64_CHARS[i3 & 0x3f];
    }

    pub fn decode(msg: &[u8]) -> Result<Vec<u8>, DecodeBase64Error> {
        let n = msg.len();
        if n % 4 != 0 || n == 0 {
            return Err(DecodeBase64Error::InvalidLength(n));
        }

        let count_equal = if let b'=' = msg[n - 2] {
            2
        } else if let b'=' = msg[n - 1] {
            1
        } else {
            0
        };

        let mut res = vec![0xff; n / 4 * 3]; //Vec::with_capacity(n / 4 * 3);
        for (i, slc) in msg.chunks(4).enumerate() {
            Self::decode_4bytes(slc, &mut res[i * 3..], i)?;
        }
        for _ in 0..count_equal {
            res.pop();
        }
        Ok(res)
    }

    #[inline(always)]
    fn decode_4bytes(msg: &[u8], res: &mut [u8], i: usize) -> Result<(), DecodeBase64Error> {
        let idx = i << 2;
        let i3 = Self::decode_char(msg[0], idx)? << 18
            | Self::decode_char(msg[1], idx + 1)? << 12
            | Self::decode_char(msg[2], idx + 2)? << 6
            | Self::decode_char(msg[3], idx + 3)?;

        res[0] &= (i3 >> 16) as u8;
        res[1] &= (i3 >> 8) as u8;
        res[2] &= i3 as u8;
        Ok(())
    }

    #[inline]
    fn decode_char(v: u8, idx: usize) -> Result<usize, DecodeBase64Error> {
        Ok((match v {
            b'A'..=b'Z' => v - b'A',
            b'a'..=b'z' => v - b'a' + 26,
            b'0'..=b'9' => v - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => 0, // actually only tailing `=`s are allowed
            _ => return Err(DecodeBase64Error::InvalidChar { c: v as char, idx }),
        }) as usize)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::conv;

    #[test]
    fn test_base64() {
        assert_eq!(
            "aGVsbG8gd29ybGQ=",
            conv::bytes_to_string(&Base64::encode(b"hello world"))
        );
        assert_eq!(
            "aGVsbG8gd29ybGQh",
            conv::bytes_to_string(&Base64::encode(b"hello world!"))
        );
        assert_eq!(
            "aGVsbG8gd29ybGQhIQ==",
            conv::bytes_to_string(&Base64::encode(b"hello world!!"))
        );

        assert_eq!(
            "hello world",
            conv::bytes_to_string(&Base64::decode(b"aGVsbG8gd29ybGQ=").unwrap())
        );
        assert_eq!(
            "hello world!",
            conv::bytes_to_string(&Base64::decode(b"aGVsbG8gd29ybGQh").unwrap())
        );
        assert_eq!(
            "hello world!!",
            conv::bytes_to_string(&Base64::decode(b"aGVsbG8gd29ybGQhIQ==").unwrap())
        );
    }
}
