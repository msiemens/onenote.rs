use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property_set::toc_container;
use crate::onenote::section::SectionEntry;
use crate::onestore::object_space::ObjectSpace;
use crate::property::common::Color;
use itertools::Itertools;

/// A OneNote notebook.
#[derive(Clone, Debug)]
pub struct Notebook {
    pub(crate) entries: Vec<SectionEntry>,
    pub(crate) color: Option<Color>,
}

impl Notebook {
    /// The section entries of this notebook.
    pub fn entries(&self) -> &[SectionEntry] {
        &self.entries
    }

    /// The color of this notebook.
    pub fn color(&self) -> Option<Color> {
        self.color
    }
}

struct TocEntry {
    entries: Vec<(u32, String)>,
    color: Option<Color>,
}

pub(crate) fn parse_toc(space: &ObjectSpace) -> Result<(Vec<String>, Option<Color>)> {
    let content_id = space
        .content_root()
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("notebook has no content root".into()))?;

    let entry = parse_toc_entry(content_id, space)?;
    let toc = entry
        .entries
        .into_iter()
        .sorted_by_key(|(ordering_id, _)| *ordering_id)
        .dedup_by(|(_, a), (_, b)| a == b)
        .map(|(_, name)| name)
        .collect();

    Ok((toc, entry.color))
}

fn parse_toc_entry(content_id: ExGuid, space: &ObjectSpace) -> Result<TocEntry> {
    let content = space.get_object(content_id).ok_or_else(|| {
        ErrorKind::MalformedOneNoteData("notebook content root is missing".into())
    })?;

    let toc = toc_container::parse(content)?;

    if let Some(name) = toc.filename {
        let ordering_id = toc
            .ordering_id
            .ok_or_else(|| ErrorKind::MalformedOneNoteData("section has no order id".into()))?;

        Ok(TocEntry {
            entries: vec![(ordering_id, name)],
            color: toc.color,
        })
    } else {
        let children = toc
            .children
            .into_iter()
            .map(|content_id| parse_toc_entry(content_id, space))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flat_map(|entry| entry.entries)
            .collect();

        Ok(TocEntry {
            entries: children,
            color: toc.color,
        })
    }
}
