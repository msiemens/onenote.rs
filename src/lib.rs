use bytes::{Buf, Bytes};

use crate::errors::Result;
use crate::types::packaging::Packaging;

mod data;
mod errors;
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

    pub fn parse(&mut self) -> Result<Packaging> {
        Packaging::parse(&mut self.data)
    }
}
