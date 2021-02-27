use crate::errors::{ErrorKind, Result};
use crate::one::property::{simple, PropertyType};
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;
use itertools::Itertools;
use widestring::U16String;

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) alternatives: Vec<String>,
    pub(crate) language_code: Option<u32>,
}

pub(crate) fn parse(object: &Object) -> Result<Data> {
    if object.id() != PropertySetId::InkAnalysisWord.as_jcid() {
        return Err(ErrorKind::MalformedOneNoteFileData(
            format!("unexpected object type: 0x{:X}", object.id().0).into(),
        )
        .into());
    }

    let language_code = simple::parse_u16(PropertyType::InkAnalysisWordLanguageId, object)?
        .map(|value| value as u32);

    let alternatives = simple::parse_vec(PropertyType::InkAnalysisWordAlternatives, object)?
        .map(|data| {
            let data: Vec<_> = data
                .chunks_exact(2)
                .map(|v| u16::from_le_bytes([v[0], v[1]]))
                .collect();
            data.split(|c| *c == 0)
                .filter(|chars| !chars.is_empty())
                .map(|chars| {
                    U16String::from_vec(chars.to_vec())
                        .to_string()
                        .map_err(|e| e.into())
                })
                .collect::<Result<_>>()
        })
        .transpose()?
        .ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("ink analysis word has no alternatives".into())
        })?;

    Ok(Data {
        alternatives,
        language_code,
    })
}
