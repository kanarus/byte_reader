use byte_reader::Reader;

#[test] fn test_whitespace() {
    let mut r = Reader::new(b" ");
    r.skip_whitespace();
    assert!(r.remaining().is_empty());

    let mut r = Reader::new(b"  a");
    r.skip_whitespace();
    assert_eq!(r.remaining(), b"a");
}

#[test] fn test_consume() {
    let input = String::from("--boundary");
    let mut r = Reader::new(input.as_bytes());
    assert_eq!(r.consume_oneof(["--", "\r\n"]), Some(0));
    assert_eq!(r.remaining(), b"boundary");
}

#[test] fn test_advance() {
    let mut r = Reader::new(b"Hello, world!");

    r.advance_by(1);
    assert_eq!(r.remaining(), b"ello, world!");

    r.advance_by(3);
    assert_eq!(r.remaining(), b"o, world!");
}

#[test] fn test_unwind() {
    let mut r = Reader::new(b"Hello, world!\nMy name is byte_reader!");
    r.read_while(|b| b != &b'\n');
    assert_eq!(r.remaining(), b"\nMy name is byte_reader!");
    #[cfg(feature="location")] assert_eq!(r.line,   1);
    #[cfg(feature="location")] assert_eq!(r.column, 14);
    r.advance_by(3);
    assert_eq!(r.remaining(), b" name is byte_reader!");
    #[cfg(feature="location")] assert_eq!(r.line,   2);
    #[cfg(feature="location")] assert_eq!(r.column, 3);
    r.unwind_by(2);
    assert_eq!(r.remaining(), b"My name is byte_reader!");
    #[cfg(feature="location")] assert_eq!(r.line,   2);
    #[cfg(feature="location")] assert_eq!(r.column, 1);
    r.unwind_by(2);
    assert_eq!(r.remaining(), b"!\nMy name is byte_reader!");
    #[cfg(feature="location")] assert_eq!(r.line,   1);
    #[cfg(feature="location")] assert_eq!(r.column, 13);

    let mut r = Reader::new(b"Hello!\nMy name is!\nkanarus!");
    r.read_while(|b| b != &b'\n');
    r.advance_by(1);
    r.read_while(|b| b != &b'\n');
    r.advance_by(1);
    assert_eq!(r.remaining(), b"kanarus!");
    #[cfg(feature="location")] assert_eq!(r.line,   3);
    #[cfg(feature="location")] assert_eq!(r.column, 1);
    r.unwind_by(1);
    assert_eq!(r.remaining(), b"\nkanarus!");
    #[cfg(feature="location")] assert_eq!(r.line,   2);
    #[cfg(feature="location")] assert_eq!(r.column, 12);
    r.unwind_by(1);
    assert_eq!(r.remaining(), b"!\nkanarus!");
    #[cfg(feature="location")] assert_eq!(r.line,   2);
    #[cfg(feature="location")] assert_eq!(r.column, 11);

}

#[test] fn test_read_while() {
    let mut r = Reader::new(b"Hello,  world!");

    let read = r.read_while(|b| !b.is_ascii_whitespace());
    assert_eq!(read, b"Hello,");
    assert_eq!(r.remaining(), b"  world!");

    let read = r.read_while(|b| b.is_ascii_whitespace());
    assert_eq!(read, b"  ");
    assert_eq!(r.remaining(), b"world!");

    let read = r.read_while(|b| b.is_ascii_alphabetic());
    assert_eq!(read, b"world");
    assert_eq!(r.remaining(), b"!");

    let read = r.read_while(|_| true);
    assert_eq!(read, b"!");
    assert_eq!(r.remaining(), b"");
}

#[test] fn test_read_until() {
    let mut r = Reader::new(b"Hello, world!");

    let read = r.read_until(b" ");
    assert_eq!(read,          b"Hello,");
    assert_eq!(r.remaining(), b" world!");
    #[cfg(feature="location")]
    assert_eq!(r.column,     7);

    let read = r.read_until(b"rl");
    assert_eq!(read,          b" wo");
    assert_eq!(r.remaining(), b"rld!");
    #[cfg(feature="location")]
    assert_eq!(r.column,     10);


    let mut r = Reader::new(b"Hello, world!");

    r.consume("Hello").unwrap();
    let read = r.read_until(b"rl");
    assert_eq!(read,          b", wo");
    assert_eq!(r.remaining(), b"rld!");
    #[cfg(feature="location")]
    assert_eq!(r.column,     10);


    let mut r = Reader::new(b"Hello, world!");

    let read = r.read_until(b"");
    assert_eq!(read,          b"");
    assert_eq!(r.remaining(), b"Hello, world!");
    #[cfg(feature="location")]
    assert_eq!(r.column,     1);


    let mut r = Reader::new(b"Hello, world!");

    let read = r.read_until(b"xyz");
    assert_eq!(read,          b"Hello, world!");
    assert_eq!(r.remaining(), b"");
    #[cfg(feature="location")]
    assert_eq!(r.column,     14);
}

#[cfg(feature="detached")]
#[test] fn detached_ref() {
    let mut r = Reader::new(b"Hello, world!");

    let some = r.read_while(u8::is_ascii);
    if r.peek().is_some() {
        let _ = some;
    }
}

#[cfg(feature="text")]
#[test] fn test_read_ident() {
    let mut r = Reader::new(b"Hello, world! I am a Reader!");
    assert_eq!(r.remaining(), b"Hello, world! I am a Reader!");

    let ident = r.read_snake().unwrap();
    assert_eq!(ident, "Hello");
    assert_eq!(r.remaining(), b", world! I am a Reader!");

    assert!(r.read_snake().is_none());
    r.advance_by(1);
    assert!(r.read_snake().is_none());
    r.advance_by(1);

    let ident = r.read_snake().unwrap();
    assert_eq!(ident, "world");
    assert_eq!(r.remaining(), b"! I am a Reader!")
}

#[cfg(feature="text")]
#[test] fn test_read_quoted() {
    let mut r = Reader::new(b"");
    assert_eq!(r.read_quoted_by(b'"', b'"'), None);
    assert_eq!(r.remaining(), b"");

    let mut r = Reader::new(b"Yeah, \"Hello!");
    assert_eq!(r.read_quoted_by(b'"', b'"'), None);
    r.consume("Yeah, ").unwrap();
    assert_eq!(r.remaining(), b"\"Hello!");
    assert_eq!(r.read_quoted_by(b'"', b'"'), None);
    assert_eq!(r.remaining(), b"\"Hello!");

    let mut r = Reader::new(b"\
        \"Hello,\" (He said,) \"I am Reader!\"\
    ");

    let lit = r.read_quoted_by(b'"', b'"').unwrap();
    assert_eq!(lit, b"Hello,");
    assert_eq!(r.remaining(), b" (He said,) \"I am Reader!\"");

    assert!(r.read_quoted_by(b'"', b'"').is_none());
    r.skip_whitespace();

    let parenthized = r.read_quoted_by(b'(', b')').unwrap();
    {
        let mut r = Reader::new(parenthized);

        assert_eq!(r.read_snake().unwrap(), "He");
        r.skip_whitespace();
        assert_eq!(r.read_snake().unwrap(), "said");
        assert_eq!(r.peek().unwrap(), &b',');
        r.advance_by(1);
    }
    assert_eq!(r.remaining(), b" \"I am Reader!\"");

    r.skip_whitespace();

    let lit = r.read_quoted_by(b'"', b'"').unwrap();
    assert_eq!(lit, b"I am Reader!");
    assert_eq!(r.remaining(), b"");
}

#[cfg(feature="text")]
#[test] fn test_read_int() {
    let mut r = Reader::new(b"42");
    assert_eq!(r.read_int(), Some(42));
    assert!(r.remaining().is_empty());

    let mut r = Reader::new(b"-42");
    assert_eq!(r.read_int(), Some(-42));
    assert!(r.remaining().is_empty());

    let mut r = Reader::new(b"-a");
    assert_eq!(r.read_int(), None);
    assert_eq!(r.remaining(), b"-a");

    let mut r = Reader::new(b"\
        model Post {\n\
          title     String @db.VarChar(200)\n\
          n_authors Int    @default(1)\n\
          z_flag    Int    @default(-42)\n\
        }\
    ");

    #[cfg(feature="location")] assert_eq!(r.line, 1);
    assert!(r.consume("model").is_some());
    r.skip_whitespace();
    assert_eq!(r.read_snake().unwrap(), "Post");
    r.skip_whitespace();
    assert_eq!(r.peek().unwrap(), &b'{'); r.advance_by(1);
    r.skip_whitespace();

    #[cfg(feature="location")] assert_eq!(r.line, 2);
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

    #[cfg(feature="location")] assert_eq!(r.line, 3);
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

    #[cfg(feature="location")] assert_eq!(r.line, 4);
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

    #[cfg(feature="location")] assert_eq!(r.line, 5);
    assert_eq!(r.peek().unwrap(), &b'}'); r.advance_by(1);
    assert_eq!(r.peek(), None)
}
