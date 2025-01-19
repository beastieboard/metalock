
macro_rules! byte_ref {
    ($val:expr, $size:expr) => {
        unsafe { &*(std::ptr::addr_of!($val) as *const [u8; $size]) }
    };
}

pub(crate) use byte_ref;

macro_rules! impl_deref_const {
    ( [$($impl_generics:tt)*], $type:ty, $target:ty, $field:tt) => {
        impl<$($impl_generics)*> std::ops::Deref for $type {
            type Target = $target;
            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }
    }
}
pub(crate) use impl_deref_const;

macro_rules! impl_deref {
    ( [$($impl_generics:tt)*], $type:ty, $target:ty, $field:tt) => {
        $crate::types::impl_deref_const!([$($impl_generics)*], $type, $target, $field);

        impl<$($impl_generics)*> std::ops::DerefMut for $type {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    };
}
pub(crate) use impl_deref;


macro_rules! impl_into {
   ([$($impl_generics:tt)*], $into:ty, $for:ty, |$self:ident| $expr:expr) => {
        impl<$($impl_generics)*> Into<$into> for $for {
            fn into($self) -> $into {
                $expr
            }
        }
   }
}
pub(crate) use impl_into;

macro_rules! each_field {
    (|$f:path|) => { };
    (|$f:path| $a:tt) => { $f!(0, $a) };
    (|$f:path| $a:tt, $b:tt) => { $f!(0, $a); $f!(1, $b); };
    (|$f:path| $a:tt, $b:tt, $c:tt) => { $f!(0, $a); $f!(1, $b); $f!(2, $c); };
    (|$f:path| $a:tt, $b:tt, $c:tt, $d:tt) => { $f!(0, $a); $f!(1, $b); $f!(2, $c); $f!(3, $d); };
}
pub(crate) use each_field;

macro_rules! anchor_derive {
    (#[derive($($trait:ident),*)] $item:item) => {
        #[cfg(feature = "anchor")]
        #[derive($($trait,)* AnchorSerialize, AnchorDeserialize)]
        $item

        #[cfg(not(feature = "anchor"))]
        #[derive($($trait),*)]
        $item
    };
}
pub(crate) use anchor_derive;
