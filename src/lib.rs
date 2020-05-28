#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "camouflage")]
pub use camouflage::*;

use core::fmt;

/// Zero-width characters
pub const CHARS: [char; 6] = [
    '\u{200C}', '\u{200D}', '\u{2060}', '\u{2062}', '\u{2063}', '\u{2064}',
];

const CHAR0: char = CHARS[0];
const CHAR1: char = CHARS[1];
const CHAR2: char = CHARS[2];
const CHAR3: char = CHARS[3];

const CHAR4: char = CHARS[4];
const CHAR5: char = CHARS[5];

#[inline]
fn val(c: char) -> Result<u8, Error> {
    match c {
        CHAR0 => Ok(0b00),
        CHAR1 => Ok(0b01),
        CHAR2 => Ok(0b10),
        CHAR3 => Ok(0b11),

        _ => Err(Error::InvalidCharacter(c)),
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Block(u8);
impl Block {
    #[inline]
    pub fn to_chars(self) -> [char; 4] {
        [
            CHARS[(self.0 & 0b0000_0011) as usize],
            CHARS[((self.0 & 0b0000_1100) >> 2) as usize],
            CHARS[((self.0 & 0b0011_0000) >> 4) as usize],
            CHARS[((self.0 & 0b1100_0000) >> 6) as usize],
        ]
    }

    #[inline]
    pub fn from_chars(chars: [char; 4]) -> Result<Self, Error> {
        Ok(Self(
            val(chars[0])? | (val(chars[1])? << 2) | (val(chars[2])? << 4) | (val(chars[3])? << 6),
        ))
    }

    #[inline]
    pub fn to_compressed_chars(self, compression: Compression) -> ([char; 4], usize) {
        compression.block_to_chars(self)
    }

    #[inline]
    pub fn from_compressed_chars(
        chars: [char; 4],
        len: usize,
        compression: Compression,
    ) -> Result<Self, Error> {
        compression.block_from_chars(chars, len)
    }
}
impl From<u8> for Block {
    #[inline]
    fn from(b: u8) -> Self {
        Self(b)
    }
}
impl From<Block> for u8 {
    #[inline]
    fn from(b: Block) -> Self {
        b.0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Compression(u8);
impl Compression {
    #[inline]
    pub fn new(g0: [char; 2], g1: [char; 2]) -> Result<Self, Error> {
        Ok(Self(Block::from_chars([g0[0], g0[1], g1[0], g1[1]])?.0))
    }

    #[inline]
    fn g0l(self) -> u8 {
        self.0 & 0b0000_1111
    }
    #[inline]
    fn g1l(self) -> u8 {
        (self.0 & 0b1111_0000) >> 4
    }

    #[inline]
    fn g0h(self) -> u8 {
        (self.0 & 0b0000_1111) << 4
    }
    #[inline]
    fn g1h(self) -> u8 {
        self.0 & 0b1111_0000
    }

    #[inline]
    fn block_to_chars(self, b: Block) -> ([char; 4], usize) {
        let mut chars = ['\0'; 4];
        let mut len = 0;

        if (b.0 & 0b0000_1111) == self.g0l() {
            chars[len] = CHARS[4];
            len += 1;
        } else if (b.0 & 0b0000_1111) == self.g1l() {
            chars[len] = CHARS[5];
            len += 1;
        } else {
            chars[len] = CHARS[(b.0 & 0b0000_0011) as usize];
            chars[len + 1] = CHARS[((b.0 & 0b0000_1100) >> 2) as usize];
            len += 2;
        }

        if (b.0 & 0b1111_0000) == self.g0h() {
            chars[len] = CHARS[4];
            len += 1;
        } else if (b.0 & 0b1111_0000) == self.g1h() {
            chars[len] = CHARS[5];
            len += 1;
        } else {
            chars[len] = CHARS[((b.0 & 0b0011_0000) >> 4) as usize];
            chars[len + 1] = CHARS[((b.0 & 0b1100_0000) >> 6) as usize];
            len += 2;
        }

        (chars, len)
    }

    #[inline]
    fn block_from_chars(self, chars: [char; 4], len: usize) -> Result<Block, Error> {
        if len == 4 {
            Block::from_chars(chars)
        } else if len == 2 {
            Ok(Block(
                match chars[0] {
                    CHAR4 => self.g0l(),
                    CHAR5 => self.g1l(),
                    c => return Err(Error::InvalidCharacter(c)),
                } | match chars[1] {
                    CHAR4 => self.g0h(),
                    CHAR5 => self.g1h(),
                    c => return Err(Error::InvalidCharacter(c)),
                },
            ))
        } else {
            let mut b = 0;
            let mut shift = 0;
            for c in chars.iter().copied().take(3) {
                b |= match c {
                    CHAR4 => {
                        shift += 4;
                        self.g0l() << (shift - 4)
                    }
                    CHAR5 => {
                        shift += 4;
                        self.g1l() << (shift - 4)
                    }
                    c => {
                        shift += 2;
                        val(c)? << (shift - 2)
                    }
                };
            }
            Ok(Block(b))
        }
    }
}

pub fn encode<T: Iterator<Item = u8>>(iter: T) -> impl Iterator<Item = char> {
    EncodeIter {
        inner: iter,
        buffer: ['\0'; 4],
        cursor: 0,
    }
}

pub fn decode<T: Iterator<Item = char>>(iter: T) -> impl Iterator<Item = Result<u8, Error>> {
    DecodeIter { inner: iter }
}

pub fn encode_compress<T: Iterator<Item = u8>>(
    iter: T,
    compression: Compression,
) -> impl Iterator<Item = char> {
    EncodeCompressIter {
        inner: iter,
        buffer: ['\0'; 4],
        buffer_len: 0,
        compression,
        cursor: 0,
    }
}

pub fn decode_decompress<T: Iterator<Item = char>>(
    iter: T,
    compression: Compression,
) -> impl Iterator<Item = Result<u8, Error>> {
    DecodeDecompressIter {
        inner: iter,
        compression,
    }
}

pub fn is_zwc(c: char) -> bool {
    match c {
        CHAR0 | CHAR1 | CHAR2 | CHAR3 | CHAR4 | CHAR5 => true,
        _ => false,
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Error {
    InvalidCharacter(char),
    IncompleteBlock(usize),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter(c) => write!(f, "expected zero-width character but got {}", c),
            Self::IncompleteBlock(len) => {
                write!(f, "expected more than {} characters in block", len)
            }
        }
    }
}
#[cfg(feature = "std")]
impl std::error::Error for crate::Error {}

/// Encoding iterator
struct EncodeIter<T: Iterator<Item = u8>> {
    inner: T,
    buffer: [char; 4],
    cursor: usize,
}
impl<T: Iterator<Item = u8>> Iterator for EncodeIter<T> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor == 0 {
            self.buffer = match self.inner.next() {
                Some(b) => Block::from(b).to_chars(),
                None => return None,
            };
        }

        let ret = Some(self.buffer[self.cursor]);
        self.cursor = (self.cursor + 1) % 4;
        ret
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner_size_hint = self.inner.size_hint();
        (
            (inner_size_hint.0 * 8) + (3 - self.cursor),
            inner_size_hint.1.map(|b| (b * 8) + (3 - self.cursor)),
        )
    }
}

/// Decoding iterator
struct DecodeIter<T: Iterator<Item = char>> {
    inner: T,
}
impl<T: Iterator<Item = char>> Iterator for DecodeIter<T> {
    type Item = Result<u8, Error>;

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
        let c1 = match_next!(self.inner, Some(Err(Error::IncompleteBlock(1))));
        let c2 = match_next!(self.inner, Some(Err(Error::IncompleteBlock(2))));
        let c3 = match_next!(self.inner, Some(Err(Error::IncompleteBlock(3))));
        Some(Block::from_chars([c0, c1, c2, c3]).map(Into::into))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner_size_hint = self.inner.size_hint();
        (inner_size_hint.0 / 8, inner_size_hint.1.map(|b| b / 8))
    }
}

/// Encoding and compressing iterator
struct EncodeCompressIter<T: Iterator<Item = u8>> {
    inner: T,
    compression: Compression,
    buffer: [char; 4],
    buffer_len: usize,
    cursor: usize,
}
impl<T: Iterator<Item = u8>> Iterator for EncodeCompressIter<T> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor == 0 {
            let nb = match self.inner.next() {
                Some(b) => Block::from(b).to_compressed_chars(self.compression),
                None => return None,
            };
            self.buffer = nb.0;
            self.buffer_len = nb.1;
        }

        let ret = Some(self.buffer[self.cursor]);
        self.cursor = (self.cursor + 1) % self.buffer_len;
        ret
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner_size_hint = self.inner.size_hint();
        (
            (inner_size_hint.0 * 4) + (self.buffer_len - 1 - self.cursor),
            inner_size_hint
                .1
                .map(|b| (b * 8) + (self.buffer_len - 1 - self.cursor)),
        )
    }
}

/// Decoding and decompressing iterator
struct DecodeDecompressIter<T: Iterator<Item = char>> {
    inner: T,
    compression: Compression,
}
impl<T: Iterator<Item = char>> Iterator for DecodeDecompressIter<T> {
    type Item = Result<u8, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! match_next {
            ($iter:expr, $err:expr) => {
                match $iter.next() {
                    Some(CHAR4) => (CHAR4, 2),
                    Some(CHAR5) => (CHAR5, 2),
                    Some(c) => (c, 1),
                    None => return $err,
                }
            };
        }

        let mut chars = ['\0'; 4];

        chars[0] = self.inner.next()?;

        let mut len = 1;
        let mut ceil = match chars[0] {
            CHAR4 | CHAR5 => 2,
            _ => 1,
        };

        while ceil < 4 {
            let ni = match_next!(self.inner, Some(Err(Error::IncompleteBlock(len))));
            chars[len] = ni.0;
            len += 1;
            ceil += ni.1;
        }

        Some(Block::from_compressed_chars(chars, len, self.compression).map(Into::into))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner_size_hint = self.inner.size_hint();
        (inner_size_hint.0 / 4, inner_size_hint.1.map(|b| b / 8))
    }
}

#[cfg(feature = "camouflage")]
mod camouflage {
    use std::fmt;

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
        let mut encoded_payload = crate::encode(compressed_payload.iter().copied());

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

    pub fn decamouflage(camouflaged: &str, key: Option<&str>) -> Result<Vec<u8>, CamouflageError> {
        use brotli::BrotliDecompress;
        use chacha20poly1305::aead::Aead;
        use generic_array::GenericArray;

        let encoded_payload = camouflaged.chars().filter(|c| crate::is_zwc(*c));
        let compressed_payload = crate::decode(encoded_payload).collect::<Result<Vec<u8>, _>>()?;

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
        ZwcDecode(crate::Error),
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
    impl From<crate::Error> for CamouflageError {
        fn from(e: crate::Error) -> Self {
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
        let encoded = super::encode(SRC.iter().copied());
        let decoded = super::decode(encoded);

        for (ob, db) in SRC.iter().copied().zip(decoded) {
            assert_eq!(ob, db.unwrap());
        }
    }

    #[test]
    fn compression_round_trip() {
        let compression = super::Compression::new(
            [super::CHARS[0], super::CHARS[1]],
            [super::CHARS[2], super::CHARS[3]],
        )
        .unwrap();
        let encoded = super::encode_compress(SRC.iter().copied(), compression);
        let decoded = super::decode_decompress(encoded, compression);

        for (ob, db) in SRC.iter().copied().zip(decoded) {
            assert_eq!(ob, db.unwrap());
        }
    }
}
