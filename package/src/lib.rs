#![doc(html_root_url = "https://docs.rs/byte_reader")]

pub struct Reader<'b> {
    buf:  &'b [u8],
    size: usize,
    pub index: usize,
    /// Line of current parsing point
    #[cfg(feature="location")] pub line: usize,
    /// Column of current parsing point
    #[cfg(feature="location")] pub column: usize,
}

unsafe impl<'b> Send for Reader<'b> {}
unsafe impl<'b> Sync for Reader<'b> {}

impl<'b> Reader<'b> {
    pub fn new(content: &'b (impl AsRef<[u8]> + ?Sized)) -> Self {
        let buf = content.as_ref();
        Self {
            buf,
            size:  buf.len(),
            index: 0,
            #[cfg(feature="location")] line:   1,
            #[cfg(feature="location")] column: 1,
        }
    }

    #[inline(always)] pub fn remaining(&self) -> &[u8] {
        unsafe {self.buf.get_unchecked(self.index..)}
    }
    #[inline(always)] unsafe fn get_unchecked(&self, index: usize) -> &u8 {
        self.buf.get_unchecked(index)
    }

    #[inline] fn advance_unchecked_by(&mut self, n: usize) {
        #[cfg(feature="location")] {
            let mut line   = self.line;
            let mut column = self.column;
            for b in unsafe {self.buf.get_unchecked(self.index..(self.index + n))} {
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
                if self.get_unchecked(here) != &b'\n' {
                    column -= 1
                } else {
                    line -= 1; column = 'c: {
                        for j in 1..=here {
                            if self.get_unchecked(here - j) == &b'\n' {break 'c j}
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
        let by = self.remaining().iter().take_while(|b| condition(b)).count();
        self.advance_unchecked_by(by)
    }
    /// `skip_while(u8::is_ascii_whitespace)`
    #[inline] pub fn skip_whitespace(&mut self) {
        self.skip_while(u8::is_ascii_whitespace)
    }
    /// Read next byte while the condition holds on it
    #[inline] pub fn read_while(&mut self, condition: impl Fn(&u8)->bool) -> &[u8] {
        let start = self.index;
        self.skip_while(condition);
        unsafe {self.buf.get_unchecked(start..self.index)}
    }

    /// Read next byte, or return None if the remaining bytes is empty
    #[inline] pub fn next(&mut self) -> Option<u8> {
        let here = self.index;
        self.advance_by(1);
        (self.index != here).then(|| *unsafe{ self.get_unchecked(here)})
    }
    /// Read next byte if the condition holds on it
    #[inline] pub fn next_if(&mut self, condition: impl Fn(&u8)->bool) -> Option<u8> {
        let value = self.peek()?.clone();
        condition(&value).then(|| {self.advance_unchecked_by(1); value})
    }

    /// Peek next byte (without consuming)
    #[inline(always)] pub fn peek(&self) -> Option<&u8> {
        (self.size - self.index > 0).then(|| unsafe {self.get_unchecked(self.index)})
    }
    /// Peek next byte of next byte (without consuming)
    #[inline] pub fn peek2(&self) -> Option<&u8> {
        (self.size - self.index > 1).then(|| unsafe {self.get_unchecked(self.index + 1)})
    }
    /// Peek next byte of next byte of next byte (without consuming)
    pub fn peek3(&self) -> Option<&u8> {
        (self.size - self.index > 2).then(|| unsafe {self.get_unchecked(self.index + 2)})
    }

    /// Read `token` if the remaining bytes start with it
    #[inline] pub fn consume(&mut self, token: impl AsRef<[u8]>) -> Option<()> {
        let token = token.as_ref();
        let n = token.len();
        (self.size - self.index >= n && unsafe {
            self.buf.get_unchecked(self.index..(self.index + n))
        } == token).then(|| self.advance_unchecked_by(n))
    }
    /// Read the first token in `tokens` that matches the start with the remaining bytes, and returns the index of the (matched) token, or `None` if none matches
    pub fn consume_oneof<const N: usize>(&mut self, tokens: [impl AsRef<[u8]>; N]) -> Option<usize> {
        for i in 0..tokens.len() {
            let token = tokens[i].as_ref();
            if self.remaining().starts_with(token) {
                self.advance_unchecked_by(token.len());
                return Some(i)
            }
        }; None
    }
}

#[cfg(feature="text")]
impl<'b> Reader<'b> {
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

    /// Read all bytes enclosed between `left` and `right`, then consume `left`, the bytes and `right`, and return the bytes.
    /// 
    /// Or, returns `None` if `left` or `right` is not found in remaining bytes.
    #[inline] pub fn read_quoted_by(&mut self, left: u8, right: u8) -> Option<&[u8]> {
        if self.peek()? != &left {return None}
        let content_len = self.remaining()[1..].iter().take_while(|b| b != &&right).count();
        let eoq /* end of quotation */ = 0 + content_len + 1;
        if self.remaining().get(eoq)? != &right {return None}

        self.advance_unchecked_by(eoq + 1);

        Some(unsafe {self.buf.get_unchecked((self.index - eoq)..(self.index - eoq + content_len))})
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
            let (abs, n_digits) = self.remaining()[1..].iter()
                .map_while(|b| b.is_ascii_digit().then(|| *b - b'0'))
                .fold((0, 0), |(abs, n), d| (abs*10+d as isize, n+1));
            (n_digits > 0).then(|| {
                self.advance_unchecked_by(1/*'-'*/ + n_digits); -abs})
        }
    }
}
