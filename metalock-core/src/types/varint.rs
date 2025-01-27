use crate::types::impl_deref;



#[repr(transparent)]
pub struct Varint(u64);
impl_deref!([], Varint, u64, 0);

pub trait VarintType {
    fn varint(&self) -> Vec<u8>;
}

impl VarintType for u8 {
    #[inline]
    fn varint(&self) -> Vec<u8> {
        if *self <= 250 { vec![*self] } else { vec![251, *self] }
    }
}

macro_rules! impl_varint {
    ($code:expr, $t:ty) => {};
    ($code:expr, $t:ty, $prev:ty $(,$rest:ty)*) => {
        impl_varint!($code-1, $prev $(,$rest)*);
        impl VarintType for $t {
            #[inline]
            fn varint(&self) -> Vec<u8> {
                if *self <= (<$prev>::MAX as $t) {
                    (*self as $prev).varint()
                } else {
                    let mut v = vec![$code];
                    v.extend(self.to_le_bytes());
                    v
                }
            }
        }
    };
}

impl_varint!(255, u128, u64, u32, u16, u8);

impl VarintType for usize {
    fn varint(&self) -> Vec<u8> {
        (*self as u64).varint()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        assert!(    250u8.varint() == vec![250]);
        assert!(  u8::MAX.varint() == vec![251, 255]);
        assert!( u16::MAX.varint() == vec![252, 255, 255]);
        assert!( u32::MAX.varint() == vec![253, 255, 255, 255, 255]);
        assert!( u64::MAX.varint() == vec![254, 255, 255, 255, 255, 255, 255, 255, 255]);
        assert!(u128::MAX.varint() == vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255]);
    }
}

