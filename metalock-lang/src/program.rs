use std::{marker::PhantomData, usize};


use metalock_core::internal::*;
use metalock_core::vm::eval::*;
use metalock_core::vm::expr::*;

use crate::compile::*;


#[derive(Clone)]
pub struct Program<Input, Output> {
    pub schema: Schema,
    pub expr: RR<Output>,
    pub _p: PhantomData<Input>
}

impl<Input: SchemaType + Into<RD>, Output: SchemaType> Program<Input, Output> {
    pub fn from<A: IntoProgram<Input, Output>>(a: A) -> Self {
        let var = Var(VarId::new()).rr();
        let expr = a.build(var);

        let schema = Function::<Input, Output>::to_schema();
        Program { schema, expr, _p: Default::default() }
    }

    pub fn compile(&mut self) -> Vec<u8> {
        let mut ctx = EncodeContext::new();
        self.expr.op_encode(&mut ctx).join()
    }

    pub fn op_tree(&mut self) -> OpTree {
        let mut ctx = EncodeContext::new();
        self.expr.op_encode(&mut ctx)
    }

    pub fn run(&self, input: Input, context: EvaluatorContext) -> EvalResult<RD> {
        let mut other = self.clone();
        let code = other.op_tree().join_threshold(usize::MAX);
        let mut eval = Evaluator::new(&mut code.as_ref(), context);
        eval.run(input.into())
    }

    pub fn to_string(&mut self) -> String {
        let code = self.compile();
        let iter = self.schema.0.iter().chain(code.iter());
        iter.map(|byte| format!("{:02x}", byte)).collect()
    }
}


pub trait IntoProgram<I: SchemaType + Into<RD>, O: SchemaType>: Sized {
    fn build(&self, input: RR<I>) -> RR<O>;
    fn to_program(self) -> Program<I, O> {
        Program::from(self)
    }
}

impl<I: SchemaType + Into<RD>, O: SchemaType, F: Fn(RR<I>) -> RR<O>> IntoProgram<I, O> for F {
    fn build(&self, input: RR<I>) -> RR<O> {
        self(input)
    }
}



#[cfg(test)]
mod tests {

    use super::*;
    use crate::api::*;

    #[test]
    fn test_program() {

        fn prog(n: RR<u8>) -> RR<u8> {
            n.add(10)
        }

        let r: u8 = prog.to_program().run(1, Default::default()).unwrap()._as();
        assert_eq!(r, 11);
    }
}
