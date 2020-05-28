#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt;

/// Converts an iterator over bytes into an iterator over zero-width characters
pub fn zwc_encode<T: Iterator<Item = u8>>(iter: T) -> impl Iterator<Item = char> {
    EncodeIter {
        inner: iter,
        buffer: ['\0'; 3],
        cursor: 0,
    }
}

/// Converts an iterator over zero-width characters into an bytes
pub fn zwc_decode<T: Iterator<Item = char>>(
    iter: T,
) -> impl Iterator<Item = Result<u8, ZwcDecodeError>> {
    DecodeIter { inner: iter }
}

/// Checks if a character is zero-width
pub fn is_zwc(c: char) -> bool {
    match c {
        '\u{200C}' => true,
        '\u{200D}' => true,
        '\u{2060}' => true,
        '\u{2061}' => true,
        '\u{2062}' => true,
        '\u{2063}' => true,
        '\u{2064}' => true,
        _ => false,
    }
}

/// Represents an error that might occur while decoding data
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ZwcDecodeError {
    InvalidCharacter(char),
    IncompleteBlock(usize),
}
impl fmt::Display for ZwcDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter(c) => write!(f, "expected zero-width character but got {}", c),
            Self::IncompleteBlock(len) => {
                write!(f, "expected 3 characters in block but got {}", len)
            }
        }
    }
}
#[cfg(feature = "std")]
impl std::error::Error for crate::ZwcDecodeError {}

/// Zero-width characters
const ZWC: [char; 7] = [
    '\u{200C}', '\u{200D}', '\u{2060}', '\u{2061}', '\u{2062}', '\u{2063}', '\u{2064}',
];

/// 7^0
const POW_0: u8 = 1;
/// 7^1
const POW_1: u8 = 7;
/// 7^2
const POW_2: u8 = 49;

/// Converts a byte to a zero-width character block
#[inline]
fn to_zwc_block(b: u8) -> [char; 3] {
    [
        ZWC[((b / POW_0) % 7) as usize],
        ZWC[((b / POW_1) % 7) as usize],
        ZWC[((b / POW_2) % 7) as usize],
    ]
}

/// Converts a zero-width character block to a byte
#[inline]
fn to_byte(c0: char, c1: char, c2: char) -> Result<u8, ZwcDecodeError> {
    Ok((zwc_val(c0)? * POW_0) + (zwc_val(c1)? * POW_1) + (zwc_val(c2)? * POW_2))
}

/// Gets the value of a char if it's zero-width
#[inline]
fn zwc_val(c: char) -> Result<u8, ZwcDecodeError> {
    match c {
        '\u{200C}' => Ok(0),
        '\u{200D}' => Ok(1),
        '\u{2060}' => Ok(2),
        '\u{2061}' => Ok(3),
        '\u{2062}' => Ok(4),
        '\u{2063}' => Ok(5),
        '\u{2064}' => Ok(6),
        _ => Err(ZwcDecodeError::InvalidCharacter(c)),
    }
}

/// Encoding iterator
struct EncodeIter<T: Iterator<Item = u8>> {
    inner: T,
    buffer: [char; 3],
    cursor: usize,
}
impl<T: Iterator<Item = u8>> Iterator for EncodeIter<T> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor == 0 {
            self.buffer = match self.inner.next() {
                Some(b) => to_zwc_block(b),
                None => return None,
            };
        }

        let ret = Some(self.buffer[self.cursor]);
        self.cursor = (self.cursor + 1) % 3;
        ret
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner_size_hint = self.inner.size_hint();
        (
            (inner_size_hint.0 * 6) + (2 - self.cursor),
            inner_size_hint.1.map(|b| (b * 6) + (2 - self.cursor)),
        )
    }
}

/// Decoding iterator
struct DecodeIter<T: Iterator<Item = char>> {
    inner: T,
}
impl<T: Iterator<Item = char>> Iterator for DecodeIter<T> {
    type Item = Result<u8, ZwcDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! match_next {
            ($iter:expr, $err:expr) => {
                match $iter.next() {
                    Some(c) => c,
                    None => return $err,
                }
            };
        }

        let c0 = match_next!(self.inner, None);
        let c1 = match_next!(self.inner, Some(Err(ZwcDecodeError::IncompleteBlock(1))));
        let c2 = match_next!(self.inner, Some(Err(ZwcDecodeError::IncompleteBlock(2))));
        Some(to_byte(c0, c1, c2))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner_size_hint = self.inner.size_hint();
        (inner_size_hint.0 / 6, inner_size_hint.1.map(|b| b / 6))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn round_trip() {
        let original = include_bytes!("./lib.rs");

        let encoded = super::zwc_encode(original.iter().copied());
        let decoded = super::zwc_decode(encoded);

        for (ob, db) in original.iter().copied().zip(decoded) {
            assert_eq!(ob, db.unwrap());
        }
    }
}
