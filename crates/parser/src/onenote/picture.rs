use crate::onestore::shared::file_blob::{FileBlob, FileDataStatus};

/// A stored picture representation associated with a OneNote object.
///
/// Some objects carry supplemental pictures, such as a browser-compatible
/// fallback for an image or the icon for an embedded file. The bytes remain
/// backed by the parser's lazy file source and are read only on demand.
#[derive(Clone, PartialEq, Debug)]
pub struct Picture {
    pub(crate) data: FileBlob,
    pub(crate) extension: Option<String>,
}

impl Picture {
    pub(crate) fn new(data: FileBlob, extension: Option<String>) -> Self {
        Self { data, extension }
    }

    /// A [`std::io::Read`] over the picture's binary data.
    pub fn read(&self) -> Box<dyn std::io::Read> {
        self.data.read()
    }

    /// The size of the picture in bytes.
    pub fn size(&self) -> u64 {
        self.data.size()
    }

    /// Availability of the picture's binary data.
    pub fn data_status(&self) -> FileDataStatus {
        self.data.status()
    }

    /// The picture's file extension, when stored by OneNote.
    pub fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::Picture;
    use crate::fs::file_source::BytesSource;
    use crate::onestore::shared::file_blob::{FileBlob, FileDataStatus};
    use bytes::Bytes;
    use std::io::Read;
    use std::sync::Arc;

    #[test]
    fn exposes_lazy_picture_data_and_metadata() {
        let source = Arc::new(BytesSource::new(Bytes::from_static(b"picture")));
        let picture = Picture::new(FileBlob::from_source(source, 0, 7), Some("png".to_owned()));

        let mut bytes = Vec::new();
        picture.read().read_to_end(&mut bytes).unwrap();

        assert_eq!(bytes, b"picture");
        assert_eq!(picture.size(), 7);
        assert_eq!(picture.data_status(), FileDataStatus::Available);
        assert_eq!(picture.extension(), Some("png"));
    }
}
