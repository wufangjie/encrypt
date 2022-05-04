use crate::error::DecodeHexError;

pub fn toggle_endian<T>(lst: Vec<T>) -> Vec<u8>
where
    T: AsRef<[u8]>,
{
    lst.into_iter()
        .flat_map(|x| hex_to_bytes(x).unwrap().into_iter().rev())
        .collect()
}

pub fn str_to_bytes(s: &str) -> Vec<u8> {
    s.to_owned().into_bytes() // as_bytes only get reference
}

pub fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|c| format!("{:02x}", c)).collect()
}

pub fn bytes_to_hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|c| format!("{:02x}", c)).collect()
}

pub fn bytes_to_hex_upper(bytes: &[u8]) -> String {
    bytes.iter().map(|c| format!("{:02X}", c)).collect()
}

pub fn hex_to_bytes<T>(hex_str: T) -> Result<Vec<u8>, DecodeHexError>
where
    T: AsRef<[u8]>,
{
    match hex_str.as_ref().len() {
        n if n & 1 == 1 => Err(DecodeHexError::OddLength(n)),
        _ => hex_str
            .as_ref()
            .chunks(2)
            .enumerate()
            .map(|(i, pair)| Ok(hc2u8(pair[0], i << 1)? << 4 | hc2u8(pair[1], (i << 1) + 1)?))
            .collect(),
    }
}

fn hc2u8(c: u8, idx: usize) -> Result<u8, DecodeHexError> {
    // hex char to u8
    match c {
        b'0'..=b'9' => Ok(c - 48), // n @ seems useless
        b'A'..=b'F' => Ok(c - 55),
        b'a'..=b'f' => Ok(c - 87),
        _ => Err(DecodeHexError::InvalidChar { c: c as char, idx }),
    }
}

#[test]
fn test_convert() {
    let bytes = vec![104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100];
    assert_eq!(bytes, str_to_bytes("hello world"));
    assert_eq!("hello world", &bytes_to_string(&bytes));
    assert_eq!("68656c6c6f20776f726c64", &bytes_to_hex(&bytes));
    assert_eq!("68656C6C6F20776F726C64", &bytes_to_hex_upper(&bytes));
    assert_eq!(bytes, hex_to_bytes(b"68656c6c6f20776f726c64").unwrap());
    assert_eq!(bytes, hex_to_bytes(b"68656C6c6F20776f726C64").unwrap());

    assert!(hex_to_bytes(b"68656C6c6F20776f726G64").is_err());
}
