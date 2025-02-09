
use super::core::*;
use super::data::*;
use super::schema::SchemaType;


pub trait Encode: Sized {
    fn rd_encode(&self) -> Vec<u8>;
}
impl<A: Encode> Encode for &A {
    fn rd_encode(&self) -> Vec<u8> {
        (*self).rd_encode()
    }
}
//pub fn rd_encode_with_schema<A: Encode + SchemaType>(a: A) -> Vec<u8> {
//}

macro_rules! impl_serialize_any {
    ([$($params:ident),*], $type:ty, |$self:ident| $body:expr) => {
        impl<$($params: Encode),*> Encode for $type {
            fn rd_encode(&$self) -> Vec<u8> { $body }
        }
    };
}
macro_rules! impl_serialize_int {
    ($type:tt) => { impl_serialize_any!([], $type, |self| self.to_le_bytes().to_vec()); };
}

impl_serialize_int!(u8);
impl_serialize_int!(u16);
impl_serialize_int!(u32);
impl_serialize_int!(u64);
impl_serialize_int!(u128);
impl_serialize_any!([], bool, |self| (*self as u8).rd_encode());
impl_serialize_any!([], [u8; 32], |self| self.as_ref().to_vec());
impl_serialize_any!([], String, |self| Buffer(self.as_bytes().to_vec()).rd_encode());
impl_serialize_any!([], (), |self| vec![]);

impl_serialize_any!([], Buffer, |self| {
    let mut out = (self.len() as u16).rd_encode();
    out.extend(&self.0);
    out
});
impl_serialize_any!([], EncodedFunction, |self| {
    let mut out = self.0.rd_encode();
    out.extend((self.1.len() as u16).rd_encode());
    out.extend(&self.1);
    out
});

impl_serialize_any!([I], Vec<I>, |self| rd_iterator(self.len(), self.iter()));
impl_serialize_any!([A], Box<A>, |self| self.as_ref().rd_encode());
impl_serialize_any!([A], Option<A>, |self| {
    let mut out = self.is_some().rd_encode();
    if let Some(o) = self {
        out.extend(o.rd_encode());
    }
    out
});


fn rd_iterator<T: Encode, I: Iterator<Item=T>>(len: usize, iterator: I) -> Vec<u8> {
    let mut out = (len as u16).rd_encode();
    iterator.for_each(|i| out.extend(i.rd_encode()));
    out
}


impl_serialize_any!([], RD, |self| {
    match self {
        RD::Unit() => vec![],
        RD::U8(u) => u.rd_encode(),
        RD::U16(u) => u.rd_encode(),
        RD::U32(u) => u.rd_encode(),
        RD::U64(u) => u.rd_encode(),
        RD::U128(u) => u.rd_encode(),
        RD::Bool(b) => b.rd_encode(),
        RD::String(s) => s.rd_encode(),
        RD::Buffer(v) => v.rd_encode(),
        RD::Buf32(p) => p.rd_encode(),
        RD::Option(o) => o.as_ref().rd_encode(),
        RD::List(v) => v.rd_encode(),
        RD::Tuple(v) => v.rd_encode(),
        RD::Function(f) => f.rd_encode(),
        RD::Native(_) => panic!("no serialize for native"),
    }
});



macro_rules! tuple_encode {
    ($a:ident) => {};
    ($a:ident, $($t:ident),+) => {
        tuple_encode!($($t),+);
        impl_serialize_any!([$a, $($t),+], ($a,$($t),+), |self| {
            let mut v = vec![];
            #[allow(non_snake_case)]
            let ($a, $($t),+) = self;
            v.extend($a.rd_encode());
            $(v.extend($t.rd_encode()));*;
            v
        });
    };
}

tuple_encode!(A, B, C, D, E, F, G);



