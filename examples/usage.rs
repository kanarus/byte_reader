use byte_reader::Reader;

fn main() {
    // Get a input from a File, standard input, or others
    // Input must implement `AsRef<[u8]>`
    let sample_input = "Hello,    byte_reader!";

    // Create mutable `r`
    let mut r = Reader::from(sample_input);

    // Use some simple operations
    // to parse the input
    r.consume("Hello").unwrap();
    r.consume(",").unwrap();
    r.skip_whitespace();
    let name = r.read_snake().unwrap(); // byte_reader
    r.consume("!").unwrap();

    println!("Greeted to `{name}`.");
}
