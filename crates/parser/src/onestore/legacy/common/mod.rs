pub mod exguid;
mod file_chunk_reference;
mod object_declaration_with_ref_count_body;
mod object_space_object_stream_header;

pub(crate) use file_chunk_reference::{
    FileChunkReference, FileChunkReference32, FileChunkReference64, FileChunkReference64x32,
};

pub(crate) use object_declaration_with_ref_count_body::ObjectDeclarationWithRefCountBody;
