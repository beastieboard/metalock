
use std::marker::PhantomData;
use std::usize;

use crate::compile::OpTree;
use crate::eval::Evaluator;
use crate::eval::EvaluatorContext;
use crate::types::impl_deref;
use crate::types::impl_into;
use crate::types::each_field;
use crate::newval::SchemaType;
use crate::types::*;
use crate::encode::*;
#[cfg(feature = "anchor")]
use crate::types::anchor::*;

use dyn_clone::{clone_trait_object, DynClone};


#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub(crate) enum OP {
    NEVER(()) = 0x00,
    CALL(CallParser) = 0x02,
    FETCH() = 0x03,
    AND(AndParser) = 0x04,
    OR(OrParser) = 0x05,
    NOT(NotParser) = 0x06,
    VAL(()) = 0x07,
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
    #[cfg(feature = "anchor")]
    INVOKE_SIGNED(InvokeSignedParser) = 0xB1,
    GET_INVOKE_RETURN(GetInvokeReturnParser) = 0xBA,
    PANIC(PanicParser) = 0xc0,
    ASSERT(AssertParser) = 0xc1,
}
const _: () = assert!(std::mem::size_of::<OP>() == 1);
impl_into!([], OP, u8, |self| unsafe { std::mem::transmute::<u8, OP>(self) });
impl_into!([], u8, OP, |self| unsafe { std::mem::transmute::<OP, u8>(self) });









pub trait OpEncode {
    fn op_encode(&mut self, _ctx: &mut EncodeContext) -> OpTree;
    fn is_op(&self) -> bool { true }
}
impl<R: Encode> OpEncode for R {
    fn is_op(&self) -> bool { false }
    fn op_encode(&mut self, _ctx: &mut EncodeContext) -> OpTree {
        OpTree::Data(self.rd_encode())
    }
}




clone_trait_object!(<R> Op<R>);
pub trait Op<R: std::fmt::Debug>: DynClone + OpEncode + std::fmt::Debug {
    fn eval(&mut self) -> RD {
        self.eval_with_context(Default::default(), usize::MAX)
    }
    fn encode(&mut self) -> Vec<u8> {
        self.op_encode(&mut EncodeContext::new()).join()
    }
    fn eval_with_context(&mut self, ctx: EvaluatorContext, dedupe_threshold: usize) -> RD {
        let o = self.op_encode(&mut EncodeContext::new()).join_threshold(dedupe_threshold);
        Evaluator::new(&mut o.as_ref(), ctx).run(RD::Unit())
    }
}


/*
 * RR hides all the nasty type complexity in the Ops, and includes only the
 * return type.
 */

pub struct RR<R>(pub Box<dyn Op<R>>);
impl_deref!([R], RR<R>, Box<dyn Op<R>>, 0);
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



macro_rules! field_wrapper {
    ($name:ident<$($param:ident$(: $tr:ident)?),*>($f0:ty), |$self:ident, $ctx:ident| $encode:expr) => {
        #[derive(Clone, Debug)]
        pub struct $name<$($param: Clone $(+ $tr)*),*>(pub $f0);
        impl<$($param: Clone $(+ $tr)*),*> OpEncode for $name<$($param),*> {
            #[allow(unused)]
            fn op_encode(&mut $self, $ctx: &mut EncodeContext) -> OpTree {
                $encode
            }
        }
        impl<$($param: SchemaType $(+ $tr)?),*> Into<$name<$($param),*>> for $f0 {
            fn into(self) -> $name<$($param),*> {
                $name(self)
            }
        }
    };
}

type RRFunction<I, O> = RR<Function<I, O>>;

macro_rules! parser_type {
    (RR) => { RR<()> };
    (RRFunction) => { RR<Function<(), ()>> };
    (Jump) => { Jump<()> };
    (Var) => { u16 };
    (Function) => { Function<(), ()> };
    (VarId) => { VarId<()> };
    ($t:ident) => { $t };
}
macro_rules! parser_type_2 {
    (PhantomData $($r:ident)*) => { parser_type_2!($($r)*) };
    ($t:ident $($r:ident)*) => { TCons<parser_type!($t), parser_type_2!($($r)*)> };
    () => { () };
}

use paste::paste;

// There should be blocked scoped variables, so during compilation the blocks need to be
// identified.
// Var should create an anon memory address, then this can be assigned a nonce during compile.

macro_rules! opcode {
    ($(#$op:ident, )? $ret:ty, $name:ident
     <$($param:ident$(: $tr:path)?),*>
     ($($f0:ident $(<$($f1:ty),*>)? ),*),
     |$self:ident, $ctx:ident| $expr:expr
    ) => {
        #[derive(Clone, Debug)]
        pub struct $name<$($param: Clone $(+ $tr)*),*>($(pub $f0 $(<$($f1),*>)? ),*);
        impl<$($param: Clone + std::fmt::Debug $(+ $tr)*),*> Op<$ret> for $name<$($param),*> {}
        impl<$($param: Clone + std::fmt::Debug $(+ $tr)*),*> OpEncode for $name<$($param),*> {
            #[allow(unused)]
            fn op_encode(&mut $self, $ctx: &mut EncodeContext) -> OpTree {
                $expr
            }
        }
        paste! {
            #[derive(Clone, Debug, PartialEq, Eq, Default)]
            pub struct [<$name Parser>];
            impl HasParser for [<$name Parser>] {
                type R = parser_type_2!($($f0)*);
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
     ($($f0:ident $(<$($f1:ty),*>)? ),*)
    ) => {
        opcode!(
            $(#$op, )? $ret, $name<$($param$(: $tr)?),*>
            ($($f0 $(<$($f1),*>)? ),*), |self, ctx| {
                let mut opcode: Option<u8> = None;
                $(opcode = Some(OP::$op(Default::default()).into());)?
                let mut trees = Vec::<OpTree>::new();
                macro_rules! mm {
                    ($i:tt, PhantomData) => { };
                    ($i:tt, $fi:tt) => { trees.push(self.$i.op_encode(ctx)); };
                }
                each_field!(|mm| $($f0),*);
                OpTree::Op(opcode, trees)
             }
        );
    };
}

opcode!(#CALL, O, Call<I: SchemaType, O: SchemaType>(RR<I>, RRFunction<I, O>));

opcode!(#EQ, bool, Equals<T>(RR<T>, RR<T>));
opcode!(#ADD, T, Add<T: std::ops::Add>(RR<T>, RR<T>));

opcode!(#AND, bool, And<>(RR<bool>, Jump<bool>));
opcode!(#OR,  bool, Or<>(RR<bool>, Jump<bool>));

pub(crate) trait HasLen { }
impl<I> HasLen for Vec<I> { }
impl HasLen for Buffer { }
impl HasLen for String { }

opcode!(#LEN, u16, Length<I: HasLen>(RR<I>));

opcode!(#NOT, bool, Not<>(RR<bool>));


#[derive(Clone, Debug)]
pub(crate) struct Val<A: Clone>(pub RD, pub PhantomData<A>);
impl<A: SchemaType> ToRR<A> for Val<A> {
    fn rr(&self) -> RR<A> {
        RR::new(self.clone())
    }
}
impl_deref!([A: Clone], Val<A>, RD, 0);
impl<A: SchemaType + std::fmt::Debug> Op<A> for Val<A> {}
fn op_encode_val<A: SchemaType>(val: OpTree, _ctx: &mut EncodeContext) -> OpTree {
    OpTree::Op(
        Some(OP::VAL(()).into()),
        vec![
            OpTree::LengthPrefix(OpTree::Data(A::to_schema().0).into()),
            val
        ]
    )
}
impl<A: SchemaType> OpEncode for Val<A> {
    fn op_encode(&mut self, ctx: &mut EncodeContext) -> OpTree {
        op_encode_val::<A>(OpTree::Data(self.0.rd_encode()), ctx)
    }
}
impl<A: SchemaType + Clone> Val<A> {
    pub fn new(rd: RD) -> Val<A> {
        Val(rd, ph())
    }
}
impl<A: SchemaType + Into<RD>> From<A> for Val<A> {
    fn from(value: A) -> Self {
        Val::new(value.into())
    }
}


field_wrapper!(Jump<R>(RR<R>), |self, ctx| OpTree::LengthPrefix(self.0.op_encode(ctx).into()));
impl From<&str> for Jump<String> {
    fn from(s: &str) -> Self {
        RR::val(s.into()).into()
    }
}


opcode!(#IF, O, If<O: Clone>(RR<bool>, Jump<O>, Jump<O>));
#[cfg(feature = "anchor")]
opcode!(#INVOKE_SIGNED, (), InvokeSigned<>(RR<MetalockProxyCall>));
opcode!(#GET_INVOKE_RETURN, Buffer, GetInvokeReturn<>());


opcode!(#PANIC, A, Panic<A>(String, PhantomData<A>));
opcode!(#ASSERT, (), Assert<>(RR<bool>, Jump<String>));
opcode!(#INDEX, O, Index<O>(RR<Vec<O> >, RR<u16>));
opcode!(#SLICE, Vec<O>, Slice<O>(RR<Vec<O> >, RR<u16>));


pub(crate) fn ph<T: Default>() -> T { Default::default() }

impl<I> OpEncode for VarId<I> {
    fn op_encode(&mut self, ctx: &mut EncodeContext) -> OpTree {
        self.populate(ctx);
        OpTree::Data((**self).rd_encode())
    }
}
//impl<I: Clone> Decode for VarId<I> {
//    fn rd_decode(buf: Buf) -> std::result::Result<Self, String> {
//        Ok(u16::rd_decode(buf)?.into())
//    }
//}
opcode!(#VAR, I, Var<I>(VarId<I>));
impl<I: SchemaType> Var<I> {
    pub fn new() -> Var<I> {
        Var(VarId::new())
    }
}
opcode!(#SETVAR, (), SetVar<I>(VarId<I>, RR<I>));


opcode!(#MAP, Option<O>, MapOption<I: SchemaType, O: SchemaType>(RR<Option<I>>, RRFunction<I, O>));
opcode!(#MAP, Vec<O>,    Map<I: SchemaType, O: SchemaType>(RR<Vec<I>>,          RRFunction<I, O>));
opcode!(#ALL, bool,      All<I: SchemaType, V: IntoIterator<Item=I>>(RR<V>,     RRFunction<I, bool>));
opcode!(#ANY, bool,      Any<I: SchemaType, V: IntoIterator<Item=I>>(RR<V>,     RRFunction<I, bool>));
opcode!(#EACH, (),       Each<I: SchemaType, V: IntoIterator<Item=I> >(RR<V>,   RRFunction<I, ()>));

opcode!(#TO_SOME, Option<I>, ToSome<I>(RR<I>));
opcode!(#FROM_SOME, I, FromSome<I>(RR<Option<I> >, Jump<I>));
opcode!(#OR_SOME, Option<I>, OrSome<I>(RR<Option<I> >, Jump<Option<I> >));



opcode!(#SEQ, R, Seq<R>(RR<()>, RR<R>));

opcode!(#GET_STRUCT_FIELD, R, GetStructField<S, R>(RR<S>, u8, u32, PhantomData<R>));
opcode!(#SET_STRUCT_FIELD, S, SetStructField<S: SchemaType, R: SchemaType>(RR<S>, u8, u32, RR<R>));


opcode!(#VAL, Function<I, O>, Function<I: SchemaType, O: SchemaType>(VarId<I>, Jump<O>), |self, ctx| {
    let var = self.0.op_encode(ctx);
    let func = self.1.op_encode(ctx).into();
    let val = OpTree::Op(None, vec![var, func]);
    op_encode_val::<Function<I, O>>(val, ctx)
});


