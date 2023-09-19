pub enum Bytes<'b> {
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
        #[inline] fn deref(&self) -> &Self::Target {
            match self {
                Self::Borrowed(slice) => slice,
                Self::Owned   (vec)   => vec,
            }
        }
    }

    impl<'b> Index<Range<usize>> for Bytes<'b> {
        type Output = [u8];
        #[inline] fn index(&self, range: Range<usize>) -> &Self::Output {&self.deref()[range]}
    }
    impl<'b> Index<RangeFrom<usize>> for Bytes<'b> {
        type Output = [u8];
        #[inline] fn index(&self, range: RangeFrom<usize>) -> &Self::Output {&self.deref()[range]}
    }
    impl<'b> Index<RangeInclusive<usize>> for Bytes<'b> {
        type Output = [u8];
        fn index(&self, range: RangeInclusive<usize>) -> &Self::Output {&self.deref()[range]}
    }
    impl<'b> Index<RangeTo<usize>> for Bytes<'b> {
        type Output = [u8];
        fn index(&self, range: RangeTo<usize>) -> &Self::Output {&self.deref()[range]}
    }
    impl<'b> Index<RangeToInclusive<usize>> for Bytes<'b> {
        type Output = [u8];
        fn index(&self, range: RangeToInclusive<usize>) -> &Self::Output {&self.deref()[range]}
    }
};

pub trait IntoBytes<'b> {
    fn into_bytes(self) -> Bytes<'b>;
} const _: () = {
    impl<'b, 'slice:'b> IntoBytes<'b> for &'slice [u8] {
        fn into_bytes(self) -> Bytes<'b> {
            Bytes::Borrowed(self)
        }
    }
    impl<'b> IntoBytes<'b> for Vec<u8> {
        fn into_bytes(self) -> Bytes<'b> {
            Bytes::Owned(self)
        }
    }
};
