use crate::errors::{ErrorKind, Result};
use crate::one::property::PropertyType;
use crate::onestore::object::Object;

/// A charset representation.
///
/// See [\[MS-ONE 2.3.55\]].
///
/// [\[MS-ONE 2.3.55\]]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/64e2db6e-6eeb-443c-9ccf-0f72b37ba411
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub enum Charset {
    Ansi,
    Default,
    Symbol,
    Mac,
    ShiftJis,
    Hangul,
    Johab,
    Gb2312,
    ChineseBig5,
    Greek,
    Turkish,
    Vietnamese,
    Hebrew,
    Arabic,
    Baltic,
    Russian,
    Thai,
    EastEurope,
    OEM,
}

impl Charset {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Result<Option<Charset>> {
        object
            .props()
            .get(prop_type)
            .map(|value| {
                value.to_u8().ok_or_else(|| {
                    ErrorKind::MalformedOneNoteFileData("charset is not a u8".into())
                })
            })
            .transpose()?
            .map(|value| match value {
                0 => Ok(Charset::Ansi),
                1 => Ok(Charset::Default),
                2 => Ok(Charset::Symbol),
                77 => Ok(Charset::Mac),
                128 => Ok(Charset::ShiftJis),
                129 => Ok(Charset::Hangul),
                130 => Ok(Charset::Johab),
                134 => Ok(Charset::Gb2312),
                136 => Ok(Charset::ChineseBig5),
                161 => Ok(Charset::Greek),
                162 => Ok(Charset::Turkish),
                163 => Ok(Charset::Vietnamese),
                177 => Ok(Charset::Hebrew),
                178 => Ok(Charset::Arabic),
                186 => Ok(Charset::Baltic),
                204 => Ok(Charset::Russian),
                222 => Ok(Charset::Thai),
                238 => Ok(Charset::EastEurope),
                255 => Ok(Charset::OEM),
                _ => Err(ErrorKind::MalformedOneNoteFileData(
                    format!("invalid charset: {}", value).into(),
                )
                .into()),
            })
            .transpose()
    }
}
