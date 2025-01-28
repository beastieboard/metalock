
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use std::usize;

use crate::types::core::*;
use crate::types::tags::*;
use crate::types::tlist::*;
use crate::types::decode::*;
use crate::types::encode::*;
use crate::types::schema::*;
use crate::types::data::*;
use crate::{impl_into, impl_deref, each_field};

use dyn_clone::{clone_trait_object, DynClone};


#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum OP {
    NEVER(()) = 0x00,
    CALL(CallParser) = 0x02,
    FETCH() = 0x03,
    AND(AndParser) = 0x04,
    OR(OrParser) = 0x05,
    NOT(NotParser) = 0x06,
    VAL(ValParser) = 0x07,
    SEQ(SeqParser) = 0x0a,
    TO_SOME(ToSomeParser) = 0x20,
    FROM_SOME(FromSomeParser) = 0x21,
    OR_SOME(OrSomeParser) = 0x22,
    EQ(EqualsParser) = 0x23,
    GET_STRUCT_FIELD(GetStructFieldParser) = 0x30,
    SET_STRUCT_FIELD(SetStructFieldParser) = 0x31,
    MAP(MapParser) = 0x40,
    ALL(AllParser) = 0x41,
    ANY(AnyParser) = 0x42,
    EACH(EachParser) = 0x43,
    LEN(LengthParser) = 0x44,
    INDEX(IndexParser) = 0x50,
    SLICE(SliceParser) = 0x55,
    VAR(VarParser) = 0x60,
    SETVAR(SetVarParser) = 0x61,
    ADD(AddParser) = 0x70,
    IF(IfParser) = 0x80,
    //#[cfg(feature = "anchor")]
    //INVOKE_SIGNED(InvokeSignedParser) = 0xB1,
    //GET_INVOKE_RETURN(GetInvokeReturnParser) = 0xBA,
    PANIC(PanicParser) = 0xc0,
    ASSERT(AssertParser) = 0xc1,
}
const _: () = assert!(std::mem::size_of::<OP>() == 1);
impl_into!([], OP, u8, |self| unsafe { std::mem::transmute::<u8, OP>(self) });
impl_into!([], u8, OP, |self| unsafe { std::mem::transmute::<OP, u8>(self) });



#[derive(Debug, Clone)]
pub enum OpTree {
    Op(Option<u8>, Vec<OpTree>),
    LengthPrefix(Box<OpTree>),
    Data(Vec<u8>),
}





#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct VarId<I>(usize, PhantomData<I>);
impl<I> Deref for VarId<I> {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.0 as *const u16) }
    }
}
impl<I> DerefMut for VarId<I> {
    fn deref_mut(&mut self) -> &mut u16 {
        unsafe { &mut *(self.0 as *const u16 as *mut u16) }
    }
}
impl<I> Decode for VarId<I> {
    fn rd_decode(buf: Buf) -> std::result::Result<Self, String> {
        Ok(VarId::from(u16::rd_decode(buf)?))
    }
}
impl<I> VarId<I> {
    pub fn new() -> Self {
        Self::from(u16::MAX)
    }
    pub fn from(var_id: u16) -> VarId<I> {
        let ptr = Box::leak(Box::new(var_id)) as *mut u16 as *mut () as usize;
        VarId(ptr, PhantomData::default())
    }
    pub fn populate(&mut self, ctx: &mut EncodeContext) {
        if **self == u16::MAX {
            **self = ctx.next();
        }
    }
}





pub trait OpEncode {
    fn op_encode(&mut self, _ctx: &mut EncodeContext) -> OpTree;
}
impl<R: Encode> OpEncode for R {
    fn op_encode(&mut self, _ctx: &mut EncodeContext) -> OpTree {
        OpTree::Data(self.rd_encode())
    }
}




clone_trait_object!(<R> Op<R>);
pub trait Op<R: std::fmt::Debug>: DynClone + OpEncode + std::fmt::Debug {}


/*
 * RR hides all the nasty type complexity in the Ops, and includes only the
 * return type.
 */

pub struct RR<R>(pub Box<dyn Op<R>>);
impl_deref!([R], RR<R> => Box<dyn Op<R>>, 0);
impl<R: std::fmt::Debug> RR<R> {
    pub fn new<O: Op<R> + 'static>(op: O) -> RR<R> {
        RR(Box::new(op))
    }
}
impl<R: SchemaType + Into<RD>> RR<R> {
    pub fn val(v: R) -> RR<R> {
        RR::new(Val::from(v))
    }
}
impl<R> OpEncode for RR<R> {
    fn op_encode(&mut self, ctx: &mut EncodeContext) -> OpTree {
        (**self).op_encode(ctx)
    }
}
impl<R: Clone> Clone for RR<R> {
    fn clone(&self) -> Self {
        RR(self.0.clone())
    }
}
impl<R> std::fmt::Debug for RR<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self.0).fmt(f)
    }
}
impl<S: SchemaType + Into<RD>> From<S> for RR<S> {
    fn from(value: S) -> Self {
        RR::val(value)
    }
}
impl<A: SchemaType> From<&RR<A>> for RR<A> {
    fn from(value: &RR<A>) -> Self {
        value.clone()
    }
}
pub trait ToRR<A> { fn rr(&self) -> RR<A>; }
impl<A: SchemaType> ToRR<A> for RR<A> { fn rr(&self) -> RR<A> { self.clone() } }
impl<A: SchemaType> ToRR<A> for &RR<A> { fn rr(&self) -> RR<A> { (*self).clone() } }
impl<A: SchemaType + Into<RD>> ToRR<A> for A { fn rr(&self) -> RR<A> { RR::val(self.clone()) } }
impl<A: SchemaType + Into<RD>> ToRR<A> for &A { fn rr(&self) -> RR<A> { RR::val((*self).clone()) } }

#[derive(Default, Debug)]
pub struct EncodeContext {
    next_var_id: u16,
    pub val_size: usize
}
impl EncodeContext {
    pub fn new() -> EncodeContext {
        Default::default()
    }
    pub fn next(&mut self) -> u16 {
        self.next_var_id += 1;
        self.next_var_id - 1
    }
}

pub(crate) trait HasParser {
    type R: TList;
}


macro_rules! parser_type {
    ((RR<Function<$($t:tt)*)) => { RR<EncodedFunction> };
    ((RR $($t:tt)*)) => { RR<()> };
    ((Skippable)) => { Skippable };
    ((VarId $($t:tt)*)) => { VarId<()> };
    ((PrependSchema::<$t:ty>)) => { PrependSchema<()> };
    ($t:ident) => { $t };
}
macro_rules! parser_type_2 {
    ((PhantomData $($t:tt)*) $($r:ident)*) => { parser_type_2!($($r)*) };
    ($t:tt $($r:tt)*) => { TCons<parser_type!($t), parser_type_2!($($r)*)> };
    () => { () };
}

use paste::paste;

macro_rules! opcode {
    ($(#$op:ident, )? $ret:ty, $name:ident
     <$($param:ident$(: $tr:path)?),*>
     ($($f0:tt $([$($mod:tt)*])?),*),
     |$self:ident, $ctx:ident| $expr:expr
    ) => {
        #[derive(Clone, Debug)]
        #[allow(unused_parens)]
        pub struct $name<$($param: Clone $(+ $tr)*),*>($(pub $f0),*);
        impl<$($param: Clone + std::fmt::Debug $(+ $tr)*),*> Op<$ret> for $name<$($param),*> {}
        impl<$($param: Clone + std::fmt::Debug $(+ $tr)*),*> OpEncode for $name<$($param),*> {
            #[allow(unused)]
            fn op_encode(&mut $self, $ctx: &mut EncodeContext) -> OpTree { $expr }
        }
        paste! {
            #[derive(Clone, Debug, PartialEq, Eq, Default)]
            pub struct [<$name Parser>];
            impl HasParser for [<$name Parser>] {
                type R = parser_type_2!($($($($mod)*)? $f0)*);
            }
        }
        impl<$($param: SchemaType $(+ $tr)*),*> ToRR<$ret> for $name<$($param),*> {
            fn rr(&self) -> RR<$ret> {
                RR::new(self.clone())
            }
        }
    };
    ($(#$op:ident, )? $ret:ty, $name:ident
     <$($param:ident$(: $tr:path)?),*>
     ($($f0:tt $([$($mod:tt)*])? ),*)
    ) => {
        opcode!(
            $(#$op, )? $ret, $name<$($param$(: $tr)?),*>
            ($($f0 $([$($mod),*])?),*), |self, ctx| {
                opcode!(|self, ctx| $($op, )? ($($f0 [$($($mod)*)?]),*) $)
             }
        );
    };
    // note $dol (https://stackoverflow.com/a/53971532) (forced to call another matcher)
    (|$self:ident,$ctx:ident| $($op:ident, )? ($($f0:tt [$($mod:tt)*] ),*) $D:tt) => {
        { let mut opcode: Option<u8> = None;
                $(opcode = Some(OP::$op(Default::default()).into());)?
                let mut trees = Vec::<OpTree>::new();
                macro_rules! mm {
                    ($i:tt, ((PhantomData<$D($t:tt)*))) => { };
                    ($i:tt, ($fi:tt $D($mia:tt),*)) => {
                        let t = $self.$i.op_encode($ctx);
                        $D(let t = <$mia>::op_encode(t, $ctx);)*
                        trees.push(t);
                    };
                }
                each_field!(|mm| $(($f0 $($mod),*)),*);
                OpTree::Op(opcode, trees) }
    };
}

impl<I: SchemaType, O: SchemaType> SchemaType for Function<I, O> {
    //type Items = ();
    fn encode_schema(out: &mut Vec<u8>) {
        out.push(tag::FUNCTION::ID);
        I::encode_schema(out);
        O::encode_schema(out);
    }
}

opcode!(#VAL, Function<I, O>, Function<I: SchemaType, O: SchemaType>(
        (VarId<I>) [(PrependSchema::<Function<I, O>>)],
        (RR<O>) [Skippable]
));

opcode!(#CALL, O, Call<I: SchemaType, O: SchemaType>((RR<I>), (RR<Function<I, O>>)));

opcode!(#EQ, bool, Equals<T>((RR<T>), (RR<T>)));
opcode!(#ADD, T, Add<T: std::ops::Add>((RR<T>), (RR<T>)));

opcode!(#AND, bool, And<>((RR<bool>), (RR<bool>) [Skippable]));
opcode!(#OR,  bool, Or<>((RR<bool>), (RR<bool>) [Skippable]));

pub trait HasLen { }
impl<I> HasLen for Vec<I> { }
impl HasLen for Buffer { }
impl HasLen for String { }

opcode!(#LEN, u16, Length<I: HasLen>((RR<I>)));

opcode!(#NOT, bool, Not<>((RR<bool>)));

opcode!(#VAL, A, Val<A: SchemaType>(RD [(PrependSchema::<A>)], (PhantomData<A>)));
impl_deref!([A: SchemaType], Val<A> => RD, 0);
impl<A: SchemaType + Clone> Val<A> {
    pub fn new(rd: RD) -> Val<A> { Val(rd, ph()) }
}
impl<A: SchemaType + Into<RD>> From<A> for Val<A> {
    fn from(value: A) -> Self {
        Val::new(value.into())
    }
}



opcode!(#IF, O, If<O: Clone>((RR<bool>), (RR<O>) [Skippable], (RR<O>) [Skippable]));
//#[cfg(feature = "anchor")]
//opcode!(#INVOKE_SIGNED, (), InvokeSigned<>((RR<MetalockProxyCall>)));
//opcode!(#GET_INVOKE_RETURN, Buffer, GetInvokeReturn<>());


opcode!(#PANIC, A, Panic<A>(String, (PhantomData<A>)));
opcode!(#ASSERT, (), Assert<>((RR<bool>), (RR<String>) [Skippable]));
opcode!(#INDEX, O, Index<O>((RR<Vec<O> >), (RR<u16>)));
opcode!(#SLICE, Vec<O>, Slice<O>((RR<Vec<O> >), (RR<u16>)));


pub(crate) fn ph<T: Default>() -> T { Default::default() }

impl<I> OpEncode for VarId<I> {
    fn op_encode(&mut self, ctx: &mut EncodeContext) -> OpTree {
        self.populate(ctx);
        OpTree::Data((**self).rd_encode())
    }
}
opcode!(#VAR, I, Var<I>((VarId<I>)));
impl<I: SchemaType> Var<I> {
    pub fn new() -> Var<I> {
        Var(VarId::new())
    }
}
opcode!(#SETVAR, (), SetVar<I>((VarId<I>), (RR<I>)));


opcode!(#MAP, Option<O>, MapOption<I: SchemaType, O: SchemaType>((RR<Option<I>>), (RR<Function<I, O>>)));
opcode!(#MAP, Vec<O>,    Map<I: SchemaType, O: SchemaType>((RR<Vec<I>>),          (RR<Function<I, O>>)));
opcode!(#ALL, bool,      All<I: SchemaType, V: IntoIterator<Item=I>>((RR<V>),     (RR<Function<I, bool>>)));
opcode!(#ANY, bool,      Any<I: SchemaType, V: IntoIterator<Item=I>>((RR<V>),     (RR<Function<I, bool>>)));
opcode!(#EACH, (),       Each<I: SchemaType, V: IntoIterator<Item=I> >((RR<V>),   (RR<Function<I, ()>>)));

opcode!(#TO_SOME, Option<I>, ToSome<I>((RR<I>)));
opcode!(#FROM_SOME, I, FromSome<I>((RR<Option<I>>), (RR<I>) [Skippable]));
opcode!(#OR_SOME, Option<I>, OrSome<I>((RR<Option<I>>), (RR<Option<I>>) [Skippable]));



opcode!(#SEQ, R, Seq<R>((RR<()>), (RR<R>)));

opcode!(#GET_STRUCT_FIELD, R, GetStructField<S, R>((RR<S>), u8, u32, (PhantomData<R>)));
opcode!(#SET_STRUCT_FIELD, S, SetStructField<S: SchemaType, R: SchemaType>((RR<S>), u8, u32, (RR<R>)));



pub(crate) struct PrependSchema<S: SchemaType>(PhantomData<S>);
impl<S: SchemaType> PrependSchema<S> {
    fn op_encode(op: OpTree, _ctx: &mut EncodeContext) -> OpTree {
        let schema = OpTree::LengthPrefix(OpTree::Data(S::to_schema().0).into());
        OpTree::Op(None, vec![schema, op])
    }
}

pub(crate) struct Skippable;
impl Skippable {
    fn op_encode(op: OpTree, _ctx: &mut EncodeContext) -> OpTree {
        OpTree::LengthPrefix(op.into())
    }
}


