use crate::base64_const::{INVALID_VALUE, STANDARD_DECODE, STANDARD_ENCODE};
use crate::error::DecodeBase64Error;

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
    res[0] = STANDARD_ENCODE[(num >> 18) & 0x3f];
    res[1] = STANDARD_ENCODE[(num >> 12) & 0x3f];
    res[2] = STANDARD_ENCODE[(num >> 6) & 0x3f];
    res[3] = STANDARD_ENCODE[num & 0x3f];
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
