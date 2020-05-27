#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt;

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

/// Iterator over blocks of 3 zero-width characters
struct ZwcBlockIter<T: Iterator<Item = u8>> {
    iter: T,
}
impl<T: Iterator<Item = u8>> Iterator for ZwcBlockIter<T> {
    type Item = [char; 3];

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(b) => Some([
                ZWC[((b / POW_0) % 7) as usize],
                ZWC[((b / POW_1) % 7) as usize],
                ZWC[((b / POW_2) % 7) as usize],
            ]),
            None => None,
        }
    }
}

/// Iterator over zero-width characters
struct ZwcIter<T: Iterator<Item = u8>> {
    iter: ZwcBlockIter<T>,
    buffer: [char; 3],
    cursor: usize,
}
impl<T: Iterator<Item = u8>> Iterator for ZwcIter<T> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor == 0 {
            self.buffer = match self.iter.next() {
                Some(c) => c,
                None => return None,
            };
        }

        let ret = Some(self.buffer[self.cursor]);
        self.cursor = (self.cursor + 1) % 3;
        ret
    }
}

/// Converts an iterator over bytes into an iterator over zero-width characters
pub fn zwc_encode<T: Iterator<Item = u8>>(iter: T) -> impl Iterator<Item = char> {
    ZwcIter {
        iter: ZwcBlockIter { iter },
        buffer: ['\0'; 3],
        cursor: 0,
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ZwcDecodeError {
    InvalidCharacter { character: char, position: usize },
    IncompleteBlock { len: usize },
}
impl fmt::Display for ZwcDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter {
                character,
                position,
            } => write!(
                f,
                "expected zero-width character but got {} at position {}",
                character, position,
            ),
            Self::IncompleteBlock { len } => {
                write!(f, "expected 3 characters in block but got only {}", len)
            }
        }
    }
}
#[cfg(feature = "std")]
impl std::error::Error for ZwcDecodeError {}

macro_rules! zwc_as_u8 {
    ($c:expr) => {
        match $c.1 {
            '\u{200C}' => 0,
            '\u{200D}' => 1,
            '\u{2060}' => 2,
            '\u{2061}' => 3,
            '\u{2062}' => 4,
            '\u{2063}' => 5,
            '\u{2064}' => 6,
            _ => {
                return Some(Err(ZwcDecodeError::InvalidCharacter {
                    character: $c.1,
                    position: $c.0,
                }))
            }
        }
    };
}

/// Iterator over bytes
struct U8Iter<T: Iterator<Item = (usize, char)>> {
    iter: T,
}
impl<T: Iterator<Item = (usize, char)>> Iterator for U8Iter<T> {
    type Item = Result<u8, ZwcDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        let c1 = match self.iter.next() {
            Some(c) => zwc_as_u8!(c),
            None => return None,
        };
        let c2 = match self.iter.next() {
            Some(c) => zwc_as_u8!(c),
            None => return Some(Err(ZwcDecodeError::IncompleteBlock { len: 1 })),
        };
        let c3 = match self.iter.next() {
            Some(c) => zwc_as_u8!(c),
            None => return Some(Err(ZwcDecodeError::IncompleteBlock { len: 2 })),
        };

        Some(Ok((c1 * POW_0) + (c2 * POW_1) + (c3 * POW_2)))
    }
}

/// Converts an iterator over zero-width characters into an bytes
pub fn zwc_decode<T: Iterator<Item = char>>(
    iter: T,
) -> impl Iterator<Item = Result<u8, ZwcDecodeError>> {
    U8Iter {
        iter: iter.enumerate(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn round_trip() {
        let original = include_bytes!("./lib.rs");
        let encoded = super::zwc_encode(original.iter().copied());
        let decoded: Result<Vec<u8>, super::ZwcDecodeError> = super::zwc_decode(encoded).collect();
        assert_eq!(decoded.unwrap().as_slice(), original.as_ref());
    }
}
