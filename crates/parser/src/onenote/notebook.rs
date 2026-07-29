use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property_set::toc_container;
use crate::onenote::section::SectionEntry;
use crate::onestore::ObjectSpace;
use crate::property::common::Color;
use crate::warn::Report;
use std::collections::HashSet;

/// A OneNote notebook.
#[derive(Clone, Debug)]
pub struct Notebook {
    pub(crate) entries: Vec<SectionEntry>,
    pub(crate) color: Option<Color>,
    pub(crate) report: Report,
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

    /// The notebook-level report of non-fatal issues.
    ///
    /// Currently captures one warning per section that failed to parse; the
    /// failing section is omitted from [`Notebook::entries`] rather than
    /// aborting the entire notebook.
    pub fn report(&self) -> &Report {
        &self.report
    }
}

struct TocEntry {
    entries: Vec<(u32, String)>,
    color: Option<Color>,
}

pub(crate) fn parse_toc(
    space: &(impl ObjectSpace + ?Sized),
) -> Result<(Vec<String>, Option<Color>)> {
    let content_id = space
        .content_root()
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("notebook has no content root".into()))?;

    let entry = parse_toc_entry(content_id, space)?;
    let toc = ordered_toc_entries(entry.entries);

    Ok((toc, entry.color))
}

fn ordered_toc_entries(entries: Vec<(u32, String)>) -> Vec<String> {
    // OneNote can retain older ordering snapshots in the same TOC. Later
    // references to a filename are authoritative.
    let mut seen = HashSet::new();
    let mut latest = entries
        .into_iter()
        .enumerate()
        .rev()
        .filter(|(_, (_, name))| seen.insert(name.clone()))
        .collect::<Vec<_>>();
    latest.sort_by_key(|(source_index, (ordering_id, _))| (*ordering_id, *source_index));
    latest.into_iter().map(|(_, (_, name))| name).collect()
}

fn parse_toc_entry(content_id: ExGuid, space: &(impl ObjectSpace + ?Sized)) -> Result<TocEntry> {
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

#[cfg(test)]
mod tests {
    use super::ordered_toc_entries;

    #[test]
    fn later_toc_snapshot_entries_override_earlier_ordering() {
        let entries = vec![
            (1, "Notes.one".to_owned()),
            (5, "Training".to_owned()),
            (10, "Datasets.one".to_owned()),
            (1, "Notes.one".to_owned()),
            (2, "Datasets.one".to_owned()),
            (3, "Training".to_owned()),
        ];

        assert_eq!(
            ordered_toc_entries(entries),
            ["Notes.one", "Datasets.one", "Training"]
        );
    }
}
