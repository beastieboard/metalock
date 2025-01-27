
use super::macros::*;



#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EncodedFunction(pub u16, pub Vec<u8>);
impl_deref!([], EncodedFunction => Vec<u8>, 1);

anchor_derive!(
    #[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Default)]
    pub struct Buffer(pub Vec<u8>);
);
impl_deref!([], Buffer => Vec<u8>, 0);


anchor_derive!(
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Schema(pub Vec<u8>);
);
