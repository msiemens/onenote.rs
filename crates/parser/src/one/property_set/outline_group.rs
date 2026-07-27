use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property::{PropertyType, simple};
use crate::one::property_set::{PropertySetId, assert_property_set};
use crate::onestore::Object;

/// An outline group.
///
/// See [\[MS-ONE\] 2.2.22].
///
/// [\[MS-ONE\] 2.2.22]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/7dcc1618-46ee-4912-b918-ab4df1b52315
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct Data {
    pub(crate) last_modified: Option<Time>,
    pub(crate) children: Vec<ExGuid>,
    pub(crate) child_level: u8,
}

pub(crate) fn parse(object: &Object) -> Result<Data> {
    assert_property_set(object, PropertySetId::OutlineGroup)?;

    let last_modified = Time::parse(PropertyType::LastModifiedTime, object)?;
    let children =
        ObjectReference::parse_vec(PropertyType::ElementChildNodes, object)?.unwrap_or_default();
    let child_level = simple::parse_u8(PropertyType::OutlineElementChildLevel, object)?
        .ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("outline group has no child level".into())
        })?;

    let data = Data {
        last_modified,
        children,
        child_level,
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
            None
        }

        fn get_object_space(&self, _index: usize, _cid: &CompactId) -> Option<CellId> {
            None
        }
    }

    #[test]
    fn accepts_a_missing_last_modified_time() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&(PropertyType::OutlineElementChildLevel as u32).to_le_bytes());
        bytes.push(0);

        let mut reader = Reader::new(&bytes);
        let object = Object {
            context_id: Default::default(),
            jc_id: PropertySetId::OutlineGroup.as_jcid(),
            props: ObjectPropSet {
                properties: PropertySet::parse(&mut reader).unwrap(),
                ..Default::default()
            },
            file_data: None,
            mapping: Rc::new(TestMapping),
        };

        let data = parse(&object).expect("missing unused timestamp should be non-fatal");

        assert_eq!(data.last_modified, None);
        assert_eq!(data.child_level, 0);
    }
}
