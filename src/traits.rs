pub trait Bytes {fn bytes(&self) -> &[u8];}
impl Bytes for Vec<u8> {
    #[inline(always)] fn bytes(&self) -> &[u8] {&self}
}
impl Bytes for &[u8] {
    #[inline(always)] fn bytes(&self) -> &[u8] {self}
}
