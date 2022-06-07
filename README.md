[![Docs](https://docs.rs/salzweg/badge.svg)](https://docs.rs/salzweg)
[![Crates.io](https://img.shields.io/crates/d/salzweg.svg)](https://crates.io/crates/salzweg)
[![Crates.io](https://img.shields.io/crates/v/salzweg.svg)](https://crates.io/crates/salzweg)

# salzweg

Salzweg is a LZW encoder and decoder, implementing the GIF variation with code sizes between 2 and 8.

LZW is a universal lossless data [compression algorithm](https://en.wikipedia.org/wiki/Lempel%E2%80%93Ziv%E2%80%93Welch).

# Crates

A few crates here:

* [lzw](lzw) - the actual library, implementing the LZW compression algo.
* [exploration](exploration) - just trying different way to compress LZW, and comparing how efficient they are.
* [memory-profiling](memory-profiling) - separate crate to check for memory usage using [dhat-rs](https://crates.io/crates/dhat).
# License

Code is licensed under MIT.
