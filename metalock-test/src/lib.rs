
#[cfg(test)]
mod fuzz;

#[cfg(test)]
mod cu;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
