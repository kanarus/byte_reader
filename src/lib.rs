#![doc(html_root_url = "https://docs.rs/byte_reader")]

#[cfg(test)] mod _test;

use core::slice;
use std::marker::PhantomData;

/// A **minimal** byte-by-byte reader for parsing input
/// 
/// ```
/// use byte_reader::Reader;
/// 
/// fn main() {
///     // Get a input from a File, standard input, or others
///     // Input must be a reference that implements `AsRef<[u8]>`
///     let sample_input = "Hello,    byte_reader!";
/// 
///     // Create mutable `r`
///     let mut r = Reader::new(sample_input);
/// 
///     // Use some simple operations
///     // to parse the input
///     r.consume("Hello").unwrap();
///     r.consume(",").unwrap();
///     r.skip_whitespace();
///     let name = r.read_snake().unwrap(); // byte_reader
///     r.consume("!").unwrap();
/// 
///     println!("Greeted to `{name}`.");
/// }
/// ```
/// 
/// <br/>
/// 
/// ## Features
/// - `"location"`
/// 
/// You can track the reader's parsing location ( **line**, **column** and **index** ) in the input bytes.
pub struct Reader<'b> {_lifetime: PhantomData<&'b()>,
    head: *const u8,
    size: usize,

    #[cfg(not(feature="location"))] index: usize,
    /// Index of current parsing point
    #[cfg(feature="location")]  pub index: usize,

    /// Line of current parsing point
    #[cfg(feature="location")] pub line:   usize,
    /// Column of current parsing point
    #[cfg(feature="location")] pub column: usize,
}

impl<'b> Reader<'b> {
    pub fn new(content: &'b (impl AsRef<[u8]> + ?Sized)) -> Self {
        let slice = content.as_ref();
        Self {_lifetime: PhantomData,
            head:  slice.as_ptr(),
            size:  slice.len(),
            index: 0,
            #[cfg(feature="location")] line:   1,
            #[cfg(feature="location")] column: 1,
        }
    }

    #[inline(always)] fn _remained(&self) -> &[u8] {
        unsafe {slice::from_raw_parts(self.head.add(self.index), self.size - self.index)}
    }
    #[inline(always)] unsafe fn _get_unchecked(&self, index: usize) -> &u8 {
        &*self.head.add(index)
    }

    #[inline] fn advance_unchecked_by(&mut self, n: usize) {
        #[cfg(feature="location")] {
            let mut line   = self.line;
            let mut column = self.column;
            for b in unsafe {slice::from_raw_parts(self.head.add(self.index), n)} {
                if &b'\n' != b {
                    column += 1
                } else {
                    line += 1; column = 1
                }
            }
            self.line   = line;
            self.column = column;
        }
        self.index += n;
    }
    #[cfg_attr(not(feature="location"), inline)] fn unwind_unchecked_by(&mut self, n: usize) {
        #[cfg(feature="location")] unsafe {
            let mut line   = self.line;
            let mut column = self.column;
            for i in 1..=n {let here = self.index - i;
                if self._get_unchecked(here) != &b'\n' {
                    column -= 1
                } else {
                    line -= 1; column = 'c: {
                        for j in 1..=here {
                            if self._get_unchecked(here - j) == &b'\n' {break 'c j}
                        }; here + 1
                    }
                }
            }
            self.line   = line;
            self.column = column;
        }
        self.index -= n;
    }
    /// Advance by `max` bytes (or, if remaining bytes is shorter than `max`, read all remaining bytes)
    #[inline(always)] pub fn advance_by(&mut self, max: usize) {
        self.advance_unchecked_by(max.min(self.size - self.index))
    }
    /// Unwind the parsing point by `max` bytes (or, if already-read bytes is shorter than `max`, rewind all)
    /// 
    /// When `"location"` feature is activated, this may be *less performant* for some extensive input
    pub fn unwind_by(&mut self, max: usize) {
        self.unwind_unchecked_by(max.min(self.index))
    }

    /// Skip next byte while `condition` holds on it
    #[inline] pub fn skip_while(&mut self, condition: impl Fn(&u8)->bool) {
        let by = self._remained().iter().take_while(|b| condition(b)).count();
        self.advance_unchecked_by(by)
    }
    /// `.skip_while(|b| b.is_ascii_whitespace())`
    #[inline] pub fn skip_whitespace(&mut self) {
        self.skip_while(|b| b.is_ascii_whitespace())
    }
    /// Read next byte while the condition holds on it
    #[inline] pub fn read_while(&mut self, condition: impl Fn(&u8)->bool) -> &[u8] {
        let start = self.index;
        self.skip_while(condition);
        unsafe {slice::from_raw_parts(self.head.add(start), self.index - start)}
    }

    /// Read next byte, or return None if the remaining bytes is empty
    #[inline] pub fn next(&mut self) -> Option<u8> {
        let here = self.index;
        self.advance_by(1);
        (self.index != here).then(|| *unsafe{ self._get_unchecked(here)})
    }
    /// Read next byte if the condition holds on it
    #[inline] pub fn next_if(&mut self, condition: impl Fn(&u8)->bool) -> Option<u8> {
        let value = self.peek()?.clone();
        condition(&value).then(|| {self.advance_unchecked_by(1); value})
    }

    /// Peek next byte (without consuming)
    #[inline(always)] pub fn peek(&self) -> Option<&u8> {
        (self.size - self.index > 0).then(|| unsafe {self._get_unchecked(self.index)})
    }
    /// Peek next byte of next byte (without consuming)
    #[inline] pub fn peek2(&self) -> Option<&u8> {
        (self.size - self.index > 1).then(|| unsafe {self._get_unchecked(self.index + 1)})
    }
    /// Peek next byte of next byte of next byte (without consuming)
    pub fn peek3(&self) -> Option<&u8> {
        (self.size - self.index > 2).then(|| unsafe {self._get_unchecked(self.index + 2)})
    }

    /// Read `token` if the remaining bytes start with it
    #[inline] pub fn consume(&mut self, token: impl AsRef<[u8]>) -> Option<()> {
        let token = token.as_ref();
        let n = token.len();
        (self.size - self.index >= n && unsafe {
            slice::from_raw_parts(self.head.add(self.index), n)
        } == token).then(|| self.advance_unchecked_by(n))
    }
    /// Read the first token in `tokens` that matches the start with the remaining bytes, and returns the index of the (matched) token, or `None` if none matches
    pub fn consume_oneof<const N: usize>(&mut self, tokens: [impl AsRef<[u8]>; N]) -> Option<usize> {
        for i in 0..tokens.len() {
            let token = tokens[i].as_ref();
            if self._remained().starts_with(token) {
                self.advance_unchecked_by(token.len());
                return Some(i)
            }
        }; None
    }

    /// Read a `camelCase` word like `helloWorld`, `userID`, ... as `String` if found
    #[inline] pub fn read_camel(&mut self) -> Option<String> {
        let ident_bytes = self.read_while(|b| matches!(b, b'a'..=b'z' | b'A'..=b'Z')).to_vec();
        // SAFETY: `ident_bytes` is consists of `b'a'..=b'z' | b'A'..=b'Z'`
        (ident_bytes.len() > 0).then(|| unsafe {String::from_utf8_unchecked(ident_bytes)})
    }
    /// Read a `snake_case` word like `hello_world`, `user_id`, ... as `String` if found
    #[inline] pub fn read_snake(&mut self) -> Option<String> {
        let ident_bytes = self.read_while(|b| matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'_')).to_vec();
        // SAFETY: `ident_bytes` is consists of `b'a'..=b'z' | b'A'..=b'Z' | b'_'`
        (ident_bytes.len() > 0).then(|| unsafe {String::from_utf8_unchecked(ident_bytes)})
    }
    /// Read a `kebeb-case` word like `hello-world`, `Content-Type`, ... as `String` if found
    #[inline] pub fn read_kebab(&mut self) -> Option<String> {
        let ident_bytes = self.read_while(|b| matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'-')).to_vec();
        // SAFETY: `ident_bytes` is consists of `b'a'..=b'z' | b'A'..=b'Z' | b'-'`
        (ident_bytes.len() > 0).then(|| unsafe {String::from_utf8_unchecked(ident_bytes)})
    }

    /// Read a double-quoted **UTF-8** string literal like `"Hello, world!"`, `"application/json"`, ... and return the quoted content as `String`
    /// 
    /// - Returns `None` if expected `"`s were not found
    /// - Returns `None` if the quoted bytes is not UTF-8
    /// - Doesn't handle escape sequences
    #[inline] pub fn read_string(&mut self) -> Option<String> {
        if self.peek()? != &b'"' {return None}
        let string = String::from_utf8(self._remained()[1..].iter().map_while(|b| (b != &b'"').then(|| *b)).collect()).ok()?;
        let eoq/* end of quotation */ = 0 + string.len() + 1;
        if self._remained().get(eoq)? != &b'"' {return None}
        self.advance_unchecked_by(eoq + 1);
        Some(string)
    }
    /// Read a double-quoted string literal like `"Hello, world!"`, `"application/json"`, ... the  and return the quoted content as `String` **without checking** if the content bytes is valid UTF-8
    /// 
    /// - Doesn't handle escape sequences
    pub unsafe fn read_string_unchecked(&mut self) -> Option<String> {
        if self.peek()? != &b'"' {return None}
        let string = unsafe {String::from_utf8_unchecked(self._remained()[1..].iter().map_while(|b| (b != &b'"').then(|| *b)).collect())};
        let eoq = 0 + string.len() + 1;
        if self._remained().get(eoq)? != &b'"' {return None}
        self.advance_unchecked_by(eoq + 1);
        Some(string)
    }
    /// Read an unsigned integer literal like `42`, `123` as `usize` if found
    /// 
    /// - Panics if the integer is larger than `usize::MAX`
    #[inline] pub fn read_uint(&mut self) -> Option<usize> {
        let digits = self.read_while(|b| b.is_ascii_digit());
        (digits.len() > 0).then(|| digits.into_iter().fold(0, |uint, d| uint*10 + (*d-b'0') as usize))
    }
    /// Read an integer literal like `42`, `-1111` as `isize` if found
    /// 
    /// - Panics if not `isize::MIN` <= {the integer} <= `isize::MAX`
    #[inline] pub fn read_int(&mut self) -> Option<isize> {
        if self.peek()? != &b'-' {
            self.read_uint().map(|u| u as isize)
        } else {
            let (abs, n_digits) = self._remained()[1..].iter()
                .map_while(|b| b.is_ascii_digit().then(|| *b - b'0'))
                .fold((0, 0), |(abs, n), d| (abs*10+d as isize, n+1));
            (n_digits > 0).then(|| {
                self.advance_unchecked_by(1/*'-'*/ + n_digits); -abs})
        }
    }
}
