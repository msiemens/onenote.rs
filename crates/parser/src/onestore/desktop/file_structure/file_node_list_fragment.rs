use super::parse_context::ParseContext;
use crate::errors::ErrorKind;
use crate::errors::Result;
use crate::onestore::desktop::common::FileChunkReference64x32;
use crate::onestore::desktop::file_node::FileNode;
use crate::onestore::desktop::file_node::FileNodeData;
use crate::onestore::desktop::parse::Parse;

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
            let is_chunk_terminator = matches!(file_node.fnd, FileNodeData::ChunkTerminatorFND);
            if !matches!(file_node.fnd, FileNodeData::Null) {
                file_nodes.push(file_node);
            }

            let consumed = remaining_0 - reader.remaining();
            if consumed != file_node_size {
                return Err(ErrorKind::MalformedOneNoteFileData(
                    format!(
                        "FileNodeListFragment size accounting mismatch: reader consumed {consumed} bytes, declared sizes summed to {file_node_size}"
                    )
                    .into(),
                )
                .into());
            }
            if is_chunk_terminator {
                break;
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::Reader;

    const HEADER_MAGIC: u64 = 0xA4567AB1F5F7F4C4;
    const FOOTER_MAGIC: u64 = 0x8BC215C38233BA4B;

    /// A fragment holding one terminator, then padding where old bytes survive:
    /// each stale word is a node header claiming an unknown type and no size.
    fn fragment_with_stale_padding() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&HEADER_MAGIC.to_le_bytes());
        bytes.extend_from_slice(&0x10u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        // ChunkTerminatorFND: id 0x0FF, size 4, the required reserved bit.
        let terminator: u32 = 0x0ff | (4 << 10) | (1 << 31);
        bytes.extend_from_slice(&terminator.to_le_bytes());
        for _ in 0..4 {
            // Node type 0x95, declared size 0: reading it is the bug.
            bytes.extend_from_slice(&(0x95u32 | (1 << 31)).to_le_bytes());
        }
        // The next fragment: nil.
        bytes.extend_from_slice(&u64::MAX.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&FOOTER_MAGIC.to_le_bytes());
        bytes
    }

    #[test]
    fn stale_padding_after_a_chunk_terminator_is_never_read_as_nodes() {
        let bytes = fragment_with_stale_padding();
        let mut reader = Reader::new(&bytes);
        let mut context = ParseContext::new();
        let fragment = FileNodeListFragment::parse(&mut reader, &mut context, bytes.len())
            .expect("a fragment ends at its terminator");
        assert_eq!(fragment.file_nodes.len(), 1);
        assert!(matches!(
            fragment.file_nodes[0].fnd,
            FileNodeData::ChunkTerminatorFND
        ));
    }
}
