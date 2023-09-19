<h1 align="center">byte reader</h1>
<p align="center">A <strong>minimum</strong> byte-by-byte reader for parsing input.</p>

<div align="right">
    <img alt="build check status of byte_reader" src="https://github.com/kana-rus/byte_reader/actions/workflows/check.yml/badge.svg"/>
    <img alt="test status of byte_reader" src="https://github.com/kana-rus/byte_reader/actions/workflows/test.yml/badge.svg"/>
</div>

<h2><a href="https://github.com/kana-rus/byte_reader/tree/main/src/examples/usage.rs">Usage</a></h2>

```rust
use byte_reader::Reader;

fn main() {
    // Get a `&[u8]` or `Vec<u8>` input from
    // a File, standard input, or something
    let sample_input = "Hello, byte_reader!".as_bytes();

    // Create mutable `r`
    let mut r = Reader::new(sample_input);

    // Use some simple operations
    // to parse the input
    r.consume("Hello").unwrap();
    r.consume(",").unwrap();
    r.skip_whitespace();
    let name_line   = r.line();         // 1
    let name_column = r.column();       // 8
    let name = r.read_snake().unwrap(); // byte_reader
    r.consume("!").unwrap();

    println!("Greeted to `{name}`.");
    println!("The name starts at column {name_column} on line {name_line}.");
}

```
