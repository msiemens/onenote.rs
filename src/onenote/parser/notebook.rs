use crate::one::property_set::toc_container;
use crate::onenote::parser::section::Section;
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub struct Notebook {
    pub(crate) sections: Vec<Section>,
}

pub(crate) fn parse_toc(space: &ObjectSpace) -> Vec<String> {
    let content_id = space.content_root().expect("notebook has no content root");

    parse_toc_entry(content_id, space)
}

fn parse_toc_entry(content_id: ExGuid, space: &ObjectSpace) -> Vec<String> {
    let content = space
        .get_object(content_id)
        .expect("notebook content root is missing");

    let toc = toc_container::parse(content);

    if let Some(name) = toc.filename {
        vec![name.to_string()]
    } else {
        toc.children
            .into_iter()
            .flat_map(|content_id| parse_toc_entry(content_id, space))
            .collect()
    }
}
