use super::parse_context::ParseContext;
use crate::errors::ErrorKind;
use crate::errors::Result;
use crate::onestore::legacy::common::FileChunkReference64x32;
use crate::onestore::legacy::file_node::FileNode;
use crate::onestore::legacy::file_node::FileNodeData;
use crate::onestore::legacy::parse::Parse;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct FileNodeListFragment {
    pub(crate) header: FileNodeListHeader,
    pub(crate) file_nodes: Vec<FileNode>,
    pub(crate) next_fragment: FileChunkReference64x32,
    pub(crate) footer: u64,
}

impl FileNodeListFragment {
    pub(crate) fn parse(
        reader: crate::Reader,
        context: &mut ParseContext,
        size: usize,
    ) -> Result<Self> {
        let header = FileNodeListHeader::parse(reader)?;
        let mut file_nodes: Vec<FileNode> = Vec::new();
        let mut file_node_size: usize = 0;

        let remaining_0 = reader.remaining();

        // Sometimes, the node count is specified externally
        let mut maximum_node_count = context.get_file_node_count(&header).unwrap_or({
            log::warn!("No node count found.");
            usize::MAX
        });

        while size - 36 - file_node_size >= 4 && maximum_node_count > 0 {
            let file_node = FileNode::parse(reader, context)?;
            file_node_size += file_node.size;

            if !matches!(
                file_node.fnd,
                FileNodeData::ChunkTerminatorFND | FileNodeData::Null
            ) {
                maximum_node_count -= 1;
            }
            if !matches!(file_node.fnd, FileNodeData::Null) {
                file_nodes.push(file_node);
            }

            assert_eq!(remaining_0 - reader.remaining(), file_node_size);
        }

        context.update_remaining_nodes_in_fragment(&header, maximum_node_count);

        let padding_length = size - 36 - file_node_size;
        reader.advance(padding_length)?;

        let next_fragment = FileChunkReference64x32::parse(reader)?;

        let footer = reader.get_u64()?;
        if footer != 0x8BC215C38233BA4B {
            return Err(ErrorKind::MalformedOneStoreData(
                format!("Invalid footer: {:#0x}", footer).into(),
            )
            .into());
        }

        Ok(Self {
            header,
            file_nodes,
            next_fragment,
            footer,
        })
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct FileNodeListHeader {
    magic: u64,
    pub(crate) file_node_list_id: u32,
    pub(crate) n_fragment_sequence: u32,
}

impl Parse for FileNodeListHeader {
    fn parse(reader: crate::Reader) -> Result<Self> {
        let magic = u64::parse(reader)?;
        if magic != 0xA4567AB1F5F7F4C4 {
            return Err(ErrorKind::ParseValidationFailed(
                "Failed to validate: magic == 0xA4567AB1F5F7F4C4".into(),
            )
            .into());
        }

        let file_node_list_id = u32::parse(reader)?;
        if file_node_list_id < 0x0010 {
            log::warn!(
                "FileNodeListHeader: file_node_list_id {:#x} is below spec minimum 0x10",
                file_node_list_id
            );
        }

        let n_fragment_sequence = u32::parse(reader)?;

        Ok(Self {
            magic,
            file_node_list_id,
            n_fragment_sequence,
        })
    }
}
