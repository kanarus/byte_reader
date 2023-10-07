use byte_reader::Reader;

#[cfg(not(feature="location"))] fn main() {
    // Get a input from a File, standard input, or others
    // Input must implement `AsRef<[u8]>`
    let sample_input = "Hello,    byte_reader!";

    // Create mutable `r`
    let mut r = Reader::new(sample_input);

    // Use some simple operations
    // to parse the input
    r.consume("Hello").unwrap();
    r.consume(",").unwrap();
    r.skip_whitespace();
    let name = r.read_snake().unwrap(); // byte_reader
    r.consume("!").unwrap();

    println!("Greeted to `{name}`.");
}

#[cfg(feature="location")] fn main() {
    let mut r = Reader::new("Hello,    byte_reader!");

    r.consume("Hello").unwrap();
    r.consume(",").unwrap();
    r.skip_whitespace();
    let name_line   = r.line;   // 1
    let name_column = r.column; // 11
    let name_index  = r.index;  // 10
    let name = r.read_snake().unwrap(); // byte_reader
    r.consume("!").unwrap();

    println!("Greeted to `{name}`.");
    println!("In the input, the name starts at column {name_column} of line {name_line} (index: {name_index})");
}
