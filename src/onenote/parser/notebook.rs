use crate::one::property_set::toc_container;
use crate::onenote::parser::section::Section;
use crate::onestore::revision::Revision;
use crate::onestore::OneStore;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub struct Notebook {
    pub(crate) sections: Vec<Section>,
}

pub(crate) fn parse_toc(rev: &Revision, store: &OneStore) -> Vec<String> {
    let content_id = rev.content_root().expect("notebook has no content root");

    parse_toc_entry(content_id, rev, store)
}

fn parse_toc_entry(content_id: ExGuid, rev: &Revision, store: &OneStore) -> Vec<String> {
    let content = rev
        .resolve_object(content_id, store)
        .expect("notebook content root is missing");

    let toc = toc_container::parse(content);

    if let Some(name) = toc.filename() {
        vec![name.to_string()]
    } else {
        toc.children()
            .iter()
            .flat_map(|id| parse_toc_entry(*id, rev, store))
            .collect()
    }
}