
use std::ops::Deref;

#[cfg(feature = "anchor")]
use anchor_lang::prelude::{AccountMeta, Pubkey, msg};

#[cfg(feature = "anchor")]
use crate::types::anchor::*;

use crate::{data::RD, macros::{impl_deref, impl_into}, parse::{rdd, R}, schema::tag::{self, TagType}, types::{Buffer, Decode, EncodedFunction, Parser, ParserBuffer, Schema}};
use crate::tlist::*;

/*
 * Schema Type
 */

pub trait SchemaType: Clone + 'static + std::fmt::Debug {
    type Items: TList;
    fn to_schema() -> Schema {
        let mut buf = Vec::<u8>::new();
        Self::encode_schema(&mut buf);
        Schema(buf)
    }
    fn encode_schema(out: &mut Vec<u8>);
}

macro_rules! schema_primitive {
    ($type:ty, $val:ty) => {
        schema_primitive!($type, $val, |out| {});
    };
    ($type:ty, $val:ty, |$out:ident| $encode:expr) => {
        impl SchemaType for $type {
            type Items = TCons<$val, ()>;
            fn encode_schema($out: &mut Vec<u8>) {
                $out.push(<$val>::ID);
                $encode;
            }
        }
    };
}

schema_primitive!((), tag::UNIT);
schema_primitive!(u8, tag::U8);
schema_primitive!(u16, tag::U16);
schema_primitive!(u32, tag::U32);
schema_primitive!(u64, tag::U64);
schema_primitive!(u128, tag::U128);
schema_primitive!(bool, tag::BOOL);
schema_primitive!(String, tag::STRING);
schema_primitive!(Buffer, tag::BUFFER);
schema_primitive!([u8; 32], tag::BUF32);
#[cfg(feature = "anchor")]
schema_primitive!(Pubkey, tag::BUF32);


macro_rules! schema_complex {
    ([$($param:ident),*], $type:tt, $val:ty) => {
        impl<$($param: SchemaType),*> SchemaType for $type<$($param),*> {
            type Items = tlist!($val, Vec<()>); // TODO: Type for tuple items?
            fn encode_schema(out: &mut Vec<u8>) {
                out.push(<$val>::ID);
                $($param::encode_schema(out);)*
            }
        }
    };
}

schema_complex!([T], Option, tag::OPTION);
schema_complex!([T], Vec,    tag::LIST);
//schema_complex!([I, O], Function, tag::FUNCTION);

#[macro_export]                                   
macro_rules! count {                              
    () => (0);                                    
    ( $x:tt $($xs:tt)* ) => (1 + count!($($xs)*));
}

pub trait TupleType: SchemaType {}

macro_rules! tuple_types {
    ($a:ident) => {};
    ($a:ident, $($t:ident),+) => {
        tuple_types!($($t),+);
        impl<$a: SchemaType, $($t: SchemaType),+> SchemaType for ($a, $($t),+) {
            type Items = tlist!(tag::TUPLE, u8, u16); // TODO
            fn encode_schema(out: &mut Vec<u8>) {
                out.push(tag::TUPLE::ID);
                let n = 1 $(+ ([] as [$t;0], 1).1)+;
                let mut v = vec![];
                $a::encode_schema(&mut v);
                $($t::encode_schema(&mut v));+;
                out.push(n as u8);
                out.extend((v.len() as u16).to_le_bytes());
                out.extend(v);
            }
        }
        impl<$a: SchemaType, $($t: SchemaType),+> TupleType for ($a, $($t),+) {}
    };
}
tuple_types!(A, B, C, D, E, F, G);

/*
 * Parsing
 */


pub fn data_parse(buf: Parser) -> R<RD> {

    fn parse_inner(schema: Parser, data: Parser) -> R<RD> {
        let tag = schema.next();
        Ok(match tag {
            tag::UNIT::ID => RD::Unit(),
            tag::U8::ID => RD::U8(rdd(data)?),
            tag::U16::ID => RD::U16(rdd(data)?),
            tag::U32::ID => RD::U32(rdd(data)?),
            tag::U64::ID => u64::rd_decode(data)?.into(),
            tag::U128::ID => u128::rd_decode(data)?.into(),
            tag::BOOL::ID => RD::Bool(rdd(data)?),
            tag::STRING::ID => rdd::<String>(data)?.into(),
            tag::BUFFER::ID => rdd::<Buffer>(data)?.into(),
            // TODO: Zero copy?
            tag::BUF32::ID => {
                let p = (&data.take::<32>()?) as *const [u8; 32];
                p.into()
                //data.take::<32>()?.into(),
            },
            tag::OPTION::ID => {
                if bool::rd_decode(data)? {
                    Some(parse_inner(schema, data)?)
                } else {
                    None
                }.into()
            },
            tag::LIST::ID => {
                let n = u16::rd_decode(data)?;
                let items = (0..n).map(|_| {
                    parse_inner(&mut schema.clone(), data)
                });
                items.collect::<R<_>>()?
            },
            tag::TUPLE::ID => {
                let n = schema.next();
                u16::rd_decode(schema)?;
                //let items = (0..n).map(|_| parse_inner(schema, data));
                panic!("tuple") // RD::Tuple(items.collect::<R<Vec<_>>>()?)
            },
            tag::RSTRUCT::ID => panic!("no parse for native"),
            tag::FUNCTION::ID => {
                rdd::<EncodedFunction>(data)?.into()
            },
            o => panic!("data_parse: {}", o)
        })
    }

    let len = buf.take_u16();
    let schema = &mut buf.clone();
    buf.skip_bytes(len as usize);
    parse_inner(schema, buf)
}


pub fn schema_is_superset(subset: Parser, superset: Parser) -> bool {
    match (subset.next(), superset.next()) {
        (tag::OPTION::ID, tag::OPTION::ID) => schema_is_superset(subset, superset),
        (tag::LIST::ID, tag::LIST::ID) => schema_is_superset(subset, superset),
        (tag::TUPLE::ID, tag::TUPLE::ID) => {
            let len_a = subset.take_varint();
            subset.skip_bytes(2);
            let len_b = superset.take_varint();
            let mut sup = superset.clone();
            sup.skip_bytes(superset.take_u16() as usize);
            
            if len_b >= len_a && (0..len_a).all(|_| schema_is_superset(subset, superset)) {
                superset.set(sup);
                true
            } else {
                false
            }
        },
        _ => { return false; }
    }
}







#[derive(Clone, Copy)]
pub struct SchemaParser(pub ParserBuffer);
impl_deref!([], SchemaParser => ParserBuffer, 0);
impl SchemaParser {
    pub fn seal(self) -> Schema {
        Schema(self.0.to_vec())
    }
    pub fn list(mut self) -> SchemaParser {
        assert!(self.0.next() == tag::LIST::ID, "Expected LIST");
        self
    }
    pub fn rstruct(mut self) -> (usize, u8, SchemaParser) {
        assert!(self.0.next() == tag::RSTRUCT::ID, "Expected RSTRUCT");
        let size = u16::rd_decode(&mut self).unwrap();
        let nfields = u8::rd_decode(&mut self).unwrap();
        //let fields = (0..nfields).map(|_| self.take_schema()).collect::<Vec<_>>();
        (size as usize, nfields, self)
    }
    pub fn skip_schema(&mut self, n: usize) {
        for _ in 0..n {
            match self.0.next() {
                tag::UNIT::ID => {},
                tag::U8::ID => {},
                tag::U16::ID => {},
                tag::U32::ID => {},
                tag::U64::ID => {},
                tag::U128::ID => {},
                tag::BOOL::ID => {},
                tag::OPTION::ID => self.skip_schema(1),
                tag::STRING::ID => {},
                tag::BUFFER::ID => {},
                tag::BUF32::ID => {},
                tag::LIST::ID => self.skip_schema(1),
                tag::RSTRUCT::ID => {},
                tag::TUPLE::ID => {
                    // skip vec len
                    self.skip_bytes(1);
                    let len = self.take_u16();
                    self.skip_bytes(len as usize);
                },
                tag::FUNCTION::ID => self.skip_schema(2),
                o => panic!("Schema.skip: {}", o)
            }
        }
    }
    pub fn take_schema(&mut self) -> Schema {
        let before = self.0.0;
        self.skip_schema(1);
        let len = self.0.0.as_ptr() as usize - before.as_ptr() as usize;
        Schema(before[..len].to_vec())
    }
}
impl_into!([], Schema, SchemaParser, |self| self.seal());


impl Schema {
    pub fn parser(&self) -> SchemaParser {
        SchemaParser(ParserBuffer::new(&self.0))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let out = &mut vec![];
        u16::encode_schema(out);
        assert!(*out == vec![tag::U16::ID]);
    }

    #[test]
    fn test_struct() {
        let out = &mut vec![];
        <(bool, u16)>::encode_schema(out);
        assert!(*out == vec![tag::TUPLE::ID, 2, 2, 0, tag::BOOL::ID, tag::U16::ID]);
    }
}




