
#[cfg(feature = "anchor")]
pub(crate) mod anchor;

use std::{marker::PhantomData, ops::{Deref, DerefMut}};

use crate::{expr::EncodeContext, newval::SchemaType, parse::R};

mod utils;
mod varint;
mod data2;
pub mod tlist;

pub use tlist::*;
pub use data2::*;
pub use utils::*;

pub(crate) type Buf<'a, 'b> = &'a mut &'b [u8];

pub type ResourceData = RD;


anchor_derive!(
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Schema(pub Vec<u8>);
);



pub trait Decode: Sized {
    fn rd_decode(buf: Buf) -> std::result::Result<Self, String>;
}



#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;

anchor_derive!(
    #[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Default)]
    pub struct Buffer(pub Vec<u8>);
);
impl_deref!([], Buffer, Vec<u8>, 0);

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct EncodedFunction(pub u16, pub Vec<u8>);
impl_deref!([], EncodedFunction, Vec<u8>, 1);


#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct VarId<I>(usize, PhantomData<I>);
impl<I> Deref for VarId<I> {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.0 as *const u16) }
    }
}
impl<I> DerefMut for VarId<I> {
    fn deref_mut(&mut self) -> &mut u16 {
        unsafe { &mut *(self.0 as *const u16 as *mut u16) }
    }
}
impl<I> VarId<I> {
    pub fn new() -> Self {
        Self::from(u16::MAX)
    }
    pub fn from(var_id: u16) -> VarId<I> {
        let ptr = Box::leak(Box::new(var_id)) as *mut u16 as *mut () as usize;
        VarId(ptr, PhantomData::default())
    }
    pub fn populate(&mut self, ctx: &mut EncodeContext) {
        if **self == u16::MAX {
            **self = ctx.next();
        }
    }
}







pub type Parser<'a> = &'a mut ParserBuffer;


#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ParserBuffer(pub &'static [u8]);
impl_deref!([], ParserBuffer, &'static [u8], 0);
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

