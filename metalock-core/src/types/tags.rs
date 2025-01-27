use crate::types::parse::ParserBuffer;
use crate::types::tlist::*;
use crate::types::core::*;


pub trait TagType {
    const ID: u8;
    type Schema: TList;
}



pub struct Tag<const ID: u8, T: TList>([T;0]);
impl<const ID: u8, T: TList> TagType for Tag<ID, T> {
    const ID: u8 = ID;
    type Schema = T;
}

pub struct SchemaFieldParser<T: TList>(pub ParserBuffer, pub std::marker::PhantomData<T>);

pub mod tag {
    use super::*;

    macro_rules! define_tags {
        ($($id:literal $name:ident $([$($t:ty),*])?),*) => {
            $( pub type $name = Tag<$id, tlist!($($($t),*)?)>; )*

            #[repr(u8)]
            pub enum SchemaTagParser {
                $($name$((SchemaFieldParser<tlist!($($t),*)>))? = $id),*
            }
        };
    }

    define_tags!(
         0 UNIT,
         1 U8,
         2 U16,
         3 U32,
         4 U64,
         5 U128,
         6 BOOL,
         7 STRING,
         8 BUFFER,
         9 BUF32,
        10 OPTION  [Schema],
        11 LIST    [Schema],
        12 TUPLE   [Vec<Schema>],
        14 RSTRUCT [u16, Vec<Schema>],
        15 NATIVE,
        16 REF,
        17 FUNCTION
    );

}

