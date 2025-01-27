use std::marker::PhantomData;

/*
 * Type level cons list
 */


pub trait TList {}
impl TList for () {}
impl<A, T: TList> TList for TCons<A, T> {}


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TCons<A, T: TList>(PhantomData<(A, T)>);


#[macro_export]
macro_rules! tlist {
    () => { () };
    ($A:ty $(,$tok:ty)*) => {
        TCons<$A, tlist!($($tok),*)>
    };
}
pub use crate::tlist;
