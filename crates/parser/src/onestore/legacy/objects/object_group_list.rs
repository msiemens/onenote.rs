use crate::errors::Result;
use crate::fsshttpb::data::exguid::ExGuid;
use crate::onestore::legacy::file_node::FileNodeData;
use crate::onestore::legacy::file_structure::FileNodeDataIterator;
use crate::onestore::legacy::objects::global_id_table::GlobalIdTable;
use crate::onestore::legacy::objects::object::Object;
use crate::onestore::legacy::objects::parse_context::ParseContext;
use std::collections::HashMap;
use std::fmt::Debug;

/// See [MS-ONESTORE 2.1.13](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/607a84d4-5762-4a3e-9244-c91acddcf647)
pub(crate) struct ObjectGroupList {
    id: ExGuid,
    // TODO: Unused?
    // pub(crate) id_table: GlobalIdTable,
    pub(crate) objects: Vec<Object>,
}

impl Debug for ObjectGroupList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Don't include all fields -- avoid duplicate data in output
        f.debug_struct("ObjectGroupList")
            .field("id", &self.id)
            .field("objects", &self.objects)
            .finish()
    }
}

impl ObjectGroupList {
    pub(crate) fn try_parse_into<'a>(
        iterator: &mut FileNodeDataIterator<'a>,
        context: &'a ParseContext<'a>,
        objects: &mut HashMap<ExGuid, crate::onestore::Object>,
    ) -> Result<Option<()>> {
        let current = iterator.peek();
        if let Some(FileNodeData::ObjectGroupListReferenceFND(data)) = current {
            iterator.next();
            let mut list_iterator = data.list.iter_data();
            Self::parse_into(&mut list_iterator, context, objects)?;

            Ok(Some(()))
        } else if let Some(FileNodeData::ObjectGroupStartFND(_)) = current {
            Self::parse_into(iterator, context, objects)?;

            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    fn parse_into<'a>(
        iterator: &mut FileNodeDataIterator<'a>,
        context: &'a ParseContext<'a>,
        objects: &mut HashMap<ExGuid, crate::onestore::Object>,
    ) -> Result<()> {
        let _start = match iterator.next() {
            Some(FileNodeData::ObjectGroupStartFND(object)) => object,
            _ => {
                return Err(onestore_parse_error!(
                    "Object group lists must start with an ObjectGroupStartFND node."
                )
                .into());
            }
        };

        let id_table = GlobalIdTable::try_parse(iterator)?
            .ok_or_else(|| onestore_parse_error!("Global ID table not found in ObjectGroupList"))?;
        let parse_context = context.with_id_table(&id_table);

        let mut last_index = iterator.get_index();
        while let Some(item) = iterator.peek() {
            if matches!(item, FileNodeData::ObjectGroupEndFND) {
                break;
            } else if let FileNodeData::DataSignatureGroupDefinitionFND(_) = item {
                log::debug!("Ignoring DataSignatureGroupDefinitionFND");

                iterator.next();
            } else if let Some(object) = Object::try_parse(iterator, &parse_context)? {
                let id = id_table.resolve_id(&object.compact_id)?;
                objects.entry(id).or_insert(object.data);
            } else {
                return Err(onestore_parse_error!(
                    "Unexpected node in ObjectGroupList: {:?}",
                    item
                )
                .into());
            }

            let index = iterator.get_index();
            assert_ne!(index, last_index);
            last_index = index;
        }

        Ok(())
    }
}
