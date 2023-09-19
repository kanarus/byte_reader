use crate::Reader;
use std::format as f;

#[test] fn test_consume() {
    let mut r = Reader::new("Hello, world!".as_bytes());

    r.advance_by(1);
    assert_eq!(r.remained(), b"ello, world!");

    r.advance_by(3);
    assert_eq!(r.remained(), b"o, world!");
}

#[test] fn test_parse_ident() {
    let mut r = Reader::new(f!(
        "Hello, world! I am a Reader!"
    ).into_bytes());

    let ident = r.read_snake().unwrap();
    assert_eq!(ident, "Hello");
    assert_eq!(r.remained(), b", world! I am a Reader!");

    assert!(r.read_snake().is_err());
    r.advance_by(1);
    assert!(r.read_snake().is_err());
    r.advance_by(1);

    let ident = r.read_snake().unwrap();
    assert_eq!(ident, "world");
    assert_eq!(r.remained(), b"! I am a Reader!")
}

#[test] fn test_parse_string_literal() {
    let mut r = Reader::new(f!("\
        \"Hello,\" He said, \"I am Reader!\"\
    ").into_bytes());

    let lit = r.read_string().unwrap();
    assert_eq!(lit, "Hello,");
    assert_eq!(r.remained(), b" He said, \"I am Reader!\"");

    assert!(r.read_string().is_err());
    r.skip_whitespace();
    assert_eq!(r.read_snake().unwrap(), "He");
    r.skip_whitespace();
    assert_eq!(r.read_snake().unwrap(), "said");
    assert_eq!(r.peek().unwrap(), &b',');
    r.advance_by(1);
    r.skip_whitespace();

    let lit = r.read_string().unwrap();
    assert_eq!(lit, "I am Reader!");
    assert_eq!(r.remained(), b"");
}

#[test] fn test_parse_int() {
    let mut r = Reader::new("\
        model Post {\n\
          title     String @db.VarChar(200)\n\
          n_authors Int    @default(1)\n\
          z_flag    Int    @default(-42)\n\
        }\
    ".to_string().into_bytes());

    assert!(r.consume("model").is_ok());
    r.skip_whitespace();
    assert_eq!(r.read_snake().unwrap(), "Post");
    r.skip_whitespace();
    assert_eq!(r.peek().unwrap(), &b'{'); r.advance_by(1);
    r.skip_whitespace();
    assert_eq!(r.read_snake().unwrap(), "title");
    r.skip_whitespace();
    assert_eq!(r.read_snake().unwrap(), "String");
    r.skip_whitespace();
    assert_eq!(r.peek().unwrap(), &b'@'); r.advance_by(1);
    assert_eq!(r.read_snake().unwrap(), "db");
    assert!(r.consume(".").is_ok());
    assert_eq!(r.read_snake().unwrap(), "VarChar");
    assert_eq!(r.peek().unwrap(), &b'('); r.advance_by(1);

    let int = r.read_unsigned_int().unwrap();
    assert_eq!(int, 200);
    assert_eq!(r.peek().unwrap(), &b')'); r.advance_by(1);

    r.skip_whitespace();
    assert_eq!(r.read_snake().unwrap(), "n_authors");
    r.skip_whitespace();
    assert_eq!(r.read_snake().unwrap(), "Int");
    r.skip_whitespace();
    assert_eq!(r.peek().unwrap(), &b'@'); r.advance_by(1);
    assert_eq!(r.read_snake().unwrap(), "default");
    assert_eq!(r.peek().unwrap(), &b'('); r.advance_by(1);

    let int = r.read_int().unwrap();
    assert_eq!(int, 1);
    assert_eq!(r.peek().unwrap(), &b')'); r.advance_by(1);

    r.skip_whitespace();
    assert_eq!(r.read_snake().unwrap(), "z_flag");
    r.skip_whitespace();
    assert_eq!(r.read_snake().unwrap(), "Int");
    r.skip_whitespace();
    assert_eq!(r.peek().unwrap(), &b'@'); r.advance_by(1);
    assert_eq!(r.read_snake().unwrap(), "default");
    assert_eq!(r.peek().unwrap(), &b'('); r.advance_by(1);

    let int = r.read_int().unwrap();
    assert_eq!(int, -42);
    assert_eq!(r.peek().unwrap(), &b')'); r.advance_by(1);

    r.skip_whitespace();
    assert_eq!(r.peek().unwrap(), &b'}'); r.advance_by(1);
    assert_eq!(r.peek(), None)
}
