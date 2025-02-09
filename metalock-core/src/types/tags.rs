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
         2 U8,                        
         4 U16,                       
         6 U32,                       
         8 U64,                       
        10 U128,                      
        12 BOOL,                      
        14 STRING,                    
        16 BUFFER,                    
        18 BUF32,                     
        20 OPTION  [Schema],          
        22 LIST    [Schema],          
        24 TUPLE   [Vec<Schema>],     
        28 RSTRUCT [u16, Vec<Schema>],
        30 NATIVE,                    
        32 REF,                       
        34 FUNCTION                   
    );

}

