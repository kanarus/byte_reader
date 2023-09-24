use crate::Reader;
use std::format as f;

#[test] fn test_whitespace() {
    let mut r = Reader::new(" ".as_bytes());
    r.skip_whitespace();
    assert!(r.remained().is_empty());

    let mut r = Reader::new("  a".as_bytes());
    r.skip_whitespace();
    assert_eq!(r.remained(), b"a");
}

#[test] fn test_advance() {
    let mut r = Reader::new("Hello, world!".as_bytes());

    r.advance_by(1);
    assert_eq!(r.remained(), b"ello, world!");

    r.advance_by(3);
    assert_eq!(r.remained(), b"o, world!");
}

#[test] fn test_rewind() {
    let mut r = Reader::new("Hello, world!".as_bytes());
    r.advance_by(1);
    assert_eq!(r.remained(), b"ello, world!");
    r.advance_by(3);
    assert_eq!(r.remained(), b"o, world!");

    #[cfg(not(feature="location"))] {
        r.rewind_by(3);
        assert_eq!(r.remained(), b"ello, world!");
        r.rewind_by(1);
        assert_eq!(r.remained(), b"Hello, world!");
    }

    let mut r = Reader::new("Hello, world!\nMy name is byte_reader!".as_bytes());
    r.read_while(|b| b != &b'\n');
    assert_eq!(r.remained(), b"\nMy name is byte_reader!");
    #[cfg(feature="location")] assert_eq!(r.line(),   1);
    #[cfg(feature="location")] assert_eq!(r.column(), 14);
    r.advance_by(3);
    assert_eq!(r.remained(), b" name is byte_reader!");
    #[cfg(feature="location")] assert_eq!(r.line(),   2);
    #[cfg(feature="location")] assert_eq!(r.column(), 3);

    #[cfg(not(feature="location"))] {
        r.rewind_by(2);
        assert_eq!(r.remained(), b"My name is byte_reader!");
        #[cfg(feature="location")] assert_eq!(r.line(),   2);
        #[cfg(feature="location")] assert_eq!(r.column(), 1);
        #[cfg(not(feature="location"))] r.rewind_by(2);
        assert_eq!(r.remained(), b"!\nMy name is byte_reader!");
        #[cfg(feature="location")] assert_eq!(r.line(),   1);
        #[cfg(feature="location")] assert_eq!(r.column(), 13);
    }
}

#[test] fn test_read_while() {
    let mut r = Reader::new("Hello, world!".as_bytes());
    assert_eq!(
        r.read_while(|b| ! b.is_ascii_whitespace()),
        b"Hello,"
    );
    assert_eq!(r.remained(), b" world!");
}

#[test] fn test_parse_ident() {
    let mut r = Reader::new(f!(
        "Hello, world! I am a Reader!"
    ).into_bytes());

    let ident = r.read_snake().unwrap();
    assert_eq!(ident, "Hello");
    assert_eq!(r.remained(), b", world! I am a Reader!");

    assert!(r.read_snake().is_none());
    r.advance_by(1);
    assert!(r.read_snake().is_none());
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

    assert!(r.read_string().is_none());
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
    let mut r = Reader::new("42".as_bytes());
    assert_eq!(r.read_int(), Some(42));
    assert!(r.remained().is_empty());

    let mut r = Reader::new("-42".as_bytes());
    assert_eq!(r.read_int(), Some(-42));
    assert!(r.remained().is_empty());

    let mut r = Reader::new("-a".as_bytes());
    assert_eq!(r.read_int(), None);
    assert_eq!(r.remained(), b"-a");

    let mut r = Reader::new("\
        model Post {\n\
          title     String @db.VarChar(200)\n\
          n_authors Int    @default(1)\n\
          z_flag    Int    @default(-42)\n\
        }\
    ".to_string().into_bytes());

    #[cfg(feature="location")] assert_eq!(r.line(), 1);
    assert!(r.consume("model").is_some());
    r.skip_whitespace();
    assert_eq!(r.read_snake().unwrap(), "Post");
    r.skip_whitespace();
    assert_eq!(r.peek().unwrap(), &b'{'); r.advance_by(1);
    r.skip_whitespace();

    #[cfg(feature="location")] assert_eq!(r.line(), 2);
    assert_eq!(r.read_snake().unwrap(), "title");
    r.skip_whitespace();
    assert_eq!(r.read_snake().unwrap(), "String");
    r.skip_whitespace();
    assert_eq!(r.peek().unwrap(), &b'@'); r.advance_by(1);
    assert_eq!(r.read_snake().unwrap(), "db");
    assert!(r.consume(".").is_some());
    assert_eq!(r.read_snake().unwrap(), "VarChar");
    assert_eq!(r.peek().unwrap(), &b'('); r.advance_by(1);

    let int = r.read_uint().unwrap();
    assert_eq!(int, 200);
    assert_eq!(r.peek().unwrap(), &b')'); r.advance_by(1);

    r.skip_whitespace();

    #[cfg(feature="location")] assert_eq!(r.line(), 3);
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

    #[cfg(feature="location")] assert_eq!(r.line(), 4);
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

    #[cfg(feature="location")] assert_eq!(r.line(), 5);
    assert_eq!(r.peek().unwrap(), &b'}'); r.advance_by(1);
    assert_eq!(r.peek(), None)
}
