pub trait Bytes {fn bytes(&self) -> &[u8];}
impl Bytes for Vec<u8> {
    #[inline(always)] fn bytes(&self) -> &[u8] {&self}
}
impl Bytes for &[u8] {
    #[inline(always)] fn bytes(&self) -> &[u8] {self}
}
impl Bytes for String {
    #[inline(always)] fn bytes(&self) -> &[u8] {self.as_bytes()}
}
impl Bytes for &str {
    #[inline(always)] fn bytes(&self) -> &[u8] {self.as_bytes()}
}