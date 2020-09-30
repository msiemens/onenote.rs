use crate::one::property_set::toc_container;
use crate::onenote::parser::section::Section;
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;
use itertools::Itertools;

#[derive(Debug)]
pub struct Notebook {
    pub(crate) sections: Vec<Section>,
}

impl Notebook {
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }
}

pub(crate) fn parse_toc(space: &ObjectSpace) -> Vec<String> {
    let content_id = space.content_root().expect("notebook has no content root");

    parse_toc_entry(content_id, space)
        .into_iter()
        .sorted_by_key(|(ordering_id, _)| *ordering_id)
        .dedup_by(|(_, a), (_, b)| a == b)
        .map(|(_, name)| name)
        .collect()
}

fn parse_toc_entry(content_id: ExGuid, space: &ObjectSpace) -> Vec<(u32, String)> {
    let content = space
        .get_object(content_id)
        .expect("notebook content root is missing");

    let toc = toc_container::parse(content);

    if let Some(name) = toc.filename {
        let ordering_id = toc.ordering_id.expect("section has no order id");

        vec![(ordering_id, name)]
    } else {
        toc.children
            .into_iter()
            .flat_map(|content_id| parse_toc_entry(content_id, space))
            .collect()
    }
}
