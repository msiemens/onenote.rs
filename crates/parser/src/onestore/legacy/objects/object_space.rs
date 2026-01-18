use std::collections::HashMap;

use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::onestore::Object;
use crate::onestore::legacy::file_node::FileNodeData;
use crate::onestore::legacy::file_node::object_space_manifest::ObjectSpaceManifestListReferenceFND;
use crate::onestore::legacy::file_structure::FileNodeDataIterator;
use crate::onestore::legacy::objects::parse_context::ParseContext;
use crate::onestore::legacy::objects::revision::RootRole;
use crate::onestore::legacy::objects::revision_manifest_list::RevisionManifestList;

/// A collection of objects, referenced from the root file node list.
///
/// See [\[MS-ONESTORE\] 2.1.4](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/1329433f-02a5-4e83-ab41-80d57ade38d9)
#[derive(Debug)]
pub(crate) struct ObjectSpace {
    pub(crate) id: ExGuid,
    roots: HashMap<RootRole, ExGuid>,
    objects: HashMap<ExGuid, Object>,
}

impl ObjectSpace {
    pub(crate) fn try_parse<'a>(
        iterator: &mut FileNodeDataIterator<'a>,
        context: &ParseContext<'a>,
    ) -> Result<Option<Self>> {
        let next = iterator.peek();

        match next {
            Some(FileNodeData::ObjectSpaceManifestListReferenceFND(list_reference)) => {
                iterator.next();
                Ok(Some(Self::parse(iterator, list_reference, context)?))
            }
            _ => Ok(None),
        }
    }

    fn parse<'a>(
        _iterator: &mut FileNodeDataIterator<'a>,
        list_reference: &ObjectSpaceManifestListReferenceFND,
        context: &ParseContext<'a>,
    ) -> Result<Self> {
        let id = list_reference.gosid.into();
        let context = &context.with_context_id(id);
        let mut list_iterator = list_reference.last_revision.list.iter_data();
        let mut roots = HashMap::new();
        let mut objects = HashMap::new();

        let parsed = RevisionManifestList::try_parse_into(
            &mut list_iterator,
            context,
            &mut roots,
            &mut objects,
        )?;

        if parsed.is_none() {
            return Err(ErrorKind::MalformedOneStoreData(
                "ObjectSpace should point to a RevisionManifestList".into(),
            )
            .into());
        }

        Ok(Self { id, roots, objects })
    }
}

impl crate::onestore::ObjectSpace for ObjectSpace {
    fn get_object(&self, id: ExGuid) -> Option<&Object> {
        self.objects.get(&id)
    }

    fn content_root(&self) -> Option<ExGuid> {
        self.roots.get(&RootRole::DefaultContent).copied()
    }

    fn metadata_root(&self) -> Option<ExGuid> {
        self.roots.get(&RootRole::MetadataRoot).copied()
    }
}
