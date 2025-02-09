
#[cfg(test)]
mod tests {
    use quickcheck::Arbitrary;

    use metalock::prelude::*;
    use metalock::compile::*;

    use metalock_core::internal::vm;


    #[quickcheck]
    fn test_and(a: bool, b: bool) {
        RR::val(a).and(RR::val(b)).eval();
    }

    #[quickcheck]
    fn test_index(a: Vec<u8>, b: u16) {
        a.get(b).eval();
    }

    #[quickcheck]
    fn test_add(a: u16, b: u16) {
        a.add(b).eval();
    }
}
