use crate::types::{ParserBuffer, Schema};


pub mod tag {
    use std::marker::PhantomData;

    use crate::types::{ParserBuffer, Schema};
    use crate::tlist::*;

    pub trait TagType {
        const ID: u8;
        type Schema: TList;
    }

    pub struct Tag<const ID: u8, T: TList>([T;0]);
    impl<const ID: u8, T: TList> TagType for Tag<ID, T> {
        const ID: u8 = ID;
        type Schema = T;
    }

    pub struct SchemaFieldParser<T: TList>(pub ParserBuffer, pub PhantomData<T>);

    macro_rules! define_tags {
        ($($name:ident $id:literal $([$($t:ty),*])?),*) => {
            $( pub type $name = Tag<$id, tlist!($($($t),*)?)>; )*

            #[repr(u8)]
            pub enum SchemaTagParser {
                $($name$((SchemaFieldParser<tlist!($($t),*)>))? = $id),*
            }
        };
    }

    define_tags!(
        UNIT 0,
        U8 1,
        U16 2,
        U32 3,
        U64 4,
        U128 5,
        BOOL 6,
        STRING 7,
        BUFFER 8,
        BUF32 9,
        OPTION 10 [Schema],
        LIST 11 [Schema],
        TUPLE 12 [Vec<Schema>],
        RSTRUCT 14 [u16, Vec<Schema>],
        NATIVE 15,
        REF 16,
        FUNCTION 17
    );
}



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
