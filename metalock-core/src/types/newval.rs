
use super::encode::Encode;
use super::{data::*, decode::*, parse::*, core::*, tags::*};
use super::macros::{impl_deref, impl_into};



impl Decode for RD {
    fn rd_decode(buf: Buf) -> R<RD> {
        data_parse(&mut ParserBuffer::new(buf))
    }
}


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
            tag::BUF32::ID => {
                let p = (&data.take::<32>()?) as *const [u8; 32];
                p.into()
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
    use crate::types::schema::*;

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




