
use std::marker::PhantomData;

use crate::{expr::*, newval::SchemaType, types::RD};

pub use crate::expr::{RR, Function};

pub fn rr<R: SchemaType, O: Op<R> + 'static>(op: O) -> RR<R> {
    RR::new(op)
}
pub fn to_function<B: ToRR<O>, I: SchemaType, O: SchemaType, F: FnOnce(RR<I>) -> B + 'static + Clone>(f: F) -> RR<Function<I, O>> {
    let var = Var::new();
    Function(var.0.clone(), f(var.rr()).rr().into()).rr()
}
pub fn assert(cond: impl ToRR<bool>, s: impl Into<String>) -> RR<()> {
    let s: String = s.into();
    Assert(cond.rr(), s.rr().into()).rr()
}
pub fn panic(s: impl Into<String>) -> RR<()> {
    Panic(s.into(), PhantomData::default()).rr()
}


impl<I: SchemaType> RR<I> {
    pub fn equals(self, other: impl ToRR<I>) -> RR<bool> {
        rr(Equals(self, other.rr()))
    }
    pub fn write(self, var: &Var<I>) -> RR<()> {
        rr(SetVar(var.0.clone(), self))
    }
    pub fn bind<O, F>(self, f: F) -> RR<O>
        where F: FnOnce(RR<I>) -> RR<O>
    {
        f(self.rr())
    }
}


impl<I: std::ops::Add + SchemaType, A: ToRR<I>> ToRRInt<I> for A {}
pub trait ToRRInt<I: std::ops::Add + SchemaType>: ToRR<I> + Sized {
    fn add(self, other: impl ToRR<I>) -> RR<I> {
        Add(self.rr(), other.rr()).rr()
    }
}

impl<A: ToRR<()>> ToRRUnit for A {}
pub trait ToRRUnit: ToRR<()> + Sized {
    fn then<R: SchemaType>(self, other: impl ToRR<R>) -> RR<R> {
        rr(Seq(self.rr(), other.rr()))
    }
}

impl<I: SchemaType, A: ToRR<Vec<I>>> ToRRVec<I> for A {}
pub trait ToRRVec<I: SchemaType>: ToRR<Vec<I>> + Sized {
    fn get(&self, idx: impl Into<RR<u16>>) -> RR<I> {
        rr(Index(self.rr(), idx.into()))
    }
    fn all<F: Fn(RR<I>) -> RR<bool> + Clone + 'static>(&self, f: F) -> RR<bool> {
        rr(All(self.rr(), to_function(f)))
    }
    fn any<F: Fn(RR<I>) -> RR<bool> + Clone + 'static>(&self, f: F) -> RR<bool> {
        rr(Any(self.rr(), to_function(f)))
    }
    fn map<F: Fn(RR<I>) -> RR<O> + Clone + 'static, O: SchemaType>(self, f: F) -> RR<Vec<O>> {
        rr(Map(self.rr(), to_function(f)))
    }
    fn slice(self, idx: impl ToRR<u16>) -> RR<Vec<I>> {
        rr(Slice(self.rr(), idx.rr()))
    }
}

impl<I: SchemaType, It: IntoIterator<Item=I> + SchemaType, A: ToRR<It>> ToRRIter<I, It> for A {}
pub trait ToRRIter<I: SchemaType, It: IntoIterator<Item=I> + SchemaType>: ToRR<It> + Sized {
    fn each<B: ToRR<()>, F: Fn(RR<I>) -> B + 'static + Clone>(self, f: F) -> RR<()> {
        rr(Each(self.rr(), to_function(f)))
    }
}

// webpage: DOS look, light grey and red letters on dark grey back
// METALOCK


impl<L: SchemaType + HasLen + 'static> RR<L> {
    pub fn len(self) -> RR<u16> {
        rr(Length(self))
    }
}


impl<A: ToRR<bool>> ToRRBool for A {}
pub trait ToRRBool: ToRR<bool> + Sized {
    fn choose<A: SchemaType>(&self, a: impl ToRR<A>, b: impl ToRR<A>) -> RR<A> {
        rr(If(self.rr(), a.rr().into(), b.rr().into()))
    }
    fn and(&self, other: Self) -> RR<bool> {
        rr(And(self.rr(), other.rr().into()))
    }
    fn or(&self, other: Self) -> RR<bool> {
        rr(Or(self.rr(), other.rr().into()))
    }
    fn not(&self) -> RR<bool> {
        rr(Not(self.rr()))
    }
}

impl<I: SchemaType, A: ToRR<Option<I>>> ToRROption<I> for A {}
pub trait ToRROption<I: SchemaType>: ToRR<Option<I>> + Sized {
    fn map<F: FnOnce(RR<I>) -> RR<O> + Clone + 'static, O: SchemaType>(self, f: F) -> RR<Option<O>> {
        rr(MapOption(self.rr(), to_function(f)))
    }
}



impl<I: SchemaType, O: SchemaType> RR<Function<I, O>> {
    fn call(&self, input: impl ToRR<I>) -> RR<O> {
        Call(input.rr(), self.clone()).rr()
    }
}

pub fn m_if<O: SchemaType>(c: RR<bool>, r: impl ToRR<O>) -> RR<Option<O>> {
    rr(If(c, rr(ToSome(r.rr())).into(), Val::new(RD::none()).rr().into()))
}


impl<O: SchemaType + Into<RD>> RR<Option<O>> {
    pub fn m_else(self, alt: impl ToRR<O>) -> RR<O> {
        rr(FromSome(self, alt.rr().into()))
    }
    pub fn m_elseif(self, c: RR<bool>, r: RR<O>) -> RR<Option<O>> {
        rr(OrSome(self, m_if(c, r).into()))
    }
}

impl<O: SchemaType + Into<RD>> RR<Option<Option<O>>> {
    pub fn join(self) -> RR<Option<O>> {
        rr(FromSome(self, None.rr().into()))
    }
}

impl<A: SchemaType, B: SchemaType> RR<(A, B)> {
    pub fn unpack(self) -> (RR<A>, RR<B>) {
        let tup_a: RR<Vec<A>> = unsafe { std::mem::transmute(self.clone()) };
        let tup_b: RR<Vec<B>> = unsafe { std::mem::transmute(self) };
        (tup_a.get(0), tup_b.get(1))
    }
}


impl<T: SchemaType> Var<T> {
    pub fn get(&self) -> RR<T> {
        rr(self.clone())
    }
}



#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_and() {
        assert!(RR::val(true).and(RR::val(true)).eval() == true.into());
        assert!(RR::val(true).and(RR::val(false)).eval() == false.into());
        assert!(RR::val(false).and(RR::val(true)).eval() == false.into());
        assert!(RR::val(false).and(RR::val(false)).eval() == false.into());
    }

    #[test]
    fn test_or() {
        assert!(RR::val(true).or(RR::val(true)).eval() == true.into());
        assert!(RR::val(true).or(RR::val(false)).eval() == true.into());
        assert!(RR::val(false).or(RR::val(true)).eval() == true.into());
        assert!(RR::val(false).or(RR::val(false)).eval() == false.into());
    }

    #[test]
    fn test_all() {
        let mut comp = Val::from(vec![false, false, false]).all(|b| b.not());
        assert!(comp.eval() == true.into());
        let mut comp = Val::from(vec![false, false, true]).all(|b| b.not());
        assert!(comp.eval() == false.into());
    }

    #[test]
    fn test_len() {
        let mut comp = RR::val(vec![false, false]).len();
        assert!(comp.eval() == 2u16.into());
    }

    #[test]
    fn test_map() {
        let v = vec!["hi".to_string(), "there".to_string()];
        let mut comp = Val::from(v).map(|s| s.len());
        assert!(comp.eval() == vec![2u16, 5].into());
        // Test that works after map too
        assert!(comp.any(|v| v.equals(5u16)).eval() == true.into());
    }

    #[test]
    fn test_map_opt() {
        let o = Some(10u8).rr();
        let mut comp = o.map(|s| s.add(1));
        assert!(comp.eval() == Some(11u8).into())
    }

    #[test]
    fn test_slice() {
        let v = (0u8..10).collect::<Vec<_>>();
        let mut comp = Val::from(v).slice(8);
        assert!(comp.eval() == vec![8u8, 9].into());
    }

    #[test]
    fn test_any() {
        let v = vec![7u8, 0xcc, 9];
        let mut comp = Val::from(v).any(|p| p.equals(0xcc));
        assert!(comp.eval() == true.into());
    }

    #[test]
    fn test_if() {
        // If
        let mut comp = If(
            RR::val(true),
            RR::val(1u8).into(),
            RR::val(2u8).into()
        );
        assert!(comp.eval() == 1u8.into());
    }

    #[test]
    fn test_mif() {
        // Mif / Melse
        let comp = |a, b| {

            m_if(
                RR::val(a),
                RR::val(1u8)
            ).m_elseif(
                RR::val(b),
                RR::val(2)
            ).m_else(
                RR::val(3)
            )
        };
        assert!(comp(true, false).eval() == 1u8.into());
        assert!(comp(false, true).eval() == 2u8.into());
        assert!(comp(false, false).eval() == 3u8.into());
    }

    #[test]
    fn test_then() {
        // Then
        let mut comp = RR::val(()).then(RR::val(true).not());
        assert!(comp.eval() == false.into());
    }

    #[test]
    fn test_join() {
        // Join
        let mut comp = RR::val(Some(Some(true))).join();
        assert_eq!(comp.eval(), Some(true).into());
    }

    #[test]
    fn test_write() {
        let a = Var::new();
        let r = RR::val(true).write(&a).then(rr(a)).eval();
        assert!(r == true.into());
    }

    #[test]
    fn test_function() {
        // functions take a reference to an offset and their variables are stored as I+offset
        let f = to_function(|i: RR<u8>| i.add(1));
        let mut r = f.call(10);
        assert_eq!(r.eval(), 11u8.into());
    }

    #[test]
    #[should_panic]
    fn test_overflow() {
        let mut comp = 200u8.add(200);
        println!("r is: {:?}", comp.eval());
    }
}

