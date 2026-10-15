use crate::shared::guid::Guid;
use std::fmt;

/// The identity stored in a OneNote file header.
///
/// A file identity remains the same when the corresponding `.one` or
/// `.onetoc2` file is renamed or moved. It is suitable for associating
/// application state with a notebook, section, or section group without
/// depending on its display name or position.
///
/// Treat the value as opaque. OneNote may assign a new identity when it creates
/// a replacement file, even if that file represents the same logical content.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FileIdentity(Guid);

impl FileIdentity {
    pub(crate) fn new(value: Guid) -> Self {
        Self(value)
    }
}

impl fmt::Display for FileIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0.0.hyphenated(), formatter)
    }
}
