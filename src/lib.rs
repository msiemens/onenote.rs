use bytes::{Buf, Bytes};

use crate::errors::Result;
use crate::fsshttpb::packaging::Packaging;
pub use crate::one::data::*;
use crate::onestore::parse_store;
use std::process::abort;

mod errors;
mod fsshttpb;
mod one;
mod onestore;
mod types;

type Reader<'a> = &'a mut dyn Buf;

pub struct Parser {
    data: Bytes,
}

impl Parser {
    pub fn new(data: Vec<u8>) -> Parser {
        Parser {
            data: Bytes::from(data),
        }
    }

    pub fn parse(&mut self) -> Result<Notebook> {
        // FIXME: Parse onetoc2 file and all sections
        unimplemented!();
    }

    pub fn parse_section(&mut self) -> Result<Section> {
        let packaging = Packaging::parse(&mut self.data)?;
        let store = parse_store(packaging)?;

        println!("{:#?}", store);

        abort();
    }
}
