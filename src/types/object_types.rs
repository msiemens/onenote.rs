use std::fmt;

use enum_primitive_derive::Primitive;
use num_traits::ToPrimitive;

#[derive(Debug, Primitive, PartialEq)]
pub enum ObjectType {
    CellManifest = 0x0B,
    DataElement = 0x01,
    DataElementFragment = 0x06A,
    DataElementPackage = 0x15,
    ObjectDataBlob = 0x02,
    ObjectGroupBlobReference = 0x1C,
    ObjectGroupData = 0x1E,
    ObjectGroupDataBlob = 0x05,
    ObjectGroupDataExcluded = 0x03,
    ObjectGroupDataObject = 0x16,
    ObjectGroupDeclaration = 0x1D,
    ObjectGroupMetadata = 0x078,
    ObjectGroupMetadataBlock = 0x79,
    ObjectGroupObject = 0x18,
    OneNotePackaging = 0x7a,
    RevisionManifest = 0x1A,
    RevisionManifestGroupReference = 0x19,
    RevisionManifestRoot = 0x0A,
    StorageIndexCellMapping = 0x0E,
    StorageIndexManifestMapping = 0x11,
    StorageIndexRevisionMapping = 0x0D,
    StorageManifest = 0x0C,
    StorageManifestRoot = 0x07,
}

impl fmt::LowerHex for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.to_u64().unwrap();
        fmt::LowerHex::fmt(&value, f)
    }
}
