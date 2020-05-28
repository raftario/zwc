#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "camouflage")]
pub use camouflage::*;

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

#[cfg(feature = "camouflage")]
mod camouflage {
    use std::fmt;

    /// Camouflages a compressed and optionally ciphered payload inside a string
    pub fn camouflage(
        mut payload: Vec<u8>,
        dummy: &str,
        compression_level: i32,
        key: Option<&str>,
    ) -> Result<String, CamouflageError> {
        use brotli::enc::BrotliEncoderParams;
        use chacha20::ChaCha20Rng;
        use chacha20poly1305::aead::Aead;
        use generic_array::GenericArray;
        use rand_core::{RngCore, SeedableRng};

        if let Some(k) = key {
            let cipher = get_cipher(k);

            let mut nonce = [0; 12];
            ChaCha20Rng::from_entropy().fill_bytes(&mut nonce);

            cipher.encrypt_in_place(GenericArray::from_slice(&nonce), b"", &mut payload)?;

            payload.extend_from_slice(&nonce);
        }

        let mut compressed_payload = Vec::with_capacity(payload.len());
        brotli::BrotliCompress(
            &mut payload.as_slice(),
            &mut compressed_payload,
            &BrotliEncoderParams {
                quality: compression_level,
                size_hint: payload.len(),
                ..Default::default()
            },
        )?;

        let mut camouflaged = String::with_capacity((compressed_payload.len() * 6) + dummy.len());
        let mut encoded_payload = crate::zwc_encode(compressed_payload.iter().copied());

        for c in dummy.chars() {
            camouflaged.push(c);
            if c.is_ascii_whitespace() {
                for c in &mut encoded_payload {
                    camouflaged.push(c);
                }
            }
        }
        for c in encoded_payload {
            camouflaged.push(c);
        }

        Ok(camouflaged)
    }

    /// Extracts a previously camouflaged payload from a string
    pub fn decamouflage(camouflaged: &str, key: Option<&str>) -> Result<Vec<u8>, CamouflageError> {
        use brotli::BrotliDecompress;
        use chacha20poly1305::aead::Aead;
        use generic_array::GenericArray;

        let encoded_payload = camouflaged.chars().filter(|c| crate::is_zwc(*c));
        let compressed_payload =
            crate::zwc_decode(encoded_payload).collect::<Result<Vec<u8>, _>>()?;

        let mut payload = Vec::with_capacity(compressed_payload.len() * 4);
        BrotliDecompress(&mut compressed_payload.as_slice(), &mut payload)?;

        if let Some(k) = key {
            let cipher = get_cipher(k);

            let nonce_boundary = payload.len() - 12;
            let nonce = GenericArray::clone_from_slice(&payload[nonce_boundary..]);
            payload.truncate(nonce_boundary);

            cipher.decrypt_in_place(&nonce, b"", &mut payload)?;
        }

        Ok(payload)
    }

    /// Represents an error that might occur while camouflaging or decamouflaging a payload
    #[derive(Debug)]
    pub enum CamouflageError {
        ZwcDecode(crate::ZwcDecodeError),
        Cipher(chacha20poly1305::aead::Error),
        Brotli(std::io::Error),
    }
    impl fmt::Display for CamouflageError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::ZwcDecode(e) => write!(f, "zero-width character decoding error: {}", e),
                Self::Cipher(e) => write!(f, "cipher error: {:?}", e),
                Self::Brotli(e) => write!(f, "brotli error: {}", e),
            }
        }
    }
    impl std::error::Error for CamouflageError {}
    impl From<crate::ZwcDecodeError> for CamouflageError {
        fn from(e: crate::ZwcDecodeError) -> Self {
            Self::ZwcDecode(e)
        }
    }
    impl From<chacha20poly1305::aead::Error> for CamouflageError {
        fn from(e: chacha20poly1305::aead::Error) -> Self {
            Self::Cipher(e)
        }
    }
    impl From<std::io::Error> for CamouflageError {
        fn from(e: std::io::Error) -> Self {
            Self::Brotli(e)
        }
    }

    /// Creates a cipher from a key
    fn get_cipher(key: &str) -> chacha20poly1305::ChaCha20Poly1305 {
        use chacha20poly1305::{aead::NewAead, ChaCha20Poly1305};
        use generic_array::GenericArray;
        use poly1305::{universal_hash::UniversalHash, Poly1305};

        let key_bytes = key.as_bytes();
        let mut key_hasher_key = [0; 32];
        for i in 0..32 {
            key_hasher_key[i] = key_bytes[i % key_bytes.len()];
        }
        let key_hash = Poly1305::new(GenericArray::from_slice(&key_hasher_key))
            .chain(key_bytes)
            .result()
            .into_bytes();
        let mut key = [0; 32];
        for i in 0..16 {
            key[i] = key_hash[i];
            key[i + 16] = key_hash[i];
        }

        ChaCha20Poly1305::new(GenericArray::clone_from_slice(&key))
    }

    #[cfg(test)]
    mod tests {
        static SRC: &[u8] = include_bytes!("./lib.rs");

        #[test]
        fn round_trip() {
            let dummy = "Hello, World!";

            let camouflaged =
                super::camouflage(SRC.to_vec(), dummy, Some(11), Some("secret")).unwrap();
            let decamouflaged = super::decamouflage(&camouflaged, Some("secret")).unwrap();

            assert_eq!(SRC, decamouflaged.as_slice());
        }
    }
}

#[cfg(test)]
mod tests {
    static SRC: &[u8] = include_bytes!("./lib.rs");

    #[test]
    fn round_trip() {
        let encoded = super::zwc_encode(SRC.iter().copied());
        let decoded = super::zwc_decode(encoded);

        for (ob, db) in SRC.iter().copied().zip(decoded) {
            assert_eq!(ob, db.unwrap());
        }
    }
}
