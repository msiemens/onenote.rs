use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property::{PropertyType, simple};
use crate::one::property_set::{PropertySetId, assert_property_set};
use crate::onestore::Object;

/// An outline element.
///
/// See [\[MS-ONE\] 2.2.21].
///
/// [\[MS-ONE\] 2.2.21]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/d47760a6-6f1f-4fd5-b2ad-a51fe5a72c21
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct Data {
    pub(crate) created_at: Option<Time>,
    pub(crate) last_modified: Time,
    pub(crate) children: Vec<ExGuid>,
    pub(crate) child_level: u8,
    pub(crate) contents: Vec<ExGuid>,
    pub(crate) list_contents: Vec<ExGuid>,
    pub(crate) list_spacing: Option<f32>,
    pub(crate) author_original: ExGuid,
    pub(crate) author_most_recent: ExGuid,
    pub(crate) rtl: bool,
    pub(crate) is_deletable: bool,
    pub(crate) is_selectable: bool,
    pub(crate) is_title_text: bool,
}

pub(crate) fn parse(object: &Object) -> Result<Data> {
    assert_property_set(object, PropertySetId::OutlineElementNode)?;

    let created_at = Time::parse(PropertyType::CreationTimeStamp, object)?;
    let last_modified = Time::parse(PropertyType::LastModifiedTime, object)?.ok_or_else(|| {
        ErrorKind::MalformedOneNoteFileData("outline element has no last modified time".into())
    })?;
    let children =
        ObjectReference::parse_vec(PropertyType::ElementChildNodes, object)?.unwrap_or_default();
    let child_level = simple::parse_u8(PropertyType::OutlineElementChildLevel, object)?
        .ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("outline has no child element level".into())
        })?;
    let contents =
        ObjectReference::parse_vec(PropertyType::ContentChildNodes, object)?.unwrap_or_default();
    let list_contents =
        ObjectReference::parse_vec(PropertyType::ListNodes, object)?.unwrap_or_default();
    let list_spacing = simple::parse_f32(PropertyType::ListSpacingMu, object)?;
    let author_original = ObjectReference::parse(PropertyType::AuthorOriginal, object)?
        .ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("outline element has no original author".into())
        })?;
    let author_most_recent = ObjectReference::parse(PropertyType::AuthorMostRecent, object)?
        .ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("outline element has no most recent author".into())
        })?;
    let rtl = simple::parse_bool(PropertyType::OutlineElementRtl, object)?.unwrap_or_default();
    let is_deletable = simple::parse_bool(PropertyType::Deletable, object)?.unwrap_or_default();
    let is_selectable = simple::parse_bool(PropertyType::CannotBeSelected, object)?
        .map(|value| !value)
        .unwrap_or_default();
    let is_title_text = simple::parse_bool(PropertyType::IsTitleText, object)?.unwrap_or_default();

    let data = Data {
        created_at,
        last_modified,
        children,
        child_level,
        contents,
        list_contents,
        list_spacing,
        author_original,
        author_most_recent,
        rtl,
        is_deletable,
        is_selectable,
        is_title_text,
    };

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::fsshttpb::data::cell_id::CellId;
    use crate::fsshttpb::data::exguid::ExGuid;
    use crate::one::property::PropertyType;
    use crate::one::property_set::PropertySetId;
    use crate::onestore::shared::compact_id::CompactId;
    use crate::onestore::shared::object_prop_set::ObjectPropSet;
    use crate::onestore::shared::prop_set::PropertySet;
    use crate::onestore::{MappingTable, Object};
    use crate::reader::Reader;
    use std::rc::Rc;

    struct TestMapping;

    impl MappingTable for TestMapping {
        fn resolve_id(&self, _index: usize, _cid: &CompactId) -> Option<ExGuid> {
            Some(ExGuid::default())
        }

        fn get_object_space(&self, _index: usize, _cid: &CompactId) -> Option<CellId> {
            None
        }
    }

    #[test]
    fn accepts_a_missing_creation_timestamp() {
        let property_ids = [
            PropertyType::LastModifiedTime,
            PropertyType::OutlineElementChildLevel,
            PropertyType::AuthorOriginal,
            PropertyType::AuthorMostRecent,
        ];
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(property_ids.len() as u16).to_le_bytes());
        for property_id in property_ids {
            bytes.extend_from_slice(&(property_id as u32).to_le_bytes());
        }
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.push(0);

        let mut reader = Reader::new(&bytes);
        let object = Object {
            context_id: Default::default(),
            jc_id: PropertySetId::OutlineElementNode.as_jcid(),
            props: ObjectPropSet {
                object_ids: vec![
                    CompactId {
                        n: 0,
                        guid_index: 0,
                    },
                    CompactId {
                        n: 1,
                        guid_index: 0,
                    },
                ],
                properties: PropertySet::parse(&mut reader).unwrap(),
                ..Default::default()
            },
            file_data: None,
            mapping: Rc::new(TestMapping),
        };

        let data = parse(&object).expect("missing unused timestamp should be non-fatal");

        assert_eq!(data.created_at, None);
        assert_eq!(data.child_level, 0);
    }
}
