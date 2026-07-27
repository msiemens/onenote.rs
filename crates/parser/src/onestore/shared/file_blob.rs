use crate::fs::FileSource;
use crate::fs::file_source::BytesSource;
use bytes::Bytes;
use std::fmt::Debug;
use std::io::{Cursor, Read};
use std::sync::Arc;

/// Availability of binary data referenced by a OneNote object.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileDataStatus {
    /// The binary payload is available.
    Available,
    /// The OneNote object does not reference a binary payload.
    Missing,
    /// The OneNote object explicitly marks the binary payload as invalid.
    Invalid,
}

/// A reference to a contiguous run of bytes within a parser-managed
/// [`FileSource`].
///
/// Carries an `Arc` to the source plus an `(offset, size)` slice. Cloning
/// is a refcount bump; the underlying bytes are never copied. Many
/// attachment references can be held simultaneously without proportional
/// memory cost.
///
/// [`FileBlob::read`] gives a [`Read`] over the blob without
/// materialising it; [`FileBlob::to_bytes`] returns an owned `Bytes`
/// (zero-copy when the backing is in-memory).
#[derive(Clone)]
pub(crate) struct FileBlob {
    source: Arc<dyn FileSource>,
    offset: u64,
    size: u64,
    status: FileDataStatus,
}

impl FileBlob {
    /// Construct a `FileBlob` over a slice of a [`FileSource`].
    pub(crate) fn from_source(source: Arc<dyn FileSource>, offset: u64, size: u64) -> Self {
        Self {
            source,
            offset,
            size,
            status: FileDataStatus::Available,
        }
    }

    /// Construct a `FileBlob` from a stand-alone [`Bytes`] buffer (e.g.
    /// FSSHTTPB wire data that was decoded from the network rather than
    /// pulled from a file).
    pub(crate) fn empty() -> Self {
        Self {
            source: Arc::new(BytesSource::new(Bytes::new())),
            offset: 0,
            size: 0,
            status: FileDataStatus::Available,
        }
    }

    pub(crate) fn missing() -> Self {
        Self {
            status: FileDataStatus::Missing,
            ..Self::empty()
        }
    }

    pub(crate) fn invalid() -> Self {
        Self {
            status: FileDataStatus::Invalid,
            ..Self::empty()
        }
    }

    pub(crate) fn status(&self) -> FileDataStatus {
        self.status
    }

    /// The size of the blob in bytes.
    pub(crate) fn size(&self) -> u64 {
        self.size
    }

    /// A [`Read`] over the blob.
    ///
    /// For in-memory backings the read cursor is over a refcount-shared
    /// slice; for lazy-read backings the bytes are fetched on demand in
    /// chunks sized by the caller's read buffer.
    pub(crate) fn read(&self) -> Box<dyn Read> {
        if let Some(buf) = self.source.as_bytes() {
            let start = self.offset as usize;
            let end = start + self.size as usize;
            return Box::new(Cursor::new(buf.slice(start..end)));
        }
        Box::new(FileBlobReader {
            source: self.source.clone(),
            offset: self.offset,
            end: self.offset + self.size,
        })
    }
}

impl Debug for FileBlob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.status != FileDataStatus::Available {
            return write!(f, "FileBlob [ {:?}; 0 KiB ]", self.status);
        }

        // Read up to 32 bytes from each end via `read_at` so the preview is
        // identical regardless of whether the source is in-memory or
        // lazy-backed. Errors fall back to a size-only line.
        let len = self.size as usize;
        let head_len = len.min(32);
        let tail_offset = len.saturating_sub(32);
        let tail_len = len - tail_offset;

        let head = self.source.read_at(self.offset, head_len);
        let tail = self
            .source
            .read_at(self.offset + tail_offset as u64, tail_len);

        match (head, tail) {
            (Ok(head), Ok(tail)) => {
                let first_32: String = head.iter().map(|b| format!("{:02x}", b)).collect();
                let last_32: String = tail.iter().map(|b| format!("{:02x}", b)).collect();
                write!(
                    f,
                    "FileBlob [ {} ... {}; {:?} KiB ]",
                    first_32,
                    last_32,
                    len / 1024
                )
            }
            _ => write!(f, "FileBlob [ <read error>; {:?} KiB ]", len / 1024),
        }
    }
}

impl PartialEq for FileBlob {
    fn eq(&self, other: &Self) -> bool {
        // Compare by identity of the underlying source and slice range.
        // Cheap and good enough for the parser's needs (deduplicating
        // references to the same attachment).
        Arc::ptr_eq(&self.source, &other.source)
            && self.offset == other.offset
            && self.size == other.size
            && self.status == other.status
    }
}

impl Eq for FileBlob {}

impl Default for FileBlob {
    fn default() -> Self {
        Self::empty()
    }
}

struct FileBlobReader {
    source: Arc<dyn FileSource>,
    offset: u64,
    end: u64,
}

impl Read for FileBlobReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remaining = self.end.saturating_sub(self.offset);
        if remaining == 0 {
            return Ok(0);
        }
        let n = (buf.len() as u64).min(remaining) as usize;
        let bytes = self.source.read_at(self.offset, n)?;
        buf[..n].copy_from_slice(&bytes);
        self.offset += n as u64;
        Ok(n)
    }
}
