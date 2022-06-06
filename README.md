# salzweg

Salzweg is a LZW encoder and decoder, implementing the GIF variation with code sizes between 2 and 8.

# Crates

A few crates here:
* [lzw](lzw) - the actual library, implementing the LZW compression algo.
* [exploration](exploration) - just trying different way to implement LZW, and comparing how efficient they are.
* [memory-profiling](memory-profiling) - separate crate to check for memory usage using [dhat-rs](https://crates.io/crates/dhat).
