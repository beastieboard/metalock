use std::{marker::PhantomData, ops::Deref};

#[cfg(feature = "anchor")]
use anchor_lang::prelude::Pubkey;

use crate::{native::Native, expr::Function, newval::SchemaType, schema::tag::*, types::impl_into};

use super::{Buffer, EncodedFunction, Schema};





#[derive(Clone, Debug)]
pub struct PackedPtr<T: 'static>([u16;3], PhantomData<T>);
impl<T> PackedPtr<T> {
    fn as_usize(&self) -> usize {
        unsafe {
            std::mem::transmute([self.0[0], self.0[1], self.0[2], 0])
        }
    }
}
impl<T: PartialEq> PartialEq for PackedPtr<T> {
    fn eq(&self, other: &Self) -> bool { **self == **other }
}
impl<T: Eq> Eq for PackedPtr<T> {}
impl<T: 'static> Deref for PackedPtr<T> {
    type Target = T;

    fn deref(&self) -> &'static Self::Target {
        let r = [self.0[0], self.0[1], self.0[2], 0];
        unsafe { std::mem::transmute(r) }
    }
}

#[derive(Clone, Debug)]
pub struct OptPackedPtr<T: 'static>(PackedPtr<T>);
impl<T> OptPackedPtr<T> {
    pub fn null() -> Self {
        OptPackedPtr(PackedPtr([0, 0, 0], Default::default()))
    }
    pub fn as_ref(&self) -> Option<&'static T> {
        let s = self.0.as_usize();
        if s > 0 {
            Some(unsafe { std::mem::transmute(s) })
        } else {
            None
        }
    }
}
impl<T: PartialEq> PartialEq for OptPackedPtr<T> {
    fn eq(&self, other: &Self) -> bool { self.as_ref() == other.as_ref() }
}
impl<T: Eq> Eq for OptPackedPtr<T> {}



#[derive(Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum RD {
    Unit()                                        = UNIT::ID,
    U8(u8)                                        = U8::ID,
    U16(u16)                                      = U16::ID,
    U32(u32)                                      = U32::ID,
    U64(PackedPtr<u64>)                           = U64::ID,
    U128(PackedPtr<u128>)                         = U128::ID,
    Bool(bool)                                    = BOOL::ID,
    String(PackedPtr<String>)                     = STRING::ID,
    Buffer(PackedPtr<Buffer>)                     = BUFFER::ID,
    Buf32(PackedPtr<[u8; 32]>)                    = BUF32::ID,
    Option(OptPackedPtr<RD>)                      = OPTION::ID,
    List(PackedPtr<Vec<RD>>)                      = LIST::ID,
    Tuple(PackedPtr<Vec<RD>>)                     = TUPLE::ID,
    Native(PackedPtr<Native>)                     = NATIVE::ID,
    Function(PackedPtr<EncodedFunction>)          = FUNCTION::ID,
}


impl RD {
    pub fn _as<T: FromRD>(&self) -> T {
        T::from_rd3(self)
    }
    pub fn tag(&self) -> u8 {
        unsafe { *(self as *const RD as *const u8) }
    }
    pub fn none() -> Self {
        RD::Option(OptPackedPtr::null())
    }
}

const _: () = assert!(std::mem::size_of::<RD>() == 8);


macro_rules! impl_into_rd3 {
    ($profile:ident, [$($param:ident$(: $tr:path)?),*], $into:ty, $for:ty, |$self:ident| $expr:expr) => {
        impl_into!([$($param$(: $tr)?),*], $into, $for, |$self| {
            crate::profile_wrap!($profile, {
                $expr
            })
        });
    };
    ([$($param:ident$(: $tr:path)?),*], $into:ty, $for:ty, |$self:ident| $expr:expr) => {
        impl_into_rd3!(IntoRd, [$($param$(: $tr)?),*], $into, $for, |$self| $expr);
    };
}

impl_into_rd3!([], RD, (),      |self| RD::Unit());
impl_into_rd3!([], RD, bool,    |self| RD::Bool(self));
impl_into_rd3!([], RD, u8,      |self| RD::U8(self));
impl_into_rd3!([], RD, u16,     |self| RD::U16(self));
impl_into_rd3!([], RD, u32,     |self| RD::U32(self));
impl_into_rd3!(IntoRdU64, [], RD, u64,     |self| pp(U64::ID, 0, self));
impl_into_rd3!(IntoRd128, [], RD, u128,    |self| pp(U128::ID, 0, self));
impl_into_rd3!(IntoRdString, [], RD, String,  |self| pp(STRING::ID, 0, self));
impl_into_rd3!(IntoRdString, [], RD, &str,  |self| pp(STRING::ID, 0, self.to_string()));

#[cfg(feature = "anchor")]
impl_into_rd3!(IntoRdPubkey, [], RD, Pubkey,  |self| pp(BUF32::ID, 0, self));

impl_into_rd3!(IntoRdBuf32, [], RD, [u8; 32], |self| pp(BUF32::ID, 0, self));
impl_into_rd3!(IntoRdBuffer, [], RD, Buffer,  |self| pp(BUFFER::ID, 0, self));
impl_into_rd3!(IntoRdNative, [], RD, Native, |self| pp(NATIVE::ID, 0, self));
impl_into_rd3!(IntoRdOption, [T: Into<RD>], RD, Option<T>, |self| match self {
    Some(o) => pp(OPTION::ID, 0, Into::<RD>::into(o)),
    None => RD::none()
});
impl_into_rd3!(IntoRdVec, [T: Into<RD>], RD, Vec<T>,
    |self| pp(LIST::ID, 0, self.into_iter().map(Into::<RD>::into).collect::<Vec<_>>())
);

impl_into_rd3!([], RD, EncodedFunction, |self| pp(FUNCTION::ID, 0, self));

fn pp<T>(discriminant: u8, extra: u8, t: T) -> RD {
    unsafe {
        let mut p: usize = Box::into_raw(Box::new(t)) as usize;
        assert!(p >> 48 == 0, "ptr doesnt have 16 free bits");
        p <<= 16;
        p |= discriminant as usize;
        p |= (extra as usize) << 8;
        std::mem::transmute(p)
    }
}


pub trait FromRD {
    fn from_rd3(rd: &RD) -> Self;
}

macro_rules! impl_from_rd3 {
    ([$($a0:ident),*], $type:ty, $pattern:tt => $expr:expr) => {
        impl<$($a0: FromRD),*> FromRD for $type {
            fn from_rd3(rd: &RD) -> $type {
                match rd {
                    #[allow(unused_parens)]
                    $pattern => $expr,
                    _ => panic!("from_rd3: unexpected")
                }
            }
        }
    };
    ([$($a0:ident),*], $type:ty, $tag:ty, |$p:ident, $e:ident| $body: expr) => {
        impl<$($a0: FromRD),*> FromRD for $type {
            fn from_rd3(rd: &RD) -> $type {
                let w: &usize = unsafe { std::mem::transmute(rd) };
                if *w as u8 == <$tag>::ID {
                    let $e = (w >> 8) as u8;
                    unsafe {
                        let $p = &*((w >> 16) as *const _);
                        $body
                    }
                } else {
                    panic!("RD convert fail")
                }
            }
        }
    };
}

impl_from_rd3!([], (), (RD::Unit()) => ());
impl_from_rd3!([], u8, (RD::U8(b)) => *b);
impl_from_rd3!([], u16, (RD::U16(b)) => *b);
impl_from_rd3!([], u32, (RD::U32(b)) => *b);
impl_from_rd3!([], u64, (RD::U64(b)) => **b);
impl_from_rd3!([], u128, (RD::U128(b)) => **b);
impl_from_rd3!([], bool, (RD::Bool(b)) => *b);
impl_from_rd3!([], &'static String, STRING, |p, _e| p);
impl_from_rd3!([], &'static Buffer, BUFFER, |p, _e| p);
//impl_from_rd3!([], &Vec<RD>, LIST, |p, _e| p);
impl_from_rd3!([T], Option<T>, (RD::Option(p)) => p.as_ref().map(T::from_rd3));
impl_from_rd3!([], &'static EncodedFunction, FUNCTION, |p, _e| p);
impl_from_rd3!([], &'static Native, NATIVE, |p, _e| p);




impl<A: Into<RD>> FromIterator<A> for RD {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        iter.into_iter().map(Into::into).collect::<Vec<RD>>().into()
    }
}


macro_rules! impl_rd_tuple {
    ($a:ident) => {};
    ($a:ident, $($t:ident),+) => {
        impl_rd_tuple!($($t),+);
        impl<$a: Into<RD>, $($t: Into<RD>),+> Into<RD> for ($a, $($t),+) {
            fn into(self) -> RD {
                #[allow(non_snake_case)]
                let ($a, $($t),+) = self;
                let v: Vec<RD> = vec![$a.into(), $($t.into()),+];
                pp(TUPLE::ID, 0, Some(v))
            }
        }
    };
}
impl_rd_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);






#[cfg(test)]
mod tests {
    use quickcheck::Arbitrary;

    use super::*;

    macro_rules! test_rd3 {
        ($func:ident, $type:ty) => {
            #[quickcheck]
            fn $func(b: $type) {
                assert_eq!(<$type>::from_rd3(&b.into()), b);
            }
        };
    }

    macro_rules! test_rd3_ref {
        ($func:ident, $type:ty) => {
            #[quickcheck]
            fn $func(b: $type) {
                assert_eq!(<&$type>::from_rd3(&b.clone().into()), &b);
            }
        };
    }

    test_rd3!(test_unit, ());
    test_rd3!(test_bool, bool);
    test_rd3!(test_u8, u8);
    test_rd3!(test_u16, u16);
    test_rd3!(test_u32, u32);
    test_rd3!(test_u64, u64);
    test_rd3!(test_u128, u128);
    test_rd3_ref!(test_string, String);
    test_rd3_ref!(test_buffer, Buffer);
    //test_rd3_ref!(test_vec, Vec<bool>);

    impl Arbitrary for Buffer {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Buffer(Arbitrary::arbitrary(g))
        }
    }
}
