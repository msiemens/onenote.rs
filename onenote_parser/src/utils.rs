use crate::errors::{ErrorKind, Result};
use widestring::U16CString;

pub(crate) trait Utf16ToString {
    fn utf16_to_string(&self) -> Result<String>;
}

impl Utf16ToString for &[u8] {
    fn utf16_to_string(&self) -> Result<String> {
        let data: Vec<_> = self
            .chunks_exact(2)
            .map(|v| u16::from_le_bytes([v[0], v[1]]))
            .collect();

        let value = U16CString::from_vec_truncate(data);
        value.to_string().map_err(|err| {
            ErrorKind::MalformedOneNoteData(
                format!("UTF-16 string conversion failed: {err}").into(),
            )
            .into()
        })
    }
}
