mod bytes;
#[cfg(test)] mod test;

use std::{borrow::Cow, format as f};
use bytes::{Bytes, IntoBytes};


pub struct Reader<'b> {
    content:        Bytes<'b>,
    current_idx:    usize,
    current_line:   usize,
    current_column: usize,
}

impl<'b> Reader<'b> {
    /// Generate new `Reader` from `Vec<u8>` or `&'b [u8]`
    pub fn new(content: impl IntoBytes<'b>) -> Self {
        let content = content.into_bytes();
        Self {
            content,
            current_idx:    0,
            current_line:   1,
            current_column: 1,
        }
    }
}

impl<'b> Reader<'b> {
    #[inline(always)] pub(crate) fn remained(&self) -> &[u8] {
        &self.content[self.current_idx..]
    }

    /// Returns current line number of the cursor (starts from 1)
    #[inline] pub fn line(&self) -> usize {
        self.current_line
    }
    /// Returns current column number of the cursor (starts from 1)
    #[inline] pub fn column(&self) -> usize {
        self.current_column
    }

    pub(crate) fn read_by(&mut self, max_bytes: usize) -> &[u8] {
        let start_idx = self.current_idx;

        let remained = self.remained();
        let add_idx  = max_bytes.min(remained.len());

        let mut line   = self.current_line.clone();
        let mut column = self.current_column.clone();

        for b in &remained[..add_idx] {
            if &b'\n' != b {
                column += 1
            } else {
                line += 1; column = 1
            }
        }

        self.current_idx += add_idx;
        self.current_line   = line;
        self.current_column = column;

        &self.content[start_idx..(start_idx + add_idx)]
    }
}

impl<'b> Reader<'b> {
    /// Read while the condition holds for the byte
    pub fn read_while(&mut self, condition: impl Fn(&u8)->bool) -> &[u8] {
        let mut until = 0;
        while self.remained().get(until).is_some_and(|b| condition(b)) {
            until += 1
        }
        self.read_by(until)
    }
    /// Advance by `max_bytes` bytes (or, if remained bytes is shorter than `max_bytes`, read all remained bytes)
    #[inline] pub fn advance_by(&mut self, max_bytes: usize) {
        let _ = self.read_by(max_bytes);
    }
    /// Read one byte if the condition holds for it
    #[inline] pub fn pop_if(&mut self, condition: impl Fn(&u8)->bool) -> Option<u8> {
        let value = self.peek()?.clone();
        if condition(&value) {self.advance_by(1); Some(value)} else {None}
    }

    /// Peek next byte (without consuming)
    #[inline] pub fn peek(&self) -> Option<&u8> {
        self.remained().get(0)
    }
    /// Peek next byte of next byte (without consuming)
    #[inline] pub fn peek2(&self) -> Option<&u8> {
        self.remained().get(1)
    }
    /// Peek next byte of next byte of next byte (without consuming)
    #[inline] pub fn peek3(&self) -> Option<&u8> {
        self.remained().get(2)
    }

    /// Advance while the byte is ascii-whitespace
    pub fn skip_whitespace(&mut self) {
        while self.remained().first().is_some_and(|b| b.is_ascii_whitespace()) {
            self.advance_by(1)
        }
    }

    /// Read `token` if the remained bytes starts with it, otherwise return `Err`
    pub fn consume(&mut self, token: &'static str) -> Result<(), Cow<'static, str>> {
        self.remained().starts_with(token.as_bytes())
            .then(|| self.advance_by(token.len()))
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
    pub fn read_camel(&mut self) -> Result<String, Cow<'static, str>> {
        let mut ident_len = 0;
        while matches!(self.remained()[ident_len], b'a'..=b'z' | b'A'..=b'Z') {
            ident_len += 1
        }
        if ident_len == 0 {return Err(Cow::Borrowed("Expected an camelCase word but it wasn't found"))}

        let ident = unsafe { String::from_utf8_unchecked(self.remained()[..ident_len].to_vec()) };
        self.advance_by(ident_len);
        Ok(ident)
    }
    /// Read a `snake_case` word like `hello_world`, `user_id`, ... as `String`
    pub fn read_snake(&mut self) -> Result<String, Cow<'static, str>> {
        let mut ident_len = 0;
        while matches!(self.remained()[ident_len], b'a'..=b'z' | b'A'..=b'Z' | b'_') {
            ident_len += 1
        }
        if ident_len == 0 {return Err(Cow::Borrowed("Expected an snake_case word but it wasn't found"))}

        let ident = unsafe { String::from_utf8_unchecked(self.remained()[..ident_len].to_vec()) };
        self.advance_by(ident_len);
        Ok(ident)
    }
    /// Read a `kebeb-case` word like `hello-world`, `Content-Type`, ... as `String`
    pub fn read_kebab(&mut self) -> Result<String, Cow<'static, str>> {
        let mut ident_len = 0;
        while matches!(self.remained()[ident_len], b'a'..=b'z' | b'A'..=b'Z' | b'-') {
            ident_len += 1
        }
        if ident_len == 0 {return Err(Cow::Borrowed("Expected an kebab-case word but it wasn't found"))}

        let ident = unsafe { String::from_utf8_unchecked(self.remained()[..ident_len].to_vec()) };
        self.advance_by(ident_len);
        Ok(ident)
    }

    /// Read a double-quoted string literal like `"Hello, world!"`, `"application/json"`, ... and return the quoted content as `String`
    pub fn read_string(&mut self) -> Result<String, Cow<'static, str>> {
        self.consume("\"")?;
        let mut literal_bytes = Vec::new();
        while self.remained().first().is_some_and(|b| &b'"' != b) {
            literal_bytes.push(self.remained()[0]);
            self.advance_by(1)
        }
        self.consume("\"")?;

        Ok(unsafe { String::from_utf8_unchecked(literal_bytes) })
    }
    /// Read a unsigned integer literal like `42`, `123` as `usize`
    pub fn read_unsigned_int(&mut self) -> Result<usize, Cow<'static, str>> {
        let mut int = 0;

        let mut degit   = 0;
        loop {
            let b = self.remained().first()
                .ok_or_else(|| Cow::Borrowed("Expected an integer but not found"))?;
            match b {
                b'0'..=b'9' => {int = int * 10 + (*b - b'0') as usize; degit += 1; self.advance_by(1)}
                _ => break,
            }
        }
        if degit == 0 {return Err(Cow::Borrowed("Expected an integer but not found"))}

        Ok(int)
    }
    /// Read a integer literal like `42`, `-1111` as `isize`
    pub fn read_int(&mut self) -> Result<isize, Cow<'static, str>> {
        let negetive = self.consume("-").is_ok();
        let absolute = self.read_unsigned_int()? as isize;
        
        Ok(if negetive { -absolute } else {absolute})
    }
}
