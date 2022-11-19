//! A OneNote file parser.

#![warn(missing_docs)]
#![deny(unused_must_use)]
#![cfg_attr(feature = "backtrace", feature(error_generic_member_access))]
#![cfg_attr(feature = "backtrace", feature(provide_any))]

pub mod errors;
mod fsshttpb;
#[macro_use]
mod macros;
mod one;
mod onenote;
mod onestore;
mod reader;
mod shared;
mod utils;

pub(crate) type Reader<'a, 'b> = &'b mut crate::reader::Reader<'a>;

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
    pub use crate::onenote::list::List;
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
