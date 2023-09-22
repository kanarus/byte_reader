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
    #[inline(always)] pub(crate) fn remained_len(&self) -> usize {
        self.content().len() - self.current_idx + 1
    }
}

impl<B: Bytes> Reader<B> {
    /// Advance by `n` bytes **without checking** if the remained bytes is not shorter then `n`
    /// 
    /// <br/>
    /// 
    /// # Panic
    /// - When the remained bytes is shorter than `n`
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
        self.advance_unchecked_by(max_bytes.min(self.remained_len()))
    }
}

impl<B: Bytes> Reader<B> {
    /// Read while the condition holds for the byte
    pub fn read_while(&mut self, condition: impl Fn(&u8)->bool) -> &[u8] {
        let start_idx = self.current_idx;
        while self.peek().is_some_and(|b| condition(b)) {
            self.advance_unchecked_by(1)
        }
        &self.content()[start_idx..self.current_idx]
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
        while self.remained().first().is_some_and(|b| b.is_ascii_whitespace()) {
            self.advance_unchecked_by(1)
        }
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
        let mut ident_len = 0;
        while matches!(
            self.remained().get(ident_len),
            Some(b'a'..=b'z' | b'A'..=b'Z')
        ) {
            ident_len += 1
        }
        if ident_len == 0 {return Err(Cow::Borrowed("Expected an camelCase word but it wasn't found"))}

        let ident = unsafe { String::from_utf8_unchecked(self.remained()[..ident_len].to_vec()) };
        self.advance_unchecked_by(ident_len);
        Ok(ident)
    }
    /// Read a `snake_case` word like `hello_world`, `user_id`, ... as `String`
    #[inline] pub fn read_snake(&mut self) -> Result<String, Cow<'static, str>> {
        let mut ident_len = 0;
        while matches!(
            self.remained().get(ident_len),
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'_')
        ) {
            ident_len += 1
        }
        if ident_len == 0 {return Err(Cow::Borrowed("Expected an snake_case word but it wasn't found"))}

        let ident = unsafe {String::from_utf8_unchecked(self.remained()[..ident_len].to_vec())};
        self.advance_unchecked_by(ident_len);
        Ok(ident)
    }
    /// Read a `kebeb-case` word like `hello-world`, `Content-Type`, ... as `String`
    #[inline] pub fn read_kebab(&mut self) -> Result<String, Cow<'static, str>> {
        let mut ident_len = 0;
        while matches!(
            self.remained().get(ident_len),
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'-')
        ) {
            ident_len += 1
        }
        if ident_len == 0 {return Err(Cow::Borrowed("Expected an kebab-case word but it wasn't found"))}

        let ident = unsafe { String::from_utf8_unchecked(self.remained()[..ident_len].to_vec()) };
        self.advance_unchecked_by(ident_len);
        Ok(ident)
    }

    /// Read a double-quoted string literal like `"Hello, world!"`, `"application/json"`, ... and return the quoted content as `String`
    /// 
    /// This doesn't handle escape sequences
    pub fn read_string(&mut self) -> Result<String, Cow<'static, str>> {
        self.consume("\"")?;
        let mut literal_bytes = Vec::new();
        while let Some(b) = self.peek() {
            if b != &b'"' {
                literal_bytes.push(*b)
            } else {break}
        }
        self.consume("\"")?;

        Ok(unsafe { String::from_utf8_unchecked(literal_bytes) })
    }
    /// Read an unsigned integer literal like `42`, `123` as `usize`
    pub fn read_uint(&mut self) -> Result<usize, Cow<'static, str>> {
        let mut int = 0;

        let mut degit = 0;
        while let Some(b) = self.peek() {
            match b {
                b'0'..=b'9' => {int = int * 10 + (*b - b'0') as usize; degit += 1; self.advance_by(1)}
                _ => break,
            }
        }
        if degit == 0 {return Err(Cow::Borrowed("Expected an integer but not found"))}

        Ok(int)
    }
    /// Read an integer literal like `42`, `-1111` as `isize`
    #[inline] pub fn read_int(&mut self) -> Result<isize, Cow<'static, str>> {
        let negetive = self.consume("-").is_ok();
        let absolute = self.read_uint()? as isize;
        
        Ok(if negetive { -absolute } else {absolute})
    }
}
