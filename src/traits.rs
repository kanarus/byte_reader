fn __unreachable__() -> ! {unsafe {std::hint::unreachable_unchecked()}}

pub trait Bytes {fn bytes(&self) -> &[u8];}
impl Bytes for Vec<u8> {
    #[inline(always)] fn bytes(&self) -> &[u8] {&self}
}
impl Bytes for &[u8] {
    #[inline(always)] fn bytes(&self) -> &[u8] {self}
}

pub trait AsBytes {fn _as_bytes(&self) -> &[u8];}
impl AsBytes for &str {
    #[inline(always)] fn _as_bytes(&self) -> &[u8] {self.as_bytes()}
}
impl AsBytes for &[u8] {
    #[inline(always)] fn _as_bytes(&self) -> &[u8] {self}
}
