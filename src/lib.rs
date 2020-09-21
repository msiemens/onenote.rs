use bytes::Buf;

mod errors;
mod fsshttpb;
mod one;
mod onenote;
mod onestore;
mod types;
mod utils;

pub(crate) type Reader<'a> = &'a mut dyn Buf;

pub use crate::errors::Result;
pub use crate::onenote::parser::Parser;
