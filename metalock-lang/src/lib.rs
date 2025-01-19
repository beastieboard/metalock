
pub mod prelude;
mod native;
mod api;
mod parse;
mod encode;
mod schema;
mod expr;
mod types;
mod eval;
mod newval;
pub mod profile;
pub mod program;
mod compile;

#[cfg(feature = "anchor")]
mod frontend;


#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
