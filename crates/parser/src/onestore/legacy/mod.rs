//! A OneStore parsing implementation for the OneNote 2016 format.
//! Parses `.one` files and has partial support for `.onetoc2` files.

mod common;
mod file_node;
pub(crate) mod file_structure;
mod objects;
pub(crate) mod one_store_file;
pub(crate) mod parse;

pub(crate) use common::exguid::ExGuid;
