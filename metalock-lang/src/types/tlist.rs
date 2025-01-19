use std::marker::PhantomData;


pub trait TList {}
impl TList for () {}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TCons<A, T: TList>(PhantomData<(A, T)>);
impl<A, T: TList> TList for TCons<A, T> {}

#[macro_export]
macro_rules! tlist {
    () => { () };
    ($A:ty $(,$tok:ty)*) => {
        TCons<$A, tlist!($($tok),*)>
    };
}
pub use crate::tlist;
