[![Docs](https://docs.rs/salzweg/badge.svg)](https://docs.rs/salzweg)
[![Crates.io](https://img.shields.io/crates/d/salzweg.svg)](https://crates.io/crates/salzweg)
[![Crates.io](https://img.shields.io/crates/v/salzweg.svg)](https://crates.io/crates/salzweg)

# salzweg

Salzweg is a LZW encoder and decoder. It supports the GIF flavored, TIFF flavored and fixed code flavors of LZW.

LZW is a universal lossless data [compression algorithm](https://en.wikipedia.org/wiki/Lempel%E2%80%93Ziv%E2%80%93Welch).

The aim of this library is to be memory efficient, and fast. 
* The decoder lives only on the stack, and will be friendly with machines with low memory.
* The encoder builds on the heap though, as it creates a growing tree of possible encoded words as the compression progresses.
# Speed

First, a few formulas

 * Compressing speed  = uncompressed bytes/seconds to compress.
 * Decompressing speed  = uncompressed bytes/seconds to decompress.

## Results

Using criterion on a `AMD Ryzen 7 2700X Eight-Core Processor 3.70 GHz CPU` , I observed the following throughput when processing data:

|                                | Variable encoder | Fix 12 bit size |
|--------------------------------|------------------|-----------------|
| Compressing image data         | 70 MiB/s         | 120 MiB/s       |
| Decompressing image data       | 200 MiB/s        | 210 MiB/s       |
| Compressing lorem ipsum text   | 70 MiB/s         | 85 MiB/s        |
| Decompressing lorem ipsum text | 200 MiB/s        | 220 MiB/s       |

These timings are rounded, indicative more than 100% accurate. But they are consistently faster than the [LZW](https://crates.io/crates/lzw) and [Weezl](https://crates.io/crates/weezl) crate for encoding, and consistently faster than the [Weezl](https://crates.io/crates/weezl) crate for decoding (I did not try to decode with LZW, as the comparison is difficult due to API design).

# Sources
* This link definitly helped me understand LZW through and through: https://www.eecis.udel.edu/~amer/CISC651/lzw.and.gif.explained.html
* This [rust example](https://rosettacode.org/wiki/LZW_compression#Rust) was a good starting point for implementing the compression, though this solution was totally abandonned later on.
* [Arena-Allocated Trees in Rust](https://dev.to/deciduously/no-more-tears-no-more-knots-arena-allocated-trees-in-rust-44k6) as I used something like that in the encoder.
# License

Code is licensed under MIT.
