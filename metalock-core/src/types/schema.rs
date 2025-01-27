use solana_program::pubkey::Pubkey;

use super::tags::*;
use super::core::*;
use super::parse::*;


/*
 * Schema Type
 */

pub trait SchemaType: Clone + 'static + std::fmt::Debug {
    fn to_schema() -> Schema {
        let mut buf = Vec::<u8>::new();
        Self::encode_schema(&mut buf);
        Schema(buf)
    }
    fn encode_schema(out: &mut Vec<u8>);
}


macro_rules! schematype {
    ($([$($param:ident),*])?, $type:tt, $val:ty) => {
        impl$(<$($param: SchemaType),*>)? SchemaType for $type$(<$($param),*>)? {
            fn encode_schema(out: &mut Vec<u8>) {
                out.push(<$val>::ID);
                $($($param::encode_schema(out);)*)?
            }
        }
    };
}

schematype!(, (), tag::UNIT);
schematype!(, u8, tag::U8);
schematype!(, u16, tag::U16);
schematype!(, u32, tag::U32);
schematype!(, u64, tag::U64);
schematype!(, u128, tag::U128);
schematype!(, bool, tag::BOOL);
schematype!(, String, tag::STRING);
schematype!(, Buffer, tag::BUFFER);
schematype!(, [u8; 32], tag::BUF32);
schematype!(, Pubkey, tag::BUF32);
schematype!([T], Option, tag::OPTION);
schematype!([T], Vec,    tag::LIST);


pub trait TupleType: SchemaType {}

macro_rules! tuple_types {
    ($a:ident) => {};
    ($a:ident, $($t:ident),+) => {
        tuple_types!($($t),+);
        impl<$a: SchemaType, $($t: SchemaType),+> SchemaType for ($a, $($t),+) {
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






trait Parseable {
    type Parser;
    fn parse(buf: ParserBuffer) -> Self::Parser;
}

impl Parseable for Schema {
    type Parser = tag::SchemaTagParser;
    fn parse(mut buf: ParserBuffer) -> Self::Parser {
        let tag_id = buf.next();
        unsafe { std::mem::transmute((tag_id, buf)) }
    }
}




//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn test_validate_simple() {
//        let r = validate_resource_data(&"", &RS::U8, &RD::U8(1));
//        assert!(r == Ok(()));
//    }
//
//    #[test]
//    fn test_validate_error() {
//        let r = validate_resource_data(
//            &"root",
//            &RS::List(RS::U16.into()),
//            &RD::List(vec![RD::U16(1).into(), RD::U8(1).into()]),
//        );
//        assert!(r == Err("root: 1: Schema mismatch".to_string()));
//    }
//
//    #[test]
//    fn test_validate_struct() {
//        let schema = ResourceSchema::new_struct(vec![
//            ("age", RS::U8),
//            ("name", RS::String),
//        ]);

//        fn item<T, V: Into<T>>(k: &str, v: V) -> (String, T) {
//            (k.into(), v.into())
//        }
//
//        let data = RD::Struct(vec![
//            item("age", RD::U8(30)),
//            item("name", RD::String("abc".into())),
//        ]);
//
//        let r = validate_resource_data(&"root", &schema, &data);
//        //println!("{:?}", r);
//        assert!(Ok(()) == r);
//    }
//
//    #[test]
//    fn test_serialize_schema() {
//        let schema = ResourceSchema::new_struct(vec![
//            ("age", RS::U8),
//            ("name", RS::String),
//        ]);
//
//        //println!("Debug: {}", schema.to_string());
//        //println!("Debug: {}", schema.to_string().len());
//        //println!("borsh: {:?}", schema.try_to_vec().unwrap());
//        //println!("borsh: {:?}", schema.try_to_vec().unwrap().len());
//        let buf = schema.encode();
//        //println!("encode: {:?}", buf);
//        //println!("encode: {:?}", buf.len());
//
//        let r = ResourceSchema::decode(&buf).unwrap();
//        //println!("Debug: {}", r.to_string());
//
//        assert!(schema == r);
//    }
//
//    #[test]
//    fn test_superset() {
//        let fields = vec![
//            ("age", RS::U8),
//            ("name", RS::String),
//        ];
//        let old = RS::List(ResourceSchema::new_struct(fields.clone()[1..].to_vec()).into());
//        let new = RS::List(ResourceSchema::new_struct(fields.clone()).into());
//
//        assert!(schema_is_superset(&old, &old));
//        assert!(schema_is_superset(&old, &new));
//        assert!(!schema_is_superset(&new, &old));
//    }
//
//    use quickcheck::{Arbitrary, Gen};
//
//    impl Arbitrary for ResourceSchema {
//        fn arbitrary(g: &mut Gen) -> ResourceSchema {
//            let v = Vec::from_iter(0..14);
//            let idx = g.choose(&v).unwrap();
//            match idx {
//                0 => RS::U8,
//                1 => RS::U16,
//                2 => RS::U32,
//                3 => RS::U64,
//                4 => RS::U128,
//                5 => RS::Bool,
//                6 => RS::String,
//                7 => RS::Buffer,
//                8 => RS::Pubkey,
//                9 => RS::Option(ResourceSchema::arbitrary(g).into()),
//                10 => RS::List(ResourceSchema::arbitrary(g).into()),
//                _ => {
//                    let n = *g.choose(&[1,2,3,4,5]).unwrap();
//                    RS::Struct(
//                        (0..n).map(|i| (i.to_string(), ResourceSchema::arbitrary(g))).collect()
//                    )
//                },
//            }
//        }
//    }
//
//    #[quickcheck]
//    fn identity_is_superset(schema: ResourceSchema) -> bool {
//        schema_is_superset(&schema, &schema)
//    }
//
//    #[quickcheck]
//    fn encoding_identity(schema: ResourceSchema) -> bool {
//        let encoded = schema.rd_encode();
//        //println!("{:?}: {:?}", encoded, schema);
//        schema == ResourceSchema::rd_decode(&mut &*encoded).unwrap()
//    }
//}
