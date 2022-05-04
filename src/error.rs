use std::fmt;

#[derive(Debug)]
pub enum DecodeHexError {
    OddLength(usize),
    InvalidChar { c: char, idx: usize },
}

impl std::error::Error for DecodeHexError {} // Error trait

impl fmt::Display for DecodeHexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::OddLength(n) => write!(f, "The length to decode is odd: `{}`", n),
            Self::InvalidChar { c, idx } => write!(f, "Invalid char at {}: `{}`", idx, c),
        }
    }
}

#[derive(Debug)]
pub enum DecodeBase64Error {
    InvalidLength(usize),
    InvalidChar { c: char, idx: usize },
}

impl std::error::Error for DecodeBase64Error {} // Error trait

impl fmt::Display for DecodeBase64Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::InvalidLength(n) if n == 0 => write!(f, "Empty string to decode"),
            Self::InvalidLength(n) => {
                write!(f, "The length to decode is invalid: `{}` (% 4 != 0)", n)
            }
            Self::InvalidChar { c, idx } => write!(f, "Invalid char at {}: `{}`", idx, c),
        }
    }
}

#[derive(Debug)]
pub enum DecodeBase58Error {
    InvalidChar { c: char, idx: usize },
}

impl std::error::Error for DecodeBase58Error {} // Error trait

impl fmt::Display for DecodeBase58Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidChar { c, idx } => write!(f, "Invalid char at {}: `{}`", idx, c),
        }
    }
}

#[test]
fn test_error() {
    // hex
    assert_eq!(
        DecodeHexError::OddLength(5).to_string(),
        "The length to decode is odd: `5`"
    );
    assert_eq!(
        DecodeHexError::InvalidChar { c: 'g', idx: 7 }.to_string(),
        "Invalid char at 7: `g`"
    );

    // base64
    assert_eq!(
        DecodeBase64Error::InvalidLength(0).to_string(),
        "Empty string to decode"
    );
    assert_eq!(
        DecodeBase64Error::InvalidLength(5).to_string(),
        "The length to decode is invalid: `5` (% 4 != 0)"
    );
    assert_eq!(
        DecodeBase64Error::InvalidChar { c: '~', idx: 7 }.to_string(),
        "Invalid char at 7: `~`"
    );

    // base58
    assert_eq!(
        DecodeBase58Error::InvalidChar { c: '~', idx: 7 }.to_string(),
        "Invalid char at 7: `~`"
    );
}
