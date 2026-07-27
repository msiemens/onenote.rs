use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property::file_type::FileType;
use crate::one::property_set::{embedded_file_container, embedded_file_node};
use crate::onenote::ParserContext;
use crate::onenote::note_tag::{NoteTag, parse_note_tags};
use crate::onestore::ObjectSpace;
use crate::onestore::shared::file_blob::{FileBlob, FileDataStatus};

/// An embedded file.
///
/// See [\[MS-ONE\] 2.2.32].
///
/// [\[MS-ONE\] 2.2.32]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/a665b5ad-ff40-4c0c-9e42-4b707254dc3f
#[derive(Clone, PartialEq, Debug)]
pub struct EmbeddedFile {
    pub(crate) filename: String,
    pub(crate) file_type: FileType,
    pub(crate) data: FileBlob,

    pub(crate) layout_max_width: Option<f32>,
    pub(crate) layout_max_height: Option<f32>,

    pub(crate) offset_horizontal: Option<f32>,
    pub(crate) offset_vertical: Option<f32>,

    pub(crate) note_tags: Vec<NoteTag>,
}

impl EmbeddedFile {
    /// The embedded file's original file name.
    ///
    /// See [\[MS-ONE\] 2.2.71].
    ///
    /// # Untrusted input
    ///
    /// This string is metadata from the parsed OneNote file and is
    /// fully controlled by whoever authored that file. It may contain
    /// path separators (`/`, `\`), parent-directory components (`..`),
    /// NUL bytes, Windows reserved device names (`CON`, `COM1`, ...),
    /// extremely long sequences, or arbitrary Unicode. **Consumers
    /// that use this value as part of a write path on disk must
    /// sanitise it themselves** — typically with `sanitize_filename`
    /// or equivalent. Passing it unmodified to `std::fs::File::create`
    /// is a path-traversal / device-handle vulnerability.
    ///
    /// [\[MS-ONE\] 2.2.71]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/9c3409c0-0d81-42a8-bd97-d02a5b130b7d
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// The file type.
    ///
    /// See [\[MS-ONE\] 2.3.62].
    ///
    /// [\[MS-ONE\] 2.3.62]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/112836a0-ed3b-4be1-bc4b-49f0f7b02295
    pub fn file_type(&self) -> &FileType {
        &self.file_type
    }

    /// A [`Read`] over the file's binary data.
    ///
    /// Bytes are pulled lazily from the underlying [`crate::fs::FileSource`],
    /// so the file is not materialised in memory just because you obtained
    /// the reader. With a memory-mapped backing the read is zero-copy; with
    /// a lazy backing each call to `read` triggers a fetch sized by the
    /// caller's buffer.
    ///
    /// For compatibility, unavailable data produces an empty reader. Call
    /// [`data_status`](Self::data_status) first to distinguish it from a
    /// valid zero-byte file.
    pub fn read(&self) -> Box<dyn std::io::Read> {
        self.data.read()
    }

    /// The size of the embedded file in bytes.
    pub fn size(&self) -> u64 {
        self.data.size()
    }

    /// Availability of the embedded file's binary data.
    ///
    /// Consumers should render a broken-content indicator when this is not
    /// [`FileDataStatus::Available`].
    pub fn data_status(&self) -> FileDataStatus {
        self.data.status()
    }

    /// The max width of the embedded file's icon in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.21].
    ///
    /// [\[MS-ONE\] 2.3.21]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/2561c763-93b8-4b64-b6c7-1b86335d5b85
    pub fn layout_max_width(&self) -> Option<f32> {
        self.layout_max_width
    }

    /// The max height of the embedded file's icon in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.23].
    ///
    /// [\[MS-ONE\] 2.3.23]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/ce514d53-1229-4e77-9908-ef8de1761ceb
    pub fn layout_max_height(&self) -> Option<f32> {
        self.layout_max_height
    }

    /// The horizontal offset from the page origin in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.18].
    ///
    /// [\[MS-ONE\] 2.3.18]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/5fb9e84a-c9e9-4537-ab14-e5512f24669a
    pub fn offset_horizontal(&self) -> Option<f32> {
        self.offset_horizontal
    }

    /// The vertical offset from the page origin in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.19].
    ///
    /// [\[MS-ONE\] 2.3.19]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/5c4992ba-1db5-43e9-83dd-7299c562104d
    pub fn offset_vertical(&self) -> Option<f32> {
        self.offset_vertical
    }

    /// Note tags for the embedded file.
    pub fn note_tags(&self) -> &[NoteTag] {
        &self.note_tags
    }
}

pub(crate) fn parse_embedded_file(
    file_id: ExGuid,
    space: &(impl ObjectSpace + ?Sized),
    ctx: &mut ParserContext,
) -> Result<EmbeddedFile> {
    let node_object = space
        .get_object(file_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("embedded file is missing".into()))?;
    let node = embedded_file_node::parse(node_object)?;

    // Helper function to create a fallback for corrupted files
    let create_fallback = |filename: String| -> Result<EmbeddedFile> {
        Ok(EmbeddedFile {
            filename,
            file_type: node.file_type,
            data: FileBlob::missing(),
            layout_max_width: node.layout_max_width,
            layout_max_height: node.layout_max_height,
            offset_horizontal: node.offset_from_parent_horiz,
            offset_vertical: node.offset_from_parent_vert,
            note_tags: parse_note_tags(&node.note_tags, space)?,
        })
    };

    // Check if we have the required fields to parse a valid embedded file
    let embedded_filename = match node.embedded_file_name {
        Some(name) => name,
        None => {
            warn!(
                ctx,
                "embedded file has no filename; preserving it as unavailable"
            );
            return create_fallback(String::new());
        }
    };

    let container_object_id = match node.embedded_file_container {
        Some(id) => id,
        None => {
            warn!(
                ctx,
                "embedded file has no data container; preserving file metadata"
            );
            return create_fallback(embedded_filename);
        }
    };

    let container_object = space.get_object(container_object_id).ok_or_else(|| {
        ErrorKind::MalformedOneNoteData("embedded file container is missing".into())
    })?;
    let container = embedded_file_container::parse(container_object)?;

    let file = EmbeddedFile {
        filename: embedded_filename,
        file_type: node.file_type,
        data: container.into_value(),
        layout_max_width: node.layout_max_width,
        layout_max_height: node.layout_max_height,
        offset_horizontal: node.offset_from_parent_horiz,
        offset_vertical: node.offset_from_parent_vert,
        note_tags: parse_note_tags(&node.note_tags, space)?,
    };

    if file.data_status() == FileDataStatus::Invalid {
        warn!(
            ctx,
            "embedded file payload is marked invalid; preserving file metadata"
        );
    }

    Ok(file)
}
