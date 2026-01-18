use std::fmt::Debug;

use crate::errors::Result;
use crate::onestore::legacy::file_node::FileNodeData;
use crate::onestore::legacy::file_node::shared::ObjectDeclarationNode;
use crate::onestore::legacy::file_structure::FileNodeDataIterator;
use crate::onestore::legacy::objects::parse_context::ParseContext;
use crate::onestore::shared::compact_id::CompactId;

pub(crate) struct Object {
    pub(crate) data: crate::onestore::Object,
    pub(crate) compact_id: CompactId,
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl Object {
    pub(crate) fn try_parse(
        iterator: &mut FileNodeDataIterator,
        context: &ParseContext,
    ) -> Result<Option<Self>> {
        let current = iterator.peek();
        let result = match current {
            Some(FileNodeData::ObjectDeclaration2RefCountFND(data)) => {
                Some(Self::parse_from_declaration(data, context)?)
            }
            Some(FileNodeData::ObjectDeclaration2LargeRefCountFND(data)) => {
                Some(Self::parse_from_declaration(data, context)?)
            }
            Some(FileNodeData::ReadOnlyObjectDeclaration2RefCountFND(data)) => {
                Some(Self::parse_from_declaration(data, context)?)
            }
            Some(FileNodeData::ReadOnlyObjectDeclaration2LargeRefCountFND(data)) => {
                Some(Self::parse_from_declaration(data, context)?)
            }
            Some(FileNodeData::ObjectDeclarationFileData3RefCountFND(data)) => {
                Some(Self::parse_from_declaration(data, context)?)
            }
            Some(FileNodeData::ObjectDeclarationFileData3LargeRefCountFND(data)) => {
                Some(Self::parse_from_declaration(data, context)?)
            }
            Some(FileNodeData::ObjectDeclarationWithRefCountFNDX(data)) => {
                Some(Self::parse_from_declaration(data, context)?)
            }
            Some(_) => None,
            None => None,
        };

        // Ensure that the iterator always advances when parsing
        if result.is_some() {
            iterator.next();
        }

        Ok(result)
    }

    fn parse_from_declaration(
        declaration: &dyn ObjectDeclarationNode,
        context: &ParseContext,
    ) -> Result<Self> {
        let props = declaration.props().cloned().unwrap_or_default();
        let file_data = if let Some(info) = declaration.get_attachment_info() {
            Some(context.find_file_data(&info)?)
        } else {
            None
        };

        let data = crate::onestore::Object {
            jc_id: declaration.id(),
            props,
            mapping: context.id_map.clone(),
            file_data: file_data.cloned(),
            context_id: context.context_id,
        };
        Ok(Self {
            data,
            compact_id: declaration.compact_id(),
        })
    }
}
