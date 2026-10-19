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
///
/// The [`Display`](fmt::Display) output is a lowercase, hyphenated UUID (for
/// example `af6de84b-d817-4f07-8baf-fb6629cb7c2c`). This format is stable, so
/// it is safe to persist.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_a_lowercase_hyphenated_uuid() {
        let identity = FileIdentity::new(guid!("{AF6DE84B-D817-4F07-8BAF-FB6629CB7C2C}"));

        assert_eq!(identity.to_string(), "af6de84b-d817-4f07-8baf-fb6629cb7c2c");
    }
}
