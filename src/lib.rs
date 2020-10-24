//! A OneNote file parser.

#![warn(missing_docs)]
#![deny(unused_must_use)]
#![cfg_attr(feature = "backtrace", feature(backtrace))]

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

pub use crate::onenote::parser::Parser;

/// The data that represents a OneNote notebook.
pub mod notebook {
    pub use crate::onenote::parser::notebook::Notebook;
}

/// The data that represents a OneNote section.
pub mod section {
    pub use crate::onenote::parser::section::{Section, SectionEntry, SectionGroup};
}

/// The data that represents a OneNote page.
pub mod page {
    pub use crate::onenote::parser::page::{Page, Title};
    pub use crate::onenote::parser::page_content::PageContent;
    pub use crate::onenote::parser::page_series::PageSeries;
}

/// The data that represents the contents of a OneNote section.
pub mod contents {
    pub use crate::onenote::parser::content::Content;
    pub use crate::onenote::parser::embedded_file::EmbeddedFile;
    pub use crate::onenote::parser::image::Image;
    pub use crate::onenote::parser::list::List;
    pub use crate::onenote::parser::note_tag::NoteTag;
    pub use crate::onenote::parser::outline::{Outline, OutlineElement, OutlineGroup, OutlineItem};
    pub use crate::onenote::parser::rich_text::RichText;
    pub use crate::onenote::parser::table::{Table, TableCell, TableRow};
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
        pub use crate::one::property_set::embedded_file_node::FileType;
    }

    /// Properties related to note tags.
    pub mod note_tag {
        pub use crate::one::property::note_tag::{ActionItemStatus, ActionItemType};
        pub use crate::one::property_set::note_tag_shared_definition_container::{
            NoteTagPropertyStatus, NoteTagShape,
        };
        pub use crate::onenote::parser::note_tag::NoteTagDefinition;
    }

    /// Properties related to rich-text content.
    pub mod rich_text {
        pub use crate::one::property::charset::Charset;
        pub use crate::one::property::paragraph_alignment::ParagraphAlignment;
        pub use crate::onenote::parser::rich_text::ParagraphStyling;
    }
}
