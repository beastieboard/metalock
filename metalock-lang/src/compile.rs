use std::{collections::BTreeMap, fmt::Debug, hash::Hasher, usize};
use std::collections::hash_map::DefaultHasher;

use metalock_core::internal::*;
use metalock_core::vm::{eval::{Evaluator, EvaluatorContext, EvalResult}, expr::*};


impl<R: Debug, O: ?Sized + Op<R>> OpEval<R> for O {}
pub trait OpEval<R: Debug>: Op<R> {
    fn eval(&mut self) -> EvalResult<RD> {
        self.eval_with_context(Default::default(), usize::MAX)
    }
    fn encode(&mut self) -> Vec<u8> {
        self.op_encode(&mut EncodeContext::new()).join()
    }
    fn eval_with_context(&mut self, ctx: EvaluatorContext, dedupe_threshold: usize) -> EvalResult<RD> {
        let o = self.op_encode(&mut EncodeContext::new()).join_threshold(dedupe_threshold);
        Evaluator::new(&mut o.as_ref(), ctx).run(RD::Unit())
    }
}



pub trait OpTreeImpl {
    fn as_mut_op_tree(&mut self) -> &mut OpTree;
    fn join(&mut self) -> Vec<u8> {
        self.join_threshold(10)
    }
    fn join_threshold(&mut self, threshold: usize) -> Vec<u8> {
        OpTreeDedup {
            threshold,
            seen: Default::default()
        }.dedup(self.as_mut_op_tree())
    }
}
impl OpTreeImpl for OpTree {
    fn as_mut_op_tree(&mut self) -> &mut OpTree { self }
}

struct OpTreeDedup {
    threshold: usize,
    seen: BTreeMap<(usize, u64), (u16, u8)>
}

impl OpTreeDedup {
    pub fn join(&mut self, off: u16, replace: Option<(u64, u16)>, op: &mut OpTree) -> Vec<u8> {
        fn hash(v: &Vec<u8>) -> u64 {
            let mut h = DefaultHasher::new();
            h.write(v);
            h.finish()
        }
        match op {
            OpTree::Data(v) => v.clone(),
            OpTree::LengthPrefix(o) => {
                let v = self.join(off + 2, replace, o);
                [(v.len() as u16).rd_encode(), v].concat()
            },
            OpTree::Op(opcode, ops) => {

                // Create output vec
                let mut out = opcode.as_slice().to_vec();
                ops.iter_mut().for_each(|o| out.extend(self.join(off + out.len() as u16, replace, o)));
                let h = hash(&out);

                // Check if replace
                if opcode.is_some() {
                    if let Some((r_hash, r_off)) = replace {
                        if r_hash == h && r_off < off {
                            let mut v = vec![OP::FETCH().into()];
                            v.extend(r_off.rd_encode());
                            *op = OpTree::Data(v.clone());
                            return v;
                        }
                    } else if out.len() > self.threshold {
                        self.seen.entry((out.len(), h)).and_modify(|r| r.1 += 1).or_insert((off, 1));
                    }
                }

                out
            }
        }
    }

    pub fn dedup(&mut self, tree: &mut OpTree) -> Vec<u8> {
        loop {
            // recreate dupe map
            self.seen = Default::default();
            let out = self.join(0, None, tree);

            // remove non dupes
            self.seen = self.seen.clone().into_iter().filter(|e| e.1.1 > 1).collect();

            if let Some(((_, h), (off, _))) = self.seen.last_key_value() {
                self.join(0, Some((*h, *off)), tree);
            } else {
                return out;
            }
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use crate::api::*;

    #[test]
    fn test_dedup() {
        let big = "1111".to_string();
        let mut comp = big.rr().equals(big).choose(112u8, 2);
        assert_eq!(comp.eval().unwrap()._as::<u8>(), 112);
        assert_eq!(comp.eval_with_context(Default::default(), 5).unwrap()._as::<u8>(), 112);
    }
}
