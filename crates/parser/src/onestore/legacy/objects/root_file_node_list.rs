use super::file_data_store::FileDataStore;
use crate::errors::Result;
use crate::onestore::legacy::file_node::FileNodeData;
use crate::onestore::legacy::file_structure::FileNodeList;
use crate::onestore::legacy::objects::object_space::ObjectSpace;
use crate::onestore::legacy::objects::parse_context::ParseContext;

// See
// - [MS-ONESTORE 2.1.14](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/28e21c1f-94b6-4f98-9d81-2e1278ebefc6)
// - [MS-ONESTORE 1.3.2](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/e3f4f871-f674-4198-9cb8-d67e1eeac2f3)
#[derive(Debug)]
pub(crate) struct RootFileNodeList {
    pub(crate) root_object_space_index: usize,
    pub(crate) object_spaces: Vec<ObjectSpace>,
    // pub(crate) file_data_store: Option<FileDataStore>,
}

impl<'a> RootFileNodeList {
    pub(crate) fn root_object_space(&self) -> &ObjectSpace {
        &self.object_spaces[self.root_object_space_index]
    }

    pub(crate) fn parse(
        node_list: &'a FileNodeList,
        context: &'a ParseContext<'a>,
    ) -> Result<Self> {
        let mut file_data_store = None;

        let mut iterator = node_list.iter_data();
        loop {
            if iterator.peek().is_none() {
                break;
            }
            if let Some(data_store) = FileDataStore::try_parse(&mut iterator)? {
                if file_data_store.is_some() {
                    return Err(onestore_parse_error!(
                        "Only one file_data_store can exist in the root node list"
                    )
                    .into());
                }
                file_data_store = Some(data_store);
            } else {
                iterator.next();
            }
        }

        let (object_spaces, root_object_space_id) = {
            let context = if let Some(file_data_store) = &file_data_store {
                context.with_file_data_store(file_data_store)
            } else {
                context.clone()
            };

            let mut object_spaces = Vec::new();
            let mut root_object_space_id = None;

            let mut iterator = node_list.iter_data();
            let mut last_index: usize = iterator.get_index();
            while let Some(current) = iterator.peek() {
                if let Some(object_space) = ObjectSpace::try_parse(&mut iterator, &context)? {
                    object_spaces.push(object_space);
                } else if FileDataStore::try_parse(&mut iterator)?.is_some() {
                    // File data stores may appear anywhere in the list; skip in this pass.
                } else if let FileNodeData::ObjectSpaceManifestRootFND(data) = current {
                    iterator.next();
                    root_object_space_id = Some(data.gosid_root.into());
                } else {
                    return Err(onestore_parse_error!(
                        "Unexpected entry in the root file node list: {:?}",
                        current
                    )
                    .into());
                }

                let index = iterator.get_index();
                if index == last_index {
                    println!(
                        "Indexes equal: {} = {}. Parsing: {:?}",
                        index, index, current
                    )
                }
                assert_ne!(index, last_index);
                last_index = index;
            }

            (object_spaces, root_object_space_id)
        };

        let root_object_space_id = root_object_space_id.ok_or_else(|| {
            onestore_parse_error!("Root file node list did not contain a node with the root ID")
        })?;
        let root_object_space_index = object_spaces
            .iter()
            .position(|space| space.id == root_object_space_id)
            .ok_or_else(|| onestore_parse_error!("Should have a default object space"))?;

        Ok(Self {
            root_object_space_index,
            // file_data_store,
            object_spaces,
        })
    }
}
