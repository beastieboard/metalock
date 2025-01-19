use std::{collections::BTreeMap, hash::{DefaultHasher, Hasher}};

use crate::encode::Encode;


const FETCH: u8 = 0x03;

#[derive(Debug, Clone)]
pub enum OpTree {
    Op(Option<u8>, Vec<OpTree>),
    LengthPrefix(Box<OpTree>),
    Data(Vec<u8>),
}

impl OpTree {
    pub fn join(&mut self) -> Vec<u8> {
        self.join_threshold(10)
    }
    pub fn join_threshold(&mut self, threshold: usize) -> Vec<u8> {
        OpTreeDedup {
            threshold,
            seen: Default::default()
        }.dedup(self)
    }
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
                            let mut v = vec![FETCH];
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
    use crate::{api::ToRRBool, expr::ToRR};

    #[test]
    fn test_dedup() {
        let big = "1111".to_string();
        let mut comp = big.rr().equals(big).choose(112u8, 2);
        assert_eq!(comp.eval()._as::<u8>(), 112);
        assert_eq!(comp.eval_with_context(Default::default(), 5)._as::<u8>(), 112);
    }
}
