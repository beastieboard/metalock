use super::macros::impl_deref;

use super::decode::{Decode, R};





pub type Parser<'a> = &'a mut ParserBuffer;


#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ParserBuffer(pub &'static [u8]);
impl_deref!([], ParserBuffer => &'static [u8], 0);
impl ParserBuffer {
    pub fn new(r: &[u8]) -> ParserBuffer {
        ParserBuffer(unsafe { &*(r as *const [u8]) })
    }
    pub fn set(&mut self, o: ParserBuffer) {
        self.0 = o.0;
    }
    pub fn skip_bytes(&mut self, n: usize) {
        self.0 = &self.0[n..];
    }
    pub fn next(&mut self) -> u8 {
        let o = self.0[0];
        self.0 = &self.0[1..];
        o
    }
    pub fn take<const T: usize>(&mut self) -> R<[u8; T]> {
        let o = TryInto::<[u8; T]>::try_into(&self.0[..T]).map_err(|s| s.to_string())?;
        self.0 = &self.0[T..];
        Ok(o)
    }
    pub fn take_u16(&mut self) -> u16 {
        self.decode::<u16>()
    }
    pub fn take_varint(&mut self) -> u64 {
        let size = self.next();
        match size {
            251 => self.next() as u64,
            252 => u16::from_le_bytes(self.take().unwrap()) as u64,
            253 => u32::from_le_bytes(self.take().unwrap()) as u64,
            254 => u64::from_le_bytes(self.take().unwrap()) as u64,
            255 => u128::from_le_bytes(self.take().unwrap()) as u64,
            _ => size as u64
        }
    }
    pub fn decode<T: Decode>(&mut self) -> T {
        T::rd_decode(&mut self.0).expect("ParserBuffer.decode")
    }
}

