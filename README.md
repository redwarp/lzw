[![Docs](https://docs.rs/salzweg/badge.svg)](https://docs.rs/salzweg)
[![Crates.io](https://img.shields.io/crates/d/salzweg.svg)](https://crates.io/crates/salzweg)
[![Crates.io](https://img.shields.io/crates/v/salzweg.svg)](https://crates.io/crates/salzweg)

# salzweg

Salzweg is a LZW encoder and decoder. It supports the GIF flavored, TIFF flavored and fixed code flavors of LZW.

LZW is a universal lossless data [compression algorithm](https://en.wikipedia.org/wiki/Lempel%E2%80%93Ziv%E2%80%93Welch).

The aim of this library is to be memory efficient, and fast. 
* The decoder lives only on the stack, and will be friendly with machines with low memory.
* The encoder builds on the heap though, as it creates a growing tree of possible encoded words as the compression progresses.
# Sources
* This link definitly helped me understand LZW through and through: https://www.eecis.udel.edu/~amer/CISC651/lzw.and.gif.explained.html
* This [rust example](https://rosettacode.org/wiki/LZW_compression#Rust) was a good starting point for implementing the compression, though this solution was totally abandonned later on.
* [Arena-Allocated Trees in Rust](https://dev.to/deciduously/no-more-tears-no-more-knots-arena-allocated-trees-in-rust-44k6) as I used something like that in the encoder.
# License

Code is licensed under MIT.
