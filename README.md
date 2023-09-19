# byte reader
A **minimum** byte-by-byte reader for parsing input.

## Usage
```rust
use byte_reader::Reader;

fn main() {
    // Get a `Vec<u8>` input from
    // a File, stanard input, or something
    let sample_input = format!("Hello, byte_reader!").to_vec();

    // Create mutable `r`
    let mut r = Reader::new(sample_input);

    // Use some simple operations
    // to parse the input
    r.parse_keyword("Hello").unwrap();
    r.consume(1);
    r.skip_whitespace();
    let name = r.pop_ident().unwrap();  // byte_reader
    let name_starts_at = r.column();    // 8
    r.consume(1);

    println!("Greeted to {name}.");
    println!("The name starts at column {name_start_at} on line 1.");
}
```
