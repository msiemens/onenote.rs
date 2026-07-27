//! A OneNote file parser.
//!
//! `onenote_parser` provides a high-level API to parse OneNote notebooks and
//! inspect sections, pages, and their contents. It implements the underlying
//! OneNote file format layers (FSSHTTPB, OneStore, and MS-ONE) and exposes a
//! stable surface for consumers through the [`Parser`] type.
//!
//! The parser supports OneNote files from both OneDrive downloads (FSSHTTP
//! packaging) and desktop OneNote applications (2016, 2019, LTSC, etc.).
//! It is **read-only** and does _not_ support writing or modifying OneNote files.
//!
//! # Usage
//!
//! ```no_run
//! use onenote_parser::Parser;
//! use typed_path::TypedPath;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut parser = Parser::new();
//! let notebook = parser.parse_notebook(TypedPath::derive("My Notebook.onetoc2"))?;
//! println!("sections: {}", notebook.entries().len());
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - `native-fs` (default): Enables the [`fs::NativeFs`] implementation
//!   for standard file system operations. Disable this feature if you want to
//!   provide a custom [`FileSystem`] implementation (e.g., for in-memory,
//!   virtual file systems, or WASM bindings).
//! - `backtrace`: Captures a `std::backtrace::Backtrace` on parse errors and
//!   exposes it via `std::error::Error::backtrace()`.
//! - `onepkg`: Enables [`Parser::parse_package`] for reading `.onepkg`
//!   notebook archives (CAB files containing a `.onetoc2` and its sections).
//!
//! # Architecture
//!
//! The code organization and architecture follows the OneNote file format which is
//! built from several layers of encodings:
//!
//! - `fsshttpb/`: This implements the FSSHTTP binary packaging format as specified
//!   in [\[MS-FSSHTTPB\]: Binary Requests for File Synchronization via SOAP Protocol].
//!   This is the packaging format used for files downloaded from OneDrive.
//! - `onestore/`: This implements the OneStore format as specified in
//!   [\[MS-ONESTORE\]: OneNote Revision Store File Format]. This layer handles the
//!   revision store containing all OneNote objects. It supports both the desktop
//!   file format (where the revision store is the file itself) and the FSSHTTP
//!   format (where the store is built from objects and revisions inside the package).
//! - `one/`: This implements the OneNote file format as specified in [\[MS-ONE\]:
//!   OneNote File Format]. This specifies how objects in a OneNote file are parsed
//!   from a OneStore revision file.
//! - `onenote/`: high-level API that resolves references between objects
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
//! The parser supports the following OneNote file formats:
//!
//! - **`.one`** – Section files containing the actual notes and content.
//! - **`.onetoc2`** – Table of contents files used to organize sections within a notebook.
//!
//! These files can be obtained from:
//! - **OneNote Desktop** (2016, 2019, LTSC, etc.)
//! - **OneDrive** (via the "Download Notebook" feature)
//! - **OneNote for Windows 10/11** (via `.one` export)
//! - **OneNote for Mac** (as backup files)
//!
//! # I/O behaviour
//!
//! With the default [`fs::NativeFs`] backend the notebook file is read
//! on demand via positional reads (`pread` on Unix, overlapped
//! `ReadFile` on Windows). The file's bytes never need to be resident
//! in process memory in their entirety — multi-GB notebooks parse with
//! a working set proportional to active reads, not file size. The
//! kernel page cache fronts repeated reads cheaply.
//!
//! Attachments returned by [`contents::Image`] / [`contents::EmbeddedFile`]
//! hold a refcount-shared reference to the underlying source and pull
//! bytes through the same lazy path when their reader is consumed.
//!
//! Custom [`FileSystem`] implementations can override
//! [`FileSystem::open_file`] with their own [`fs::FileSource`] (e.g. a
//! WASM-side `Blob`-backed reader) to avoid materialising the file in
//! memory.
//!
//! Modifying the underlying file while a parse is in progress — or
//! while any derived attachment is alive — is unsupported. The parse
//! may fail with [`MalformedOneStoreData`](errors::ErrorKind::MalformedOneStoreData)
//! or [`IO`](errors::ErrorKind::IO).
//!
//! # Path handling
//!
//! Paths cross two boundaries in this crate.
//!
//! **Inbound: caller → parser.** [`Parser::parse_notebook`] /
//! [`Parser::parse_section`] / [`Parser::parse_package`] take a
//! [`typed_path::TypedPath`], which carries a runtime
//! [`PathType`](typed_path::PathType) tag selecting Unix or Windows
//! parsing rules. Pick the encoding that matches your byte source:
//!
//! - From a host-shaped source (argv, `std::env`, `std::fs::read_dir`):
//!   match the host. On Unix bridge via
//!   [`OsStrExt::as_bytes`](std::os::unix::ffi::OsStrExt::as_bytes); on
//!   Windows go through `to_str()`. `TypedPath::derive` is a convenience
//!   for this case — it picks Windows iff the string starts with `\`,
//!   otherwise Unix.
//! - From bytes of unknown provenance: prefer
//!   `TypedPath::new(_, PathType::Windows)`. Windows parsing treats both
//!   `/` and `\` as separators, so component-level validation can't be
//!   bypassed by switching separators. Do **not** use `derive` here.
//!
//! **Outbound: parser → host.** The parser hands `TypedPath`s back to
//! the [`FileSystem`] impl, which is responsible for translating them
//! into whatever the underlying storage expects. The bundled
//! [`fs::NativeFs`] does an encoding-checked conversion (rejecting paths
//! with the wrong encoding at the boundary) and routes Windows opens
//! through the `\\?\` verbatim namespace to neutralise the DOS device-
//! name trap (`CON`, `COM1`, …). Custom impls own the equivalent
//! defence, see the security contract on the [`FileSystem`] trait.
//!
//! # Stability
//!
//! The public API follows semantic versioning and is intended to be stable.
//!
//! **Minimum Supported Rust Version (MSRV):** 1.85
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

mod debug;
pub mod errors;
pub mod fs;
mod fsshttpb;
mod one;
pub(crate) mod onenote;
#[cfg(feature = "onepkg")]
mod onepkg;
mod onestore;
mod reader;
mod shared;
mod utils;
pub mod warn;

pub(crate) type Reader<'b> = &'b mut reader::Reader;

pub use crate::fs::FileSystem;
pub use crate::onenote::Parser;

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
    pub use crate::onenote::ink_recognition::{
        InkRecognition, InkRecognizedLine, InkRecognizedWord,
    };
    pub use crate::onenote::list::List;
    pub use crate::onenote::math_inline_object::{MathInlineObject, MathObjectType};
    pub use crate::onenote::note_tag::NoteTag;
    pub use crate::onenote::outline::{Outline, OutlineElement, OutlineGroup, OutlineItem};
    pub use crate::onenote::rich_text::{
        EmbeddedInkContainer, EmbeddedInkSpace, EmbeddedObject, ParagraphStyling, RichText,
    };
    pub use crate::onenote::table::{Table, TableCell, TableRow};
    pub use crate::onestore::shared::file_blob::FileDataStatus;
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
