use crate::error::DecodeBase64Error;

const BASE64_CHARS: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn encode_base64(lst: &[u8]) -> Vec<u8> {
    // let mut ret = vec![];
    // for slc in lst.chunks(3) {
    //     ret.extend(encode_base64_3bytes(slc));
    // }
    // ret;
    lst.chunks(3).flat_map(encode_base64_3bytes).collect()
}

fn encode_base64_3bytes(lst: &[u8]) -> Vec<u8> {
    let i3: usize = lst
        .iter()
        .enumerate()
        .map(|(i, v)| (*v as usize) << (8 * (2 - i)))
        .sum();

    let mut ret = [
        (i3 & (0b111111 << 18)) >> 18,
        (i3 & (0b111111 << 12)) >> 12,
        (i3 & (0b111111 << 6)) >> 6,
        i3 & 0b111111,
    ]
    .into_iter()
    .map(|idx| BASE64_CHARS[idx])
    .collect::<Vec<u8>>();

    match lst.len() {
        2 => ret[3] = b'=',
        1 => {
            ret[2] = b'=';
            ret[3] = b'=';
        }
        _ => (),
    }
    ret
}

fn decode_base64(lst: &[u8]) -> Result<Vec<u8>, DecodeBase64Error> {
    let n = lst.len();
    if n % 4 != 0 || n == 0 {
        return Err(DecodeBase64Error::InvalidLength(n));
    }

    let count_equal = if let b'=' = lst[n - 2] {
        2
    } else if let b'=' = lst[n - 1] {
        1
    } else {
        0
    };

    let mut ret = Vec::with_capacity(n / 4 * 3);
    for slc in lst.chunks(4) {
        ret.extend(decode_base64_4bytes(slc)?);
    }
    for _ in 0..count_equal {
        ret.pop();
    }
    Ok(ret)
}

fn decode_base64_4bytes(lst: &[u8]) -> Result<Vec<u8>, DecodeBase64Error> {
    // let i3: usize = lst
    //     .iter()
    //     .enumerate()
    //     .map(|(i, v)| decode_base64_char(*v, i).unwrap() << (6 * (3 - i)))
    //     .sum();
    let mut i3 = 0usize;
    for (i, v) in lst.iter().enumerate() {
        i3 += decode_base64_char(*v, i)? << (6 * (3 - i));
    }

    Ok([
        (i3 & (0b11111111 << 16)) >> 16,
        (i3 & (0b11111111 << 8)) >> 8,
        i3 & 0b11111111,
    ]
    .into_iter()
    .map(|x| x as u8)
    .collect::<Vec<u8>>())
}

fn decode_base64_char(v: u8, idx: usize) -> Result<usize, DecodeBase64Error> {
    Ok((match v {
        b'A'..=b'Z' => v - b'A',
        b'a'..=b'z' => v - b'a' + 26,
        b'0'..=b'9' => v - b'0' + 52,
        b'=' => 0, // actually only tailing `=`s are allowed
        _ => return Err(DecodeBase64Error::InvalidChar { c: v as char, idx }),
    }) as usize)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::conv;

    #[test]
    fn test_base64() {
        assert_eq!(
            "aGVsbG8gd29ybGQ=",
            conv::bytes_to_string(&encode_base64(b"hello world"))
        );
        assert_eq!(
            "hello world",
            conv::bytes_to_string(&decode_base64(b"aGVsbG8gd29ybGQ=").unwrap())
        );
    }
}
