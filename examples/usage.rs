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
    let name = r.read_snake().unwrap(); // byte_reader
    r.consume("!").unwrap();

    println!("Greeted to `{name}`.");
}
