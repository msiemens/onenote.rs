use crate::errors::{ErrorKind, Result};
use crate::one::property::PropertyType;
use crate::onestore::object::Object;

/// A page size declaration.
///
/// See [\[MS-ONE\] 2.3.36].
///
/// [\[MS-ONE\] 2.3.36]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/8866c05a-602d-4868-95de-2d8b1a0b9d2e
#[derive(Debug)]
pub(crate) enum PageSize {
    Auto,
    Us,
    AnsiLetter,
    AnsiTabloid,
    UsLegal,
    IsoA3,
    IsoA4,
    IsoA5,
    IsoA6,
    JisB4,
    JisB5,
    JisB6,
    JapanesePostcard,
    IndexCard,
    Billfold,
    Custom,
}

impl PageSize {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Result<Option<PageSize>> {
        object
            .props()
            .get(prop_type)
            .map(|value| {
                value
                    .to_u8()
                    .ok_or_else(|| {
                        ErrorKind::MalformedOneNoteFileData("page size is not a u8".into())
                    })
                    .and_then(|value| match value {
                        0 => Ok(PageSize::Auto),
                        1 => Ok(PageSize::Us),
                        2 => Ok(PageSize::AnsiLetter),
                        3 => Ok(PageSize::AnsiTabloid),
                        4 => Ok(PageSize::UsLegal),
                        5 => Ok(PageSize::IsoA3),
                        6 => Ok(PageSize::IsoA4),
                        7 => Ok(PageSize::IsoA5),
                        8 => Ok(PageSize::IsoA6),
                        9 => Ok(PageSize::JisB4),
                        10 => Ok(PageSize::JisB5),
                        11 => Ok(PageSize::JisB6),
                        12 => Ok(PageSize::JapanesePostcard),
                        13 => Ok(PageSize::IndexCard),
                        14 => Ok(PageSize::Billfold),
                        15 => Ok(PageSize::Custom),
                        _ => Err(ErrorKind::MalformedOneNoteFileData(
                            format!("invalid page size: {}", value).into(),
                        )),
                    })
            })
            .transpose()
            .map_err(|e| e.into())
    }
}

impl Default for PageSize {
    fn default() -> Self {
        PageSize::Auto
    }
}
