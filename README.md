<h1 align="center">byte reader</h1>

A **minimum** byte-by-byte reader for parsing input.

<div align="right">
    <img alt="build check status of byte_reader" src="https://github.com/kana-rus/byte_reader/actions/workflows/check.yml/badge.svg"/>
    <img alt="test status of byte_reader" src="https://github.com/kana-rus/byte_reader/actions/workflows/test.yml/badge.svg"/>
</div>


## Usage
```rust
use byte_reader::Reader;

fn main() {
    // Get a `&[u8]` or `Vec<u8>` input from
    // a File, stanard input, or something
    let sample_input = b"Hello, byte_reader!";

    // Create mutable `r`
    let mut r = Reader::borrowed(sample_input);

    // Use some simple operations
    // to parse the input
    r.consume("Hello").unwrap();
    r.consume(",");
    r.skip_whitespace();
    let name = r.read_snake().unwrap(); // byte_reader
    let name_starts_at = r.column();    // 8
    r.consume("!");

    println!("Greeted to {name}.");
    println!("The name starts at column {name_start_at} on line 1.");
}
```
