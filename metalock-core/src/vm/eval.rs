
use std::marker::PhantomData;

#[cfg(feature = "measure-cu")]
use anchor_lang::solana_program::compute_units::sol_remaining_compute_units;

#[cfg(feature = "anchor")]
use anchor_lang::{prelude::*, solana_program as sp};

use crate::types::tlist::*;
use crate::types::core::*;
use crate::types::decode::*;
use crate::types::data::*;
use crate::types::newval::*;
use crate::types::parse::*;
use super::expr::*;

pub use super::expr::Function;



macro_rules! orsome {
    ($self:ident, $p:ident, |$a:ident| $ae:expr, |$res:ident| $rese:expr) => {
        {
            let p = EvalParser::from($self, $p);
            let C($a, p) = p.eval();
            if let RD::Option(r) = $ae {
                if let Some($res) = r.as_ref() {
                    p.skip();
                    $rese
                } else {
                    p.eval()
                }
            } else {
                panic!("expected RD::Option")
            }
        }
    };
}

macro_rules! fn_map {
    (|$self:ident, $p:ident, $f:ident, $val:ident| $expr:expr) => {
        {
            let p = EvalParser::from($self, $p);
            let ($val, EncodedFunction(ref_id, body)) = p.eval().take_fun1();

            let buf = *$self.buf;

            let mut $f = |item: &RD| {
                $self.vars[*ref_id as usize] = item.clone();
                *$self.buf = body;
                $self.eval()
            };

            let out = $expr;
            *$self.buf = buf;
            out
        }
    };
}


#[derive(Default, Clone)]
pub struct EvaluatorContext {
    //#[cfg(feature = "anchor")]
    //pub proxy_calls: Vec<MetalockProxyCall>,
    //#[cfg(feature = "anchor")]
    //pub remaining_accounts: &'static [AccountInfo<'static>],
    //pub beastie_seeds: Vec<&'static [u8]>
}


pub struct Evaluator {
    start: ParserBuffer,
    pub(crate) buf: ParserBuffer,
    vars: Vec<RD>, // Vector of pointers
    ctx: EvaluatorContext,
    stack: Vec<u8>,
    #[cfg(feature = "measure-cu")]
    profile: (OP, u64, BTreeMap<OP, u64>),
}

impl Evaluator {
    pub fn new<'a, 'b>(buf: Buf<'a, 'b>, ctx: EvaluatorContext) -> Evaluator {
        Evaluator {
            start: ParserBuffer::new(*buf),
            buf: ParserBuffer::new(*buf),
            vars: vec![RD::Unit(); 100],
            ctx,
            stack: vec![0],
            #[cfg(feature = "measure-cu")]
            profile: Default::default(),
        }
    }

    pub fn run(&mut self, input: RD) -> RD {
        self.vars[0] = input;
        self.eval()
    }

    fn eval(&mut self) -> RD {
        let op = self.take_op();

        #[cfg(feature = "measure-cu")]
        let last_op = {
            if self.profile.1 > 0 {
                let cu = sol_remaining_compute_units();
                let used = self.profile.1 - cu;
                self.profile.2.entry(op).and_modify(|cu| *cu += used).or_insert(used);
            }
            let last_op = self.profile.0;
            self.profile.0 = op;
            self.profile.1 = sol_remaining_compute_units();
            last_op
        };

        let r = match op {

            OP::AND(p) => {
                let p = EvalParser::from(self, p);
                let C(r, p) = p.eval();
                if r._as() {
                    p.eval()
                } else {
                    p.skip();
                    false.into()
                }
            },

            OP::OR(p) => {
                let p = EvalParser::from(self, p);
                let C(r, p) = p.eval();
                if r._as() {
                    p.skip();
                    r
                } else {
                    p.eval()
                }
            },
            OP::NOT(_) => RD::Bool(self.eval()._as::<bool>() == false),

            OP::EQ(p) => {
                let p = EvalParser::from(self, p);
                let (a, b) = p.eval().eval();
                (a == b).into()
            },
            OP::LEN(_) => {
                ((match self.eval() {
                    RD::String(s) => s.len(),
                    RD::List(s) => s.len(),
                    RD::Buffer(s) => s.len(),
                    _ => panic!("Length: type mismatch")
                }) as u16).into()
            },
            OP::ADD(_) => {
                match (self.eval(), self.eval()) {
                    (RD::U8(a),  RD::U8(b))  => (a+b).into(),
                    (RD::U16(a), RD::U16(b)) => (a+b).into(),
                    (RD::U32(a), RD::U32(b)) => (a+b).into(),
                    (RD::U64(a), RD::U64(b)) => (*a+*b).into(),
                    (a, b) => panic!("OP::ADD unexpected: {:?}, {:?}", a, b)
                }
            },

            //
            OP::SEQ(_) => { self.eval(); self.eval() },

            //
            OP::MAP(p) => {
                fn_map!(|self, p, f, val| {
                    match val {
                        RD::List(v) => v.iter().map(f).collect::<Vec<_>>().into(),
                        RD::Option(o) => o.as_ref().map(f).into(),
                        RD::Native(c) => c.iter().map(|o| f(&o)).collect::<Vec<_>>().into(),
                        _ => panic!("OP::MAP: unexpected")
                    }
                })
            },
            OP::ALL(p) => self.fn_map(p).into_iter().all(|rd| rd == RD::Bool(true)).into(),
            OP::ANY(p) => self.fn_map(p).iter().any(|rd| rd == &RD::Bool(true)).into(),
            OP::EACH(p) => { self.fn_map(p); RD::Unit() },
            OP::SLICE(_) => {
                let o = self.eval();
                let idx = self.eval()._as::<u16>() as usize;
                match o {
                    RD::List(vec) => vec[idx..].to_vec().into(),
                    RD::Native(c) => c.slice(idx).into(),
                    _ => panic!("SLICE: Expecting RD::List")
                }
            },
            OP::INDEX(p) => {
                let p = EvalParser::from(self, p);
                let C(rd, p) = p.eval();
                let idx = p.eval_as::<u16>() as usize;

                match rd {
                    RD::List(vec) => vec[idx].clone(),
                    RD::Tuple(vec) => vec[idx].clone(),
                    RD::Native(p) => (*p).index(idx).into(),
                    _ => panic!("INDEX: Expecting RD::[List,Tuple,Native], got: {:?}", rd)
                }
            },

            //
            OP::VAL(p) => {
                data_parse(&mut self.buf).expect("failed reading data")
            },
            OP::VAR(p) => {
                let var_id = EvalParser::from(self, p).take();
                self.vars[*var_id as usize].clone()
            },
            OP::SETVAR(p) => {
                let p = EvalParser::from(self, p);
                let (ref_id, r) = p.take().eval();
                self.vars[*ref_id as usize] = r;
                RD::Unit()
            },

            OP::GET_STRUCT_FIELD(p) => {
                let p = EvalParser::from(self, p);
                let ((c, field), off) = p.eval_as::<&'static Native>().take().take();
                c.get_struct_field(field, off)
            },

            OP::SET_STRUCT_FIELD(p) => {
                let p = EvalParser::from(self, p);
                let (((c, _), off), val) = p.eval_as::<&'static Native>().take().take().eval();
                c.set_struct_field(off, val).into()
            },

            OP::IF(p) => {
                let p = EvalParser::from(self, p);
                let C(e, p) = p.eval();

                if e._as() {
                    p.eval().skip().0
                } else {
                    p.skip().eval().1
                }
            },

            OP::TO_SOME(_) => { Some(self.eval()).into() },
            OP::FROM_SOME(p) => { orsome!(self, p, |a| a, |res| res.clone()) },
            OP::OR_SOME(p) => orsome!(self, p, |a| &a, |_res| a),

            //#[cfg(feature = "anchor")]
            //OP::INVOKE_SIGNED(_) => {
            //    let call = match self.eval() {
            //        RD::Native(c) => {
            //            let Native(_rs, p) = &*c;
            //            unsafe { &*(*p as *const MetalockProxyCall) }
            //        },
            //        _ => panic!("INVOKE_SIGNED: expected Native")
            //    };

            //    // Not as much of a CU saving as you might think
            //    let accounts: Vec<AccountMeta> = unsafe {
            //        let p: &[usize; 3] = std::mem::transmute(&call.accounts);
            //        std::mem::transmute(*p)
            //    };

            //    //let cu = sol_remaining_typed parser that yields bcompute_units();
            //    sp::program::invoke_signed_unchecked(
            //        &sp::instruction::Instruction::new_with_bytes(call.program_id, &call.data.0, accounts),
            //        self.ctx.remaining_accounts,
            //        &[&*self.ctx.beastie_seeds],
            //    ).expect("INVOKE_SIGNED: call failed");
            //    //msg!("console.log call used: {}", cu - sol_remaining_compute_units());

            //    RD::Unit()
            //},
            //#[cfg(feature = "anchor")]
            //OP::GET_INVOKE_RETURN(_) => {
            //    let (_, r) = sp::program::get_return_data().expect("Expected instruction to return data");
            //    Buffer(r).into()
            //},

            OP::PANIC(_) => {
                let s: &String = self.eval()._as();
                panic!("{}", s);
            },
            OP::ASSERT(_) => {
                let pass = self.eval() == true.into();
                let len = self.buf.decode();
                if pass {
                    self.skip(len);
                } else {
                    let msg: &String = self.eval()._as();
                    panic!("{}", msg);
                }
                RD::Unit()
            },

            OP::CALL(p) => {
                let p = EvalParser::from(self, p);
                let C(input, p) = p.eval();
                let f = p.take_fun1();

                self.vars[f.0 as usize] = input;
                self.fetch(&f.1)
            },

            OP::FETCH() => {
                let off = self.buf.take_u16() as usize;
                self.fetch(&self.start[off..])
            },

            _ => {
                panic!("Invalid opcode: {}", Into::<u8>::into(op))
            },
        };


        #[cfg(feature = "measure-cu")]
        {
            let cu = sol_remaining_compute_units();
            let used = self.profile.1 - cu + (1<<32);
            self.profile.2.entry(op).and_modify(|cu| *cu += used).or_insert(used);
            self.profile.0 = last_op;
            self.profile.1 = sol_remaining_compute_units();
        }

        r
    }

    fn fetch(&mut self, buf: &'static [u8]) -> RD {
        let prev = *self.buf;
        *self.buf = buf;
        let out = self.eval();
        *self.buf = prev;
        out
    }

    fn fn_map(&mut self, p: impl HasParser<R=tlist!(RR<()>, RR<EncodedFunction>)>) -> Vec<RD> {
        fn_map!(|self, p, f, val| {
            match val {
                RD::List(vec) => vec.iter().map(f).collect(),
                RD::Option(o) => o.as_ref().into_iter().map(f).collect(),
                RD::Native(c) => c.iter().map(|o| f(&o)).collect(),
                _ => panic!("eval::fn_map: invalid type")
            }
        })
    }

    fn take_op(&mut self) -> OP {
        let op = self.buf.next().into();
        //println!("OP IS: {:?}", op);
        op
    }

    #[inline]
    fn skip(&mut self, bytes: u16) {
        *self.buf = &self.buf[bytes as usize..];
    }


    pub fn print_profile_info(&self, _n: usize) {
        #[cfg(feature = "measure-cu")]
        {
            let mut vals: Vec<(OP, u64)> = self.profile.2.clone().into_iter().collect();
            vals.sort_by(|(_, a), (_, b)| (b&0xffffffff).cmp(&(a&0xffffffff)));
            vals.into_iter().take(n).for_each(|(op, cu)| {
                msg!("CU for op: {:?}: {} ({})", op, cu&0xffffffff, cu>>32);
            });
        }
    }

}


struct C<'a, O, T: TList>(pub O, pub EvalParser<'a, T>);


struct EvalParser<'a, T: TList>(&'a mut Evaluator, PhantomData<T>);
impl<'a, A, T: TList> EvalParser<'a, TCons<A, T>> {
    pub fn from<P: HasParser<R=TCons<A, T>>>(eval: &'a mut Evaluator, _p: P) -> EvalParser<'a, P::R> {
        EvalParser(eval, PhantomData::default())
    }
}
macro_rules! wrap_tcons {
    ($head:ty, $($rest:ty),*) => { TCons<$head, wrap_tcons!($($rest),*)> };
    ($head:ty) => { $head };
}
macro_rules! parser_taker {
    (<$($param:ident$(: $tr0:ident)?),*> ($($matcher:ty),*), $name:ident, $(@<$($f:ident: $t:ident)*>)?$ret:ty, |$self:ident| $expr:expr) => {
        #[allow(unused)]
        impl<'a, B, T: TList$(, $param$(: $tr0)?)*> EvalParser<'a, wrap_tcons!($($matcher,)* TCons<B, T>)> {
            pub fn $name$(<$($f: $t)*>)?($self) -> C<'a, $ret, TCons<B, T>> {
                C($expr, EvalParser($self.0, PhantomData::default()))
            }
        }
        #[allow(unused)]
        impl<'a, E, B, T: TList$(, $param$(: $tr0)?)*> C<'a, E, wrap_tcons!($($matcher,)* TCons<B, T>)> {
            pub fn $name$(<$($f: $t)*>)?($self) -> C<'a, (E, $ret), TCons<B, T>> {
                let C(r, p) = $self.1.$name();
                C(($self.0, r), p)
            }
        }
        #[allow(unused)]
        impl<'a $(, $param$(: $tr0)?)*> EvalParser<'a, wrap_tcons!($($matcher,)* ())> {
            pub fn $name$(<$($f: $t)*>)?($self) -> $ret { $expr }
        }
        #[allow(unused)]
        impl<'a, E $(, $param$(: $tr0)?)*> C<'a, E, wrap_tcons!($($matcher,)* ())> {
            pub fn $name$(<$($f: $t)*>)?($self) -> (E, $ret) {
                ($self.0, $self.1.$name())
            }
        }
    };
}
parser_taker!(<> (RR<()>), eval_as, @<O: FromRD> O, |self| {
    self.0.eval()._as()
});
parser_taker!(<> (RR<()>), eval, RD, |self| {
    self.0.eval()
});
parser_taker!(<S: Decode> (S), take, S, |self| {
    self.0.buf.decode()
});
parser_taker!(<N> (Skippable, N), skip, (), |self| {
    let n: u16 = self.0.buf.decode();
    self.0.buf.skip_bytes(n as usize);
});
parser_taker!(<> (Skippable, RR<()>), eval, RD, |self| {
    self.0.buf.skip_bytes(2);
    self.0.eval()
});
parser_taker!(<> (RR<EncodedFunction>), take_fun1, &'static EncodedFunction, |self| {
    self.0.eval()._as()
});
