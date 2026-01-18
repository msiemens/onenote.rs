//! A OneNote file parser.
//!
//! `onenote_parser` provides a high-level API to parse OneNote notebooks and
//! inspect sections, pages, and their contents. It implements the underlying
//! OneNote file format layers (FSSHTTPB, OneStore, and MS-ONE) and exposes a
//! stable surface for consumers through the [`Parser`] type.
//!
//! The parser targets OneNote files obtained from OneDrive downloads (FSSHTTP
//! packaging). It is read-only and does not aim to support legacy OneNote 2016
//! desktop files.
//!
//! # Usage
//!
//! ```no_run
//! use onenote_parser::Parser;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut parser = Parser::new();
//! let notebook = parser.parse_notebook(Path::new("My Notebook.onetoc2"))?;
//! println!("sections: {}", notebook.entries().len());
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - `backtrace`: Captures a `std::backtrace::Backtrace` on parse errors and
//!   exposes it via `std::error::Error::backtrace()`.
//!
//! # Architecture
//!
//! The parser mirrors the OneNote file format layers:
//! - FSSHTTPB: binary packaging used by OneDrive downloads
//! - OneStore: revision store embedded in the package
//! - MS-ONE: object model for sections, pages, and content
//! - `onenote`: high-level API that resolves references between objects
//!
//! # Error handling
//!
//! Most fallible APIs return [`errors::Result`], which wraps an [`errors::Error`]
//! containing an error kind. You can format the error for user-facing messages
//! and (with the `backtrace` feature enabled) access the captured backtrace via
//! `std::error::Error::backtrace()`.
//!
//! # Input files
//!
//! Use `.onetoc2` and `.one` files from OneDrive downloads (FSSHTTP packaging). For `.onetoc2`
//! files, the parser expects the `.one` file to be in the same directory. The parser does not
//! support legacy OneNote 2016 desktop files.
//!
//! # Stability
//!
//! The public API follows semantic versioning and is intended to be stable.
//!
//! # References
//!
//! - [\[MS-ONESTORE\]: OneNote Revision Store File Format]
//! - [\[MS-ONE\]: OneNote File Format]
//! - [\[MS-FSSHTTPB\]: Binary Requests for File Synchronization via SOAP Protocol]
//!
//! [\[MS-ONESTORE\]: OneNote Revision Store File Format]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/ae670cd2-4b38-4b24-82d1-87cfb2cc3725
//! [\[MS-ONE\]: OneNote File Format]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/73d22548-a613-4350-8c23-07d15576be50
//! [\[MS-FSSHTTPB\]: Binary Requests for File Synchronization via SOAP Protocol]: https://docs.microsoft.com/en-us/openspecs/sharepoint_protocols/ms-fsshttpb/f59fc37d-2232-4b14-baac-25f98e9e7b5a

#![warn(missing_docs)]
#![deny(unused_must_use)]
#![cfg_attr(feature = "backtrace", feature(error_generic_member_access))]

#[macro_use]
mod macros;

pub mod errors;
mod fsshttpb;
mod one;
mod onenote;
mod onestore;
mod reader;
mod shared;
mod utils;

pub(crate) type Reader<'a, 'b> = &'b mut reader::Reader<'a>;

pub use crate::onenote::Parser;
use std::path::Path;

pub trait FileSystem: Send + Sync {
    fn is_directory(&self, path: &Path) -> Result<bool, std::io::Error>;

    fn read_dir(&self, path: &Path) -> Result<Vec<String>, std::io::Error>;

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, std::io::Error>;

    fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), std::io::Error>;

    fn make_dir(&self, path: &Path) -> Result<(), std::io::Error>;

    fn exists(&self, path: &Path) -> Result<bool, std::io::Error>;

    // TODO: Compat layer!?
    // These functions correspond to the similarly-named
    // NodeJS path functions and should behave like the NodeJS
    // functions (rather than the corresponding Rust functions).
    fn get_file_name(&self, path: &Path) -> Option<String>;

    fn get_file_extension(&self, path: &Path) -> String;

    fn get_dir_name(&self, path: &Path) -> String;

    /// This function should behave like NodeJS's `path.join` function.
    /// As a result, unlike Rust's `Path::join`, if `path_2` starts with "/",
    /// `path_2` is still appended to `path_1`.
    fn join(&self, a: &Path, b: &Path) -> String;

    /// Splits filename into (base, extension).
    fn split_file_name(&self, _filename: &Path) -> (String, String) {
        unimplemented!()

        // let ext = self.get_file_extension(filename);
        // let base = filename.strip_suffix(&ext).unwrap_or(filename);

        // (base.into(), ext)
    }

    fn remove_prefix<'a>(&self, _full_path: &Path, _prefix: &Path) -> &'a str {
        unimplemented!()

        // if let Some(without_prefix) = full_path.strip_prefix(prefix) {
        //     without_prefix
        // } else {
        //     full_path
        // }
    }

    fn get_output_path(&self, _input_dir: &Path, _output_dir: &Path, _file_path: &Path) -> String {
        unimplemented!()

        // let base_path = self.remove_prefix(file_path, input_dir);
        // let rebased_output = self.join(output_dir, base_path);
        // self.get_dir_name(&rebased_output)
    }

    fn get_parent_dir(&self, _path: &Path) -> Option<String> {
        unimplemented!()

        // let dir_name = self.get_dir_name(path);
        // let result = self.get_file_name(&dir_name);

        // result.filter(|value| !value.is_empty())
    }
}

/// The data that represents a OneNote notebook.
pub mod notebook {
    pub use crate::onenote::notebook::Notebook;
}

/// The data that represents a OneNote section.
pub mod section {
    pub use crate::onenote::section::{Section, SectionEntry, SectionGroup};
}

/// The data that represents a OneNote page.
pub mod page {
    pub use crate::onenote::page::{Page, Title};
    pub use crate::onenote::page_content::PageContent;
    pub use crate::onenote::page_series::PageSeries;
}

/// The data that represents the contents of a OneNote section.
pub mod contents {
    pub use crate::onenote::content::Content;
    pub use crate::onenote::embedded_file::EmbeddedFile;
    pub use crate::onenote::image::Image;
    pub use crate::onenote::ink::{Ink, InkBoundingBox, InkPoint, InkStroke};
    pub use crate::onenote::list::List;
    pub use crate::onenote::math_inline_object::{MathInlineObject, MathObjectType};
    pub use crate::onenote::note_tag::NoteTag;
    pub use crate::onenote::outline::{Outline, OutlineElement, OutlineGroup, OutlineItem};
    pub use crate::onenote::rich_text::{
        EmbeddedInkContainer, EmbeddedInkSpace, EmbeddedObject, ParagraphStyling, RichText,
    };
    pub use crate::onenote::table::{Table, TableCell, TableRow};
}

/// Collection of properties used by the OneNote file format.
pub mod property {
    /// Properties related to multiple types of objects.
    pub mod common {
        pub use crate::one::property::color::Color;
        pub use crate::one::property::color_ref::ColorRef;
    }

    /// Properties related to embedded files.
    pub mod embedded_file {
        pub use crate::one::property::file_type::FileType;
    }

    /// Properties related to note tags.
    pub mod note_tag {
        pub use crate::one::property::note_tag::{ActionItemStatus, ActionItemType};
        pub use crate::one::property::note_tag_property_status::NoteTagPropertyStatus;
        pub use crate::one::property::note_tag_shape::NoteTagShape;
        pub use crate::onenote::note_tag::NoteTagDefinition;
    }

    /// Properties related to rich-text content.
    pub mod rich_text {
        pub use crate::one::property::charset::Charset;
        pub use crate::one::property::paragraph_alignment::ParagraphAlignment;
        pub use crate::onenote::rich_text::ParagraphStyling;
    }
}
