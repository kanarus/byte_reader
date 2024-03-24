<h1 align="center">byte_reader</h1>
<p align="center">A <strong>minimal</strong> byte-by-byte reader for parsing input.</p>

<div align="right">
    <img alt="test status of byte_reader" src="https://github.com/kana-rus/byte_reader/actions/workflows/test.yml/badge.svg"/>
    <a href="https://crates.io/crates/byte_reader"><img alt="crates.io" src="https://img.shields.io/crates/v/byte_reader" /></a>
</div>

## Use case
Following situation:

> I want to read and parse some input, but it's **not so large-scale** parsing task, so I'd like to avoid adding a *heavyweight* crate like [nom](https://crates.io/crates/nom) or [nom8](https://crates.io/crates/nom8) to my `dependencies` ...

Of course, `byte_reader` supports *no std* environment.

<br/>

<h2><a href="https://github.com/kana-rus/byte_reader/blob/main/examples/usage.rs">Usage</a></h2>

```rust
use byte_reader::Reader;

fn main() {
    // Get an input `&[u8]` from a File, standard input, or others
    let sample_input = "Hello,    byte_reader!".as_bytes();

    // Create mutable `r` for the input
    let mut r = Reader::new(sample_input);

    // Use some simple operations
    // to parse the input
    r.consume("Hello").unwrap();
    r.consume(",").unwrap();
    r.skip_whitespace();
    let name = r.read_while(|b| b != &b'!'); // b"byte_reader"
    let name = String::from_utf8_lossy(name).to_string();
    r.consume("!").unwrap();

    println!("Greeted to `{name}`.");
}
```

<br/>

## Operations
- `remaining`
- `read_while`, `read_until`
- `next`, `next_if`
- `peek`, `peek2`, `peek3`
- `advance_by`, `unwind_by`
- `consume`, `consume_oneof`
- `skip_while`, `skip_whitespace`

<br/>

## Features

### `"location"`

You can track the reader's parsing location ( **line** and **column** ) in the input bytes.

```rust
/* activate "location" feature */
use byte_reader::Reader;

fn main() {
    let mut r = Reader::new("Hello,    byte_reader!");

    r.consume("Hello").unwrap();
    r.consume(",").unwrap();
    r.skip_whitespace();
    let name_line   = r.line;   // 1
    let name_column = r.column; // 11
    let name_index  = r.index;  // 10
    let name = r.read_snake().unwrap(); // "byte_reader"
    r.consume("!").unwrap();

    println!("Greeted to `{name}`.");
    println!("In the input, the name starts at column {name_column} of line {name_line} (index: {index})");
}
```

### `"text"`

Some utility methods for text-parsing are available：

  - `read_quoted_by`
  - `read_uint`, `read_int`
  - `read_camel`, `read_snake`, `read_kebab`

### `"detahced"` (**unsafe**)

Make read bytes *detahced* in `read_*` operations. This resolves issue around borrowing `Reader`.

<br/>

## License
`byte_reader` is licensed under the MIT License ([LICENSE](https://github.com/kana-rus/byte_reader/blob/main/LICENSE) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).