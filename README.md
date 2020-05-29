# zwc

Rust library for encoding binary data using zero-width characters. Also optionally includes helpers for compressing, encrypting and hiding data inside strings, and a CLI wrapping that functionality.

## Why

[StegCloak](https://github.com/KuroLabs/stegcloak) popped on my GitHub dashboard feed and I thought it was pretty cool, and decided to reimplement it myself for fun.

## How it works

The basic encoding feature just converts each byte to anywhere between two and four zero-width unicode characters (depending on which bit patterns are used for compression), and vice-versa for decoding. Data is optionally compressed using [Brotli](https://www.ietf.org/rfc/rfc7932.txt) and encrypted using [ChaCha20-Poly1305](https://tools.ietf.org/rfc/rfc7539.txt).

## Performance

On an i7-7700HQ @ 2.80GHz, roundtrip throughput for the [sample text data](./samples/lorem.txt) was around 15 MiB/s for basic encoded and compressed data, and around 280 KiB/s for basic encoded and compressed, encrypted, quality 10 Brotli compressed data. You can run the benchmarks yourself for full results using `cargo bench`.

## `no_std` support

The core encoding and decoding iterators do not do any heap allocation and support `no_std`. Extra helpers that require `std` are included by default but can be discarded by setting `default-features` to `fakse`.
