

mod types;
pub mod macros;
mod encode;
mod parse;
mod schema;
mod newval;
pub mod tlist;
mod data;


pub use schema::tag::{self, TagType};
pub use data::{RD, FromRD};
pub use types::{Buffer, Native, Schema, ParserBuffer, EncodedFunction, Buf, Decode};
pub use newval::{SchemaType, data_parse};
pub use encode::{Encode};


#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
