use bytes::Buf;

mod errors;
mod fsshttpb;
#[macro_use]
mod macros;
mod one;
mod onenote;
mod onestore;
mod types;
mod utils;

pub(crate) type Reader<'a> = &'a mut dyn Buf;

pub use crate::errors::Result;
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
pub use crate::onenote::parser::section::Section;
pub use crate::onenote::parser::table::{Table, TableCell, TableRow};
pub use crate::onenote::parser::Parser;
