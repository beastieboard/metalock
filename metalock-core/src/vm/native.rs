
use crate::types::core::*;
use crate::types::native::*;
use crate::types::data::*;
use crate::types::tags::*;




impl Native {

    pub fn deref<S: NativeData>(&self) -> &'static S {
        assert!(S::to_schema() == self.0, "Native deref: wrong schema");
        unsafe { &*(self.1 as *const S) }
    }

    pub fn from<S: NativeData>(s: S) -> Native {
        let r = Box::leak(Box::new(s));
        Native(S::to_schema(), r as *mut S as *const u8)
    }
    pub fn from_ptr<S: NativeData>(s: *const S) -> Native {
        Native(S::to_schema(), s as *const u8)
    }

    pub fn get_struct_field(&self, field_idx: u8, offset: u32) -> RD {
        let (_size, _, mut fields) = self.0.parser().rstruct();
        fields.skip_schema(field_idx as usize);
        let p = self.1 as *const u8;
        let p = unsafe { p.add(offset as usize) };
        match fields[0] {
            tag::BOOL::ID => RD::Bool(unsafe { *(p as *const _) }),
            tag::U32::ID => RD::U32(unsafe { *(p as *const _) }),
            tag::BUF32::ID => unsafe { *(p as *const [u8; 32]) }.into(),
            tag::LIST::ID => Native(fields.seal(), p).into(), // probably wrong
            tag::BUFFER::ID => unsafe { &*(p as *const Buffer) }.clone().into(),
            _ => panic!("GET_STRUCT_FIELD: Ptr")
        }
    }

    pub fn set_struct_field(&self, offset: u32, val: RD) -> Native {
        unsafe {
            // copy the struct
            let struct_size = self.0.parser().rstruct().0;
            let ptr = std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(struct_size, 8));
            std::ptr::copy_nonoverlapping(self.1 as *const u8, ptr, struct_size);

            // point to field
            let fptr = ptr.add(offset as usize) as *const ();

            // update
            match val {
                RD::U8(b)  => { *(fptr as *mut _) = b; },
                RD::U32(b) => { *(fptr as *mut _) = b; },
                RD::Bool(b) => { *(fptr as *mut _) = b; },
                RD::Buffer(b) => { *(fptr as *mut _) = b; },
                RD::Buf32(p) => { *(fptr as *mut _) = *p; },
                o => panic!("SET_STRUCT_FIELD: Ptr: {:?}", o)
            }

            Native(self.0.clone(), ptr)
        }
    }

    pub fn index(&self, idx: usize) -> Native {
        let rstruct = self.0.parser().list();
        let size = rstruct.clone().rstruct().0;
        let v = unsafe { &*(self.1 as *const () as *const Vec<u8>)};
        if idx >= v.len() {
            panic!("Vec idx oob");
        }
        let p = v.as_ptr();
        let off = size * idx;
        let p = unsafe { p.add(off) };
        Native(rstruct.into(), p)
    }

    pub fn iter(&self) -> impl Iterator<Item=RD> + '_ {
        let parser = self.0.parser().list();
        let v = unsafe { &*(self.1 as *const Vec<u8>)};
        let len = v.len();
        let size = match parser[0] {
            tag::RSTRUCT::ID => parser.clone().rstruct().0,
            o => panic!("Native::iter for: {}", o)
        };
        let p = v.as_ptr();
        let schema = parser.seal();
        (0..len).map(move |idx| {
            let off = size * idx as usize;
            let p = unsafe { p.add(off) };
            Native(schema.clone(), p).into()
        })
    }

    pub fn slice(&self, idx: usize) -> Native {
        let parser = self.0.parser().list();
        let v = unsafe { &*(self.1 as *const Vec<u8>)};
        assert!(idx < v.len(), "Native::slice: oob");
        let size = match parser[0] {
            tag::RSTRUCT::ID => parser.clone().rstruct().0,
            o => panic!("Native::iter for: {}", o)
        };
        let off = size * idx as usize;
        let dv: [usize;3] = [v.as_ptr() as usize + off, 0, v.len()-idx];
        let v = Box::into_raw(Box::new(dv));
        Native(self.0.clone(), v as *const u8)
    }
}


//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::types::tlist::*;
//    //use crate::program::*;
//
//    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
//    struct MyStruct { a: u32, b: u8, c: [u8; 32] }
//    rr_native_struct!(MyStruct { a 0: u32, b 1: u8, c 2: [u8; 32] });
//
//
//    #[test]
//    pub fn test_set_field() {
//        let p = [0u8;32];
//        let p2 = [1u8;32];
//        let mk = |a, b, c| MyStruct { a, b, c };
//        let run = |prog: Program<_, _>| prog.run(mk(0, 0, p), Default::default())._as::<&'static Native>().deref::<MyStruct>();
//        let prog = Program::from(|s: RR<MyStruct>| s.set_a(1));
//        assert!(run(prog) == &mk(1, 0, p));
//
//        let prog = Program::from(|s: RR<MyStruct>| s.set_b(1));
//        assert!(run(prog) == &mk(0, 1, p));
//
//        let prog = Program::from(|s: RR<MyStruct>| s.set_c(p2));
//        assert_eq!(run(prog), &mk(0, 0, p2));
//
//        let prog = Program::from(|s: RR<MyStruct>| s.set_c(p2).set_a(1));
//        assert_eq!(run(prog), &mk(1, 0, p2));
//    }
//}




