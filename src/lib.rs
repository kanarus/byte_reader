#![doc(html_root_url = "https://docs.rs/byte_reader")]

#[cfg(test)] mod _test;

/// A **minimal** byte-by-byte reader for parsing input
/// 
/// <br/>
/// 
/// <h2><a href="https://github.com/kana-rus/byte_reader/blob/main/examples/usage.rs">Usage</a></h2>
/// 
/// ```
/// use byte_reader::Reader;
/// 
/// fn main() {
///     // Get a input from a File, standard input, or others
///     // Input must implement `AsRef<[u8]>`
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
pub struct Reader {
    content: Content,

    #[cfg(not(feature="location"))] index: usize,
    /// Index of current parsing point
    #[cfg(feature="location")]  pub index: usize,

    /// Line of current parsing point
    #[cfg(feature="location")] pub line:   usize,
    /// Column of current parsing point
    #[cfg(feature="location")] pub column: usize,
}
struct Content {
    head: *const u8,
    size: usize,
} impl Content {
    #[inline] fn from_slice(s: &[u8]) -> Self {
        Self {
            head: s.as_ptr(),
            size: s.len(),
        }
    }
}

impl Reader {
    pub fn new(content: impl AsRef<[u8]>) -> Self {
        Self {
            content: Content::from_slice(content.as_ref()),
            index:   0,
            #[cfg(feature="location")] line:   1,
            #[cfg(feature="location")] column: 1,
        }
    }

    #[inline(always)] fn content(&self) -> &[u8] {
        self.content.as_ref()
    }
    #[inline(always)] fn remained(&self) -> &[u8] {
        &self.content()[self.index..]
    }

    #[inline] fn advance_unchecked_by(&mut self, n: usize) {
        #[cfg(feature="location")] {
            let mut line   = self.line;
            let mut column = self.column;
            for b in &self.remained()[..n] {
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
        #[cfg(feature="location")] {
            let mut line   = self.line;
            let mut column = self.column;
            let c = self.content();
            for i in 1..=n {let here = self.index - i;
                if &c[here] != &b'\n' {
                    column -= 1
                } else {
                    line -= 1; column = 'c: {
                        for j in 1..=here {
                            if &c[here - j] == &b'\n' {break 'c j}
                        }; here + 1
                    }
                }
            }
            self.line   = line;
            self.column = column;
        }
        self.index -= n;
    }
    /// Advance by `max` bytes (or, if remained bytes is shorter than `max`, read all remained bytes)
    #[inline(always)] pub fn advance_by(&mut self, max: usize) {
        self.advance_unchecked_by(max.min(self.remained().len()))
    }
    /// Unwind the parsing point by `max` bytes (or, if already-read bytes is shorter than `max`, rewind all)
    /// 
    /// When `"location"` feature is activated, this may be *less performant* for some extensive input
    pub fn unwind_by(&mut self, max: usize) {
        self.unwind_unchecked_by(max.min(self.index))
    }

    /// Skip next byte while `condition` holds on it
    #[inline] pub fn skip_while(&mut self, condition: impl Fn(&u8)->bool) {
        self.advance_unchecked_by(
            self.remained().iter().take_while(|b| condition(b)).count())
    }
    /// `.skip_while(|b| b.is_ascii_whitespace())`
    #[inline] pub fn skip_whitespace(&mut self) {
        self.skip_while(|b| b.is_ascii_whitespace())
    }
    /// Read next byte while the condition holds on it
    #[inline] pub fn read_while(&mut self, condition: impl Fn(&u8)->bool) -> &[u8] {
        let start = self.index;
        self.skip_while(condition);
        &self.content()[start..self.index]
    }

    /// Read next one byte, or return None if the remained bytes is empty
    #[inline] pub fn next(&mut self) -> Option<u8> {
        let here = self.index;
        self.advance_by(1);
        (self.index != here).then(|| self.content()[here])
    }
    /// Read next one byte if the condition holds on it
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

    /// Read `token` if the remained bytes starts with it
    #[inline] pub fn consume(&mut self, token: impl AsRef<[u8]>) -> Option<()> {
        let token = token.as_ref();
        self.remained().starts_with(token).then(|| self.advance_unchecked_by(token.len()))
    }
    /// Read first `token` in `tokens` that the remained bytes starts with, and returns the index of the (matched) token, or `None` if none matched
    pub fn consume_oneof<const N: usize>(&mut self, tokens: [impl AsRef<[u8]>; N]) -> Option<usize> {
        for i in 0..tokens.len() {
            let token = tokens[i].as_ref();
            if self.remained().starts_with(token) {
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
        let string = String::from_utf8(
            self.remained()[1..].iter().map_while(|b| (b != &b'"').then(|| *b)).collect()
        ).ok()?;
        let eoq/* end of quotation */ = 0 + string.len() + 1;
        if self.remained().get(eoq)? != &b'"' {return None}

        self.advance_unchecked_by(eoq + 1);
        Some(string)
    }
    /// Read a double-quoted string literal like `"Hello, world!"`, `"application/json"`, ... the  and return the quoted content as `String` **without checking** if the content bytes is valid UTF-8
    /// 
    /// - Doesn't handle escape sequences
    pub unsafe fn read_string_unchecked(&mut self) -> Option<String> {
        if self.peek()? != &b'"' {return None}
        let string = unsafe {String::from_utf8_unchecked(
            self.remained()[1..].iter().map_while(|b| (b != &b'"').then(|| *b)).collect()
        )};
        let eoq = 0 + string.len() + 1;
        if self.remained().get(eoq)? != &b'"' {return None}

        self.advance_unchecked_by(eoq + 1);
        Some(string)
    }
    /// Read an unsigned integer literal like `42`, `123` as `usize` if found
    /// 
    /// - Panics if the integer is larger then `usize::MAX`
    #[inline] pub fn read_uint(&mut self) -> Option<usize> {
        let digits = self.read_while(|b| b.is_ascii_digit());
        (digits.len() > 0).then(|| digits.into_iter().fold(0, |uint, d| uint*10 + (*d-b'0') as usize))
    }
    /// Read an integer literal like `42`, `-1111` as `isize` if found
    /// 
    /// - Panics if the integer is larger then `isize::MAX` or smaller then `isize::MIN`
    #[inline] pub fn read_int(&mut self) -> Option<isize> {
        if self.peek()? != &b'-' {
            self.read_uint().map(|u| u as isize)
        } else {
            let (abs, n_digits) = self.remained()[1..].iter()
                .map_while(|b| b.is_ascii_digit().then(|| *b - b'0'))
                .fold((0, 0), |(abs, n), d| (abs*10+d as isize, n+1));
            (n_digits > 0).then(|| {
                self.advance_unchecked_by(1/*'-'*/ + n_digits); -abs})
        }
    }
}
