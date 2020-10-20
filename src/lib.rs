#![deny(unused_must_use)]
#![cfg_attr(feature = "backtrace", feature(backtrace))]

mod errors;
mod fsshttpb;
#[macro_use]
mod macros;
mod one;
mod onenote;
mod onestore;
mod reader;
mod types;
mod utils;

pub(crate) type Reader<'a, 'b> = &'b mut crate::reader::Reader<'a>;

pub use crate::errors::{Error, ErrorKind, Result};
pub use crate::onenote::parser::content::Content;
pub use crate::onenote::parser::embedded_file::EmbeddedFile;
pub use crate::onenote::parser::image::Image;
pub use crate::onenote::parser::list::List;
pub use crate::onenote::parser::note_tag::{NoteTag, NoteTagDefinition};
pub use crate::onenote::parser::notebook::Notebook;
pub use crate::onenote::parser::outline::{Outline, OutlineElement, OutlineGroup, OutlineItem};
pub use crate::onenote::parser::page::{Page, Title};
pub use crate::onenote::parser::page_content::PageContent;
pub use crate::onenote::parser::page_series::PageSeries;
pub use crate::onenote::parser::rich_text::{ParagraphStyling, RichText};
pub use crate::onenote::parser::section::{Section, SectionEntry, SectionGroup};
pub use crate::onenote::parser::table::{Table, TableCell, TableRow};
pub use crate::onenote::parser::Parser;

pub use crate::one::property::color::Color;
pub use crate::one::property::color_ref::ColorRef;
pub use crate::one::property::note_tag::ActionItemStatus;
pub use crate::one::property::paragraph_alignment::ParagraphAlignment;
pub use crate::one::property_set::embedded_file_node::FileType;
pub use crate::one::property_set::note_tag_shared_definition_container::{
    NoteTagPropertyStatus, NoteTagShape,
};
