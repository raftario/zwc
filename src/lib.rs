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

/// Returns the binary value of a zero-width character
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

/// Represents a 2-4 zero-width character block
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Block(u8);
impl Block {
    /// Converts a block to chars without compression
    #[inline]
    pub fn to_chars(self) -> [char; 4] {
        [
            CHARS[(self.0 & 0b0000_0011) as usize],
            CHARS[((self.0 & 0b0000_1100) >> 2) as usize],
            CHARS[((self.0 & 0b0011_0000) >> 4) as usize],
            CHARS[((self.0 & 0b1100_0000) >> 6) as usize],
        ]
    }

    /// Creates a block from chars without compression
    #[inline]
    pub fn from_chars(chars: [char; 4]) -> Result<Self, Error> {
        Ok(Self(
            val(chars[0])? | (val(chars[1])? << 2) | (val(chars[2])? << 4) | (val(chars[3])? << 6),
        ))
    }

    /// Converts a block to chars with compression
    #[inline]
    pub fn to_compressed_chars(self, compression: Compression) -> ([char; 4], usize) {
        compression.block_to_chars(self)
    }

    /// Creates a block from chars with compression
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

/// Represents 4 bits patterns to compress into a single character instead of two
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Compression(u8);
impl Compression {
    /// Creates a compression setting from two 4 bits patterns
    pub fn new(p0: u8, p1: u8) -> Result<Self, Error> {
        if p0 >= 16 {
            return Err(Error::InvalidCompressionPattern(p0));
        }
        if p1 >= 16 {
            return Err(Error::InvalidCompressionPattern(p1));
        }
        Ok(Self(p0 | (p1 << 4)))
    }

    /// Creates a compression setting by finding the two most common patterns in the provided data
    pub fn optimal(data: &[u8]) -> (Self, u8, u8) {
        let mut patterns: [usize; 16] = [0; 16];
        for b in data.iter().copied() {
            patterns[(b & 0b0000_1111) as usize] += 1;
            patterns[((b & 0b1111_0000) >> 4) as usize] += 1;
        }

        let recurrent_patterns = patterns.iter().enumerate().fold((0, 0), |acc, (i, o)| {
            if *o > patterns[acc.0] {
                (i, acc.1)
            } else if *o > patterns[acc.1] {
                (acc.0, i)
            } else {
                acc
            }
        });
        let rp0 = recurrent_patterns.0 as u8;
        let rp1 = recurrent_patterns.1 as u8;

        (Self::new(rp0, rp1).unwrap(), rp0, rp1)
    }

    /// Returns the first pattern as lower bytes
    #[inline]
    fn g0l(self) -> u8 {
        self.0 & 0b0000_1111
    }
    /// Returns the second pattern as lower bytes
    #[inline]
    fn g1l(self) -> u8 {
        (self.0 & 0b1111_0000) >> 4
    }

    /// Returns the first pattern as higher bytes
    #[inline]
    fn g0h(self) -> u8 {
        (self.0 & 0b0000_1111) << 4
    }
    /// Returns the second pattern as higher bytes
    #[inline]
    fn g1h(self) -> u8 {
        self.0 & 0b1111_0000
    }

    /// Converts a block to chars
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

    /// Creates a block from chars
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

/// Converts a byte iterator into a zero-width character iterator
pub fn encode<T: Iterator<Item = u8>>(iter: T) -> impl Iterator<Item = char> {
    EncodeIter {
        inner: iter,
        buffer: ['\0'; 4],
        cursor: 0,
    }
}

/// Converts a zero-width character iterator into a byte iterator
pub fn decode<T: Iterator<Item = char>>(iter: T) -> impl Iterator<Item = Result<u8, Error>> {
    DecodeIter { inner: iter }
}

/// Converts a byte iterator into a zero-width character iterator compressed using the provided settings
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

/// Converts a zero-width character iterator into a byte iterator decompressed using the provided settings
pub fn decode_decompress<T: Iterator<Item = char>>(
    iter: T,
    compression: Compression,
) -> impl Iterator<Item = Result<u8, Error>> {
    DecodeDecompressIter {
        inner: iter,
        compression,
    }
}

/// Check if a character is zero-width
pub fn is_zw(c: char) -> bool {
    match c {
        CHAR0 | CHAR1 | CHAR2 | CHAR3 | CHAR4 | CHAR5 => true,
        _ => false,
    }
}

/// Represents an error that might occur while dealing with zero-width character iterators
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Error {
    /// Occurs when trying to decode a non-zero-width character
    InvalidCharacter(char),
    /// Occurs when trying to decode an incomplete zero-width character block into a byte
    IncompleteBlock(usize),
    /// Occurs when trying to use a pattern larger than 4 bits for compression
    InvalidCompressionPattern(u8),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter(c) => write!(f, "expected zero-width character but got {}", c),
            Self::IncompleteBlock(len) => {
                write!(f, "expected more than {} characters in block", len)
            }
            Self::InvalidCompressionPattern(p) => {
                write!(f, "expected a 4 bits value but got {:08b}", p)
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

    /// Hides a compressed and optionally encrypted payload inside a string
    pub fn camouflage(
        payload: Vec<u8>,
        dummy: &str,
        key: Option<&str>,
        compression_level: Option<i32>,
    ) -> Result<String, Error> {
        use brotli::enc::BrotliEncoderParams;
        use chacha20::ChaCha20Rng;
        use chacha20poly1305::aead::Aead;
        use generic_array::GenericArray;
        use rand_core::{RngCore, SeedableRng};

        let mut compressed_payload = Vec::with_capacity(payload.len());
        brotli::BrotliCompress(
            &mut payload.as_slice(),
            &mut compressed_payload,
            &BrotliEncoderParams {
                quality: compression_level.unwrap_or(10),
                size_hint: payload.len(),
                ..Default::default()
            },
        )?;

        if let Some(k) = key {
            let cipher = get_cipher(k);

            let mut nonce = [0; 12];
            ChaCha20Rng::from_entropy().fill_bytes(&mut nonce);

            cipher.encrypt_in_place(
                GenericArray::from_slice(&nonce),
                b"",
                &mut compressed_payload,
            )?;

            compressed_payload.extend_from_slice(&nonce);
        }

        let (compression, rp0, rp1) = crate::Compression::optimal(&compressed_payload);

        let mut encoded_payload =
            crate::encode_compress(compressed_payload.iter().copied(), compression);

        let mut camouflaged = String::with_capacity((compressed_payload.len() * 8) + dummy.len());
        for c in dummy.chars() {
            camouflaged.push(c);
            if c == ' ' {
                for c in &mut encoded_payload {
                    camouflaged.push(c);
                }
                camouflaged.push(crate::CHARS[(rp0 & 0b0011) as usize]);
                camouflaged.push(crate::CHARS[((rp0 & 0b1100) >> 2) as usize]);
                camouflaged.push(crate::CHARS[(rp1 & 0b0011) as usize]);
                camouflaged.push(crate::CHARS[((rp1 & 0b1100) >> 2) as usize]);
            }
        }
        if encoded_payload.next().is_some() {
            return Err(Error::NoSpaces);
        }

        Ok(camouflaged)
    }

    /// Retrieves a compressed and optionally encrypted payload from a string
    pub fn decamouflage(camouflaged: &str, key: Option<&str>) -> Result<Vec<u8>, Error> {
        use brotli::BrotliDecompress;
        use chacha20poly1305::aead::Aead;
        use generic_array::GenericArray;

        let mut encoded_payload: Vec<char> =
            camouflaged.chars().filter(|c| crate::is_zw(*c)).collect();

        let c3 = encoded_payload.pop().ok_or(Error::InvalidPayload)?;
        let c2 = encoded_payload.pop().ok_or(Error::InvalidPayload)?;
        let c1 = encoded_payload.pop().ok_or(Error::InvalidPayload)?;
        let c0 = encoded_payload.pop().ok_or(Error::InvalidPayload)?;
        let compression = crate::Compression::new(
            crate::val(c0)? | (crate::val(c1)? << 2),
            crate::val(c2)? | (crate::val(c3)? << 2),
        )?;

        let mut compressed_payload =
            crate::decode_decompress(encoded_payload.into_iter(), compression)
                .collect::<Result<Vec<u8>, _>>()?;

        if let Some(k) = key {
            let cipher = get_cipher(k);

            let nonce_boundary = compressed_payload.len() - 12;
            let nonce = GenericArray::clone_from_slice(&compressed_payload[nonce_boundary..]);
            compressed_payload.truncate(nonce_boundary);

            cipher.decrypt_in_place(&nonce, b"", &mut compressed_payload)?;
        }

        let mut payload = Vec::with_capacity(compressed_payload.len() * 4);
        BrotliDecompress(&mut compressed_payload.as_slice(), &mut payload)?;

        Ok(payload)
    }

    /// Represents an error that might occur while hiding or retrieving a payload
    #[derive(Debug)]
    pub enum Error {
        Zwc(crate::Error),
        Cipher(chacha20poly1305::aead::Error),
        Brotli(std::io::Error),
        NoSpaces,
        InvalidPayload,
    }
    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Zwc(e) => write!(f, "zero-width character decoding error: {}", e),
                Self::Cipher(e) => write!(f, "cipher error: {:?}", e),
                Self::Brotli(e) => write!(f, "brotli error: {}", e),
                Self::NoSpaces => write!(f, "no spaces in dummy string"),
                Self::InvalidPayload => write!(f, "the payload is invalid"),
            }
        }
    }
    impl std::error::Error for Error {}
    impl From<crate::Error> for Error {
        fn from(e: crate::Error) -> Self {
            Self::Zwc(e)
        }
    }
    impl From<chacha20poly1305::aead::Error> for Error {
        fn from(e: chacha20poly1305::aead::Error) -> Self {
            Self::Cipher(e)
        }
    }
    impl From<std::io::Error> for Error {
        fn from(e: std::io::Error) -> Self {
            Self::Brotli(e)
        }
    }

    /// Generates a cipher instance from a key
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
}

#[cfg(test)]
mod tests {
    static SRC: &[u8] = include_bytes!("./lib.rs");

    #[test]
    fn round_trip() {
        let encoded = crate::encode(SRC.iter().copied());
        let decoded = crate::decode(encoded);

        for (ob, db) in SRC.iter().copied().zip(decoded) {
            assert_eq!(ob, db.unwrap());
        }
    }

    #[test]
    fn compression_round_trip() {
        let compression = crate::Compression::new(0b0000, 0b1111).unwrap();
        let encoded = crate::encode_compress(SRC.iter().copied(), compression);
        let decoded = crate::decode_decompress(encoded, compression);

        for (ob, db) in SRC.iter().copied().zip(decoded) {
            assert_eq!(ob, db.unwrap());
        }
    }

    #[cfg(feature = "camouflage")]
    #[test]
    fn camouflage_round_trip() {
        let dummy = "Hello, World!";

        let camouflaged = crate::camouflage(SRC.to_vec(), dummy, Some("secret"), None).unwrap();
        let decamouflaged = crate::decamouflage(&camouflaged, Some("secret")).unwrap();

        assert_eq!(SRC, decamouflaged.as_slice());
    }
}
