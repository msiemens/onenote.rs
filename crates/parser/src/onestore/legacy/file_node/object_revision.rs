use crate::Reader;
use crate::onestore::legacy::file_node::FileNodeDataRef;
use crate::onestore::legacy::file_node::shared::{ParseWithRef, read_property_set};
use crate::onestore::shared::compact_id::CompactId;
use crate::onestore::shared::object_prop_set::ObjectPropSet;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ObjectRevisionWithRefCountFNDX {
    oid: CompactId,
    f_has_oid_references: bool,
    f_has_osid_references: bool,
    property_set: ObjectPropSet,
    c_ref: u8,
}

impl<'a> ParseWithRef<'a> for ObjectRevisionWithRefCountFNDX {
    fn parse(reader: Reader, data_ref: &FileNodeDataRef) -> crate::errors::Result<Self> {
        let property_set = read_property_set(reader, data_ref)?;
        let oid = CompactId::parse(reader)?;
        let metadata = reader.get_u8()?;
        Ok(Self {
            oid,
            f_has_oid_references: metadata & 0x1 > 0,
            f_has_osid_references: metadata & 0x2 > 0,
            c_ref: (metadata & 0b1111_1100) >> 2,
            property_set,
        })
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ObjectRevisionWithRefCount2FNDX {
    oid: CompactId,
    f_has_oid_references: bool,
    f_has_osid_references: bool,
    property_set: ObjectPropSet,
    c_ref: u32,
}

impl<'a> ParseWithRef<'a> for ObjectRevisionWithRefCount2FNDX {
    fn parse(reader: Reader, data_ref: &FileNodeDataRef) -> crate::errors::Result<Self> {
        let property_set = read_property_set(reader, data_ref)?;
        let oid = CompactId::parse(reader)?;
        let metadata = reader.get_u32()?;
        Ok(Self {
            oid,
            f_has_oid_references: metadata & 0x1 > 0,
            f_has_osid_references: metadata & 0x2 > 0,
            c_ref: reader.get_u32()?,
            property_set,
        })
    }
}
