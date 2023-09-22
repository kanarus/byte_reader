#![doc(html_root_url = "https://docs.rs/byte_reader")]

mod traits;
#[cfg(test)] mod _test;

use traits::*;
use std::{format as f, borrow::Cow};


pub struct Reader<B: Bytes> {
    content:        B,
    current_idx:    usize,
    #[cfg(feature="location")] current_line:   usize,
    #[cfg(feature="location")] current_column: usize,
}
    
impl<B: Bytes> Reader<B> {
    /// Generate new `Reader` from `Vec<u8>` or `&[u8]`
    pub fn new(content: B) -> Self {
        Self {
            content,
            current_idx: 0,
            #[cfg(feature="location")] current_line:   1,
            #[cfg(feature="location")] current_column: 1,
        }
    }

    /// Returns current line number of the cursor (starts from 1)
    #[cfg(feature="location")] #[inline(always)] pub fn line(&self) -> usize {
        self.current_line
    }
    /// Returns current column number of the cursor (starts from 1)
    #[cfg(feature="location")] #[inline(always)] pub fn column(&self) -> usize {
        self.current_column
    }

    #[inline(always)] pub(crate) fn content(&self) -> &[u8] {
        self.content.bytes()
    }
    #[inline(always)] pub(crate) fn remained(&self) -> &[u8] {
        &self.content()[self.current_idx..]
    }
}

impl<B: Bytes> Reader<B> {
    #[cfg_attr(not(feature="location"), inline)] pub(crate) fn advance_unchecked_by(&mut self, n: usize) {
        #[cfg(feature="location")] {
            let mut line   = self.current_line.clone();
            let mut column = self.current_column.clone();
            for b in &self.remained()[..n] {
                if &b'\n' != b {
                    column += 1
                } else {
                    line += 1; column = 1
                }
            }
            self.current_line   = line;
            self.current_column = column;
        }

        self.current_idx += n;
    }

    /// Advance by `max_bytes` bytes (or, if remained bytes is shorter than `max_bytes`, read all remained bytes)
    #[inline(always)] pub fn advance_by(&mut self, max_bytes: usize) {
        self.advance_unchecked_by(max_bytes.min(self.remained().len()))
    }
}

impl<B: Bytes> Reader<B> {
    /// Read while the condition holds for the byte
    pub fn read_while(&mut self, condition: impl Fn(&u8)->bool) -> &[u8] {
        let start_idx = self.current_idx;
        let mut len = 0;
        while self.remained().get(len).is_some_and(|b| condition(b)) {
            len += 1
        }
        self.advance_unchecked_by(len);
        &self.content()[start_idx..(start_idx+len)]
    }

    /// Read next one byte, or return None if the reamined bytes is empty
    #[inline] pub fn next(&mut self) -> Option<u8> {
        let here = self.current_idx;
        self.advance_by(1);
        (self.current_idx > here).then(|| self.content()[here])
    }
    /// Read next one byte if the condition holds for it
    #[inline] pub fn next_if(&mut self, condition: impl Fn(&u8)->bool) -> Option<u8> {
        let value = self.peek()?.clone();
        condition(&value).then(|| {self.advance_unchecked_by(1); value})
    }

    /// Peek next byte (without consuming)
    #[inline(always)] pub fn peek(&self) -> Option<&u8> {
        self.remained().get(0)
    }
    /// Peek next byte of next byte (without consuming)
    #[inline] pub fn peek2(&self) -> Option<&u8> {
        self.remained().get(1)
    }
    /// Peek next byte of next byte of next byte (without consuming)
    pub fn peek3(&self) -> Option<&u8> {
        self.remained().get(2)
    }

    /// Advance while the byte is ascii-whitespace
    #[inline] pub fn skip_whitespace(&mut self) {
        let mut whitespace_len = 0;
        while self.remained().get(whitespace_len).is_some_and(|b| b.is_ascii_whitespace()) {
            whitespace_len += 1
        }
        self.advance_unchecked_by(whitespace_len)
    }

    /// Read `token` if the remained bytes starts with it, otherwise return `Err`
    #[inline] pub fn consume(&mut self, token: &'static str) -> Result<(), Cow<'static, str>> {
        self.remained().starts_with(token.as_bytes())
            .then(|| self.advance_unchecked_by(token.len()))
            .ok_or_else(|| Cow::Owned(f!("Expected token `{token}` but not found")))
    }
    /// Read first token in `tokens` that the remained bytes starts with, and returns the index of the (matched) token.
    /// 
    /// Returns `Err` if none matched.
    pub fn consume_oneof<const N: usize>(&mut self, tokens: [&'static str; N]) -> Result<usize, Cow<'static, str>> {
        for i in 0..tokens.len() {
            if self.remained().starts_with(&tokens[i].as_bytes()) {
                self.advance_by(tokens[i].len());
                return Ok(i)
            }
        }
        Err(Cow::Owned(f!("Expected oneof {} but none matched", tokens.map(|t| f!("`{t}`")).join(", "))))
    }

    /// Read a `camelCase` word like `helloWorld`, `userID`, ... as `String`
    #[inline] pub fn read_camel(&mut self) -> Result<String, Cow<'static, str>> {
        let ident_bytes = self.read_while(|b| matches!(b, b'a'..=b'z' | b'A'..=b'Z')).to_vec();
        if ident_bytes.len() == 0 {return Err(Cow::Borrowed("Expected an camelCase word but it wasn't found"))}
        Ok(unsafe {String::from_utf8_unchecked(ident_bytes)})
    }
    /// Read a `snake_case` word like `hello_world`, `user_id`, ... as `String`
    #[inline] pub fn read_snake(&mut self) -> Result<String, Cow<'static, str>> {
        let ident_bytes = self.read_while(|b| matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'_')).to_vec();
        if ident_bytes.len() == 0 {return Err(Cow::Borrowed("Expected an camelCase word but it wasn't found"))}
        Ok(unsafe {String::from_utf8_unchecked(ident_bytes)})
    }
    /// Read a `kebeb-case` word like `hello-world`, `Content-Type`, ... as `String`
    #[inline] pub fn read_kebab(&mut self) -> Result<String, Cow<'static, str>> {
        let ident_bytes = self.read_while(|b| matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'-')).to_vec();
        if ident_bytes.len() == 0 {return Err(Cow::Borrowed("Expected an camelCase word but it wasn't found"))}
        Ok(unsafe {String::from_utf8_unchecked(ident_bytes)})
    }

    /// Read a double-quoted string literal like `"Hello, world!"`, `"application/json"`, ... and return the quoted content as `String`
    /// 
    /// This doesn't handle escape sequences
    pub fn read_string(&mut self) -> Result<String, Cow<'static, str>> {
        self.consume("\"")?;
        let content = self.read_while(|b| b != &b'"').to_vec();
        self.consume("\"")?;

        String::from_utf8(content).map_err(|e| Cow::Owned(f!("{e}")))
    }
    /// Read a double-quoted string literal like `"Hello, world!"`, `"application/json"`, ... the  and return the quoted content as `String` **without checking** if the content bytes is valid UTF-8
    /// 
    /// This doesn't handle escape sequences
    pub unsafe fn read_string_unchecked(&mut self) -> Result<String, Cow<'static, str>> {
        self.consume("\"")?;
        let content = self.read_while(|b| b != &b'"').to_vec();
        self.consume("\"")?;

        Ok(String::from_utf8_unchecked(content))
    }
    /// Read an unsigned integer literal like `42`, `123` as `usize`
    #[inline] pub fn read_uint(&mut self) -> Result<usize, Cow<'static, str>> {
        let digits = self.read_while(|b| &b'0' <= b && b <= &b'9');
        if digits.len() == 0 {return Err(Cow::Borrowed("Expected an integer but not found"))}

        Ok(digits.into_iter().fold(0, |int, d| int * 10 + (*d - b'0') as usize))
    }
    /// Read an integer literal like `42`, `-1111` as `isize`
    #[inline] pub fn read_int(&mut self) -> Result<isize, Cow<'static, str>> {
        let negetive = self.consume("-").is_ok();
        let absolute = self.read_uint()? as isize;
        
        Ok(if negetive { -absolute } else {absolute})
    }
}
