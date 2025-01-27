
#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;

use solana_program::pubkey::Pubkey;

use super::core::{Buffer, EncodedFunction};



pub type Buf<'a, 'b> = &'a mut &'b [u8];


pub trait Decode: Sized {
    fn rd_decode(buf: Buf) -> std::result::Result<Self, String>;
}


pub(crate) type R<T> = std::result::Result<T, String>;



#[inline]
fn many<I: IntoIterator, T, F: FnMut(&I::Item) -> R<T>>(n: I, mut f: F) -> R<Vec<T>> {
    n.into_iter().map(|a| f(&a)).collect::<R<Vec<_>>>()
}

pub fn take<const T: usize>(buf: Buf) -> R<[u8; T]> {
    let o = TryInto::<[u8; T]>::try_into(&buf[..T]).map_err(|s| s.to_string())?;
    *buf = &buf[T..];
    Ok(o)
}

#[inline]
pub fn rdd<T: Decode>(buf: Buf) -> R<T> {
    Decode::rd_decode(buf)
}


macro_rules! impl_deserialize_any {
    ($type:tt$(<$param:ident>)?, |$buf:ident| $process:expr) => {
        impl<$($param: Decode)?> Decode for $type$(<$param>)? {
            fn rd_decode($buf: Buf) -> R<$type$(<$param>)?> {
                $process
            }
        }
    }
}

#[macro_export]
macro_rules! impl_deserialize_int {
    ($type:tt) => {
        impl_deserialize_any!($type, |buf| Ok($type::from_le_bytes(take(buf)?)));
    };
}

impl_deserialize_int!(u8);
impl_deserialize_int!(u16);
impl_deserialize_int!(u32);
impl_deserialize_int!(u64);
impl_deserialize_int!(u128);
impl_deserialize_any!((), |_buf| Ok(()));
impl_deserialize_any!(bool, |buf| Ok(u8::rd_decode(buf)? > 0));
impl_deserialize_any!(Pubkey, |buf| Ok(Pubkey::from(take(buf)?)));
impl_deserialize_any!(Option<T>, |buf| Option::rd_many(buf, rdd));
impl_deserialize_any!(Vec<T>, |buf| Vec::rd_many(buf, rdd));
impl_deserialize_any!(Box<T>, |buf| Ok(Box::new(rdd(buf)?)));
impl_deserialize_any!(String, |buf| {
    let Buffer(v) = Buffer::rd_decode(buf)?;
    Ok(unsafe { String::from_utf8_unchecked(v) })
});

impl_deserialize_any!(Buffer, |buf| {
    let len: u16 = rdd(buf)?;
    let (a, rest) = buf.split_at(len as usize);
    *buf = rest;
    Ok(Buffer(a.to_vec()))
});
impl_deserialize_any!(EncodedFunction, |buf| {
    let ref_id: u16 = rdd(buf)?;
    let len: u16 = rdd(buf)?;
    let (a, rest) = buf.split_at(len as usize);
    *buf = rest;
    Ok(EncodedFunction(ref_id, a.to_vec()))
});


/*
 * Is this trait worth it?
 */
pub trait ResourceDataContainer<T>: Sized {
    fn rd_many<F: FnMut(Buf) -> R<T>>(buf: Buf, f: F) -> R<Self>;
}

impl<T> ResourceDataContainer<T> for Vec<T> {
    fn rd_many<F: FnMut(Buf) -> R<T>>(buf: Buf, mut f: F) -> R<Self> {
        many(0..u16::rd_decode(buf)?, |_| f(buf))
    }
}
impl<T> ResourceDataContainer<T> for Option<T> {
    fn rd_many<F: FnMut(Buf) -> R<T>>(buf: Buf, mut f: F) -> R<Self> {
        if u8::rd_decode(buf)? > 0 {
            Ok(Some(f(buf)?))
        } else {
            Ok(None)
        }
    }
}




macro_rules! tuple_decode {
    ($a:ident) => {};
    ($a:ident, $($t:ident),+) => {
        tuple_decode!($($t),+);
        impl<$a: Decode, $($t: Decode),+>
            Decode for ($a, $($t),+)
        {
            fn rd_decode(buf: Buf) -> std::result::Result<Self, String> {
                Ok(
                    (rdd::<$a>(buf)?, $(rdd::<$t>(buf)?),+)
                )
            }
        }
    };
}

tuple_decode!(A, B, C, D, E, F, G);



