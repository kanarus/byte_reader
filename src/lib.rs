#[cfg(test)] mod test;

use std::{borrow::Cow, format as f};


pub struct Reader<'b> {
    content:        Bytes<'b>,
    current_idx:    usize,
    current_line:   usize,
    current_column: usize,
}

enum Bytes<'b> {
    Borrowed(&'b [u8]),
    Owned(Vec<u8>),
} const _: () = {
    use std::ops::{
        Deref,
        Index,
        Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive,
    };

    impl<'b> Deref for Bytes<'b> {
        type Target = [u8];
        fn deref(&self) -> &Self::Target {
            match self {
                Self::Borrowed(slice) => slice,
                Self::Owned   (vec)   => vec,
            }
        }
    }

    impl<'b> Index<Range<usize>> for Bytes<'b> {
        type Output = [u8]; fn index(&self, range: Range<usize>) -> &Self::Output {&self.deref()[range]}
    }
    impl<'b> Index<RangeFrom<usize>> for Bytes<'b> {
        type Output = [u8]; fn index(&self, range: RangeFrom<usize>) -> &Self::Output {&self.deref()[range]}
    }
    impl<'b> Index<RangeInclusive<usize>> for Bytes<'b> {
        type Output = [u8]; fn index(&self, range: RangeInclusive<usize>) -> &Self::Output {&self.deref()[range]}
    }
    impl<'b> Index<RangeTo<usize>> for Bytes<'b> {
        type Output = [u8]; fn index(&self, range: RangeTo<usize>) -> &Self::Output {&self.deref()[range]}
    }
    impl<'b> Index<RangeToInclusive<usize>> for Bytes<'b> {
        type Output = [u8]; fn index(&self, range: RangeToInclusive<usize>) -> &Self::Output {&self.deref()[range]}
    }
};


impl<'b> Reader<'b> {
    pub fn owned(content: Vec<u8>) -> Result<Self, Cow<'static, str>> {
        Ok(Self {
            content: Bytes::Owned(content),
            current_idx:    0,
            current_line:   1,
            current_column: 1,
        })
    }
    pub fn borrowed(content: &'b [u8]) -> Result<Self, Cow<'static, str>> {
        Ok(Self {
            content: Bytes::Borrowed(content),
            current_idx:    0,
            current_line:   1,
            current_column: 1,
        })
    }

    #[inline(always)] pub(crate) fn remained(&self) -> &[u8] {
        &self.content[self.current_idx..]
    }

    #[inline(always)] pub fn line(&self) -> usize {
        self.current_line
    }
    #[inline(always)] pub fn column(&self) -> usize {
        self.current_column
    }
}

impl<'b> Reader<'b> {
    pub fn read(&mut self, max_bytes: usize) -> &[u8] {
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
    pub fn read_while(&mut self, condition: impl Fn(&u8)->bool) -> String {
        let mut read_bytes = Vec::new();
        while let Some(b) = self.pop_if(|_b| condition(_b)) {
            read_bytes.push(b)
        }
        unsafe {String::from_utf8_unchecked(read_bytes)}
    }
    #[inline] pub fn advance_by(&mut self, max_bytes: usize) {
        let _ = self.read(max_bytes);
    }
    #[inline] pub fn pop_if(&mut self, condition: impl Fn(&u8)->bool) -> Option<u8> {
        let value = self.peek()?.clone();
        if condition(&value) {self.advance_by(1); Some(value)} else {None}
    }

    #[inline] pub fn peek(&self) -> Option<&u8> {
        self.remained().get(0)
    }
    #[inline] pub fn peek2(&self) -> Option<&u8> {
        self.remained().get(1)
    }
    #[inline] pub fn peek3(&self) -> Option<&u8> {
        self.remained().get(2)
    }

    pub fn skip_whitespace(&mut self) {
        while self.remained().first().is_some_and(|b| b.is_ascii_whitespace()) {
            self.advance_by(1)
        }
    }
}

impl<'b> Reader<'b> {
    pub fn consume(&mut self, keyword: &'static str) -> Result<(), Cow<'static, str>> {
        self.remained().starts_with(keyword.as_bytes())
            .then(|| self.advance_by(keyword.len()))
            .ok_or_else(|| Cow::Owned(f!("Expected keyword `{keyword}` but not found")))
    }
    pub fn consume_oneof<const N: usize>(&mut self, keywords: [&'static str; N]) -> Result<usize, Cow<'static, str>> {
        for i in 0..keywords.len() {
            if self.remained().starts_with(&keywords[i].as_bytes()) {
                self.advance_by(keywords[i].len());
                return Ok(i)
            }
        }
        Err(Cow::Owned(f!("Expected oneof {} but none matched", keywords.map(|kw| f!("`{kw}`")).join(", "))))
    }

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
    pub fn read_int(&mut self) -> Result<isize, Cow<'static, str>> {
        let negetive = self.consume("-").is_ok();
        let absolute = self.read_unsigned_int()? as isize;
        
        Ok(if negetive { -absolute } else {absolute})
    }
}
