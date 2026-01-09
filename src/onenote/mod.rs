use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::packaging::OneStorePackaging;
use crate::onenote::notebook::Notebook;
use crate::onenote::section::{Section, SectionEntry, SectionGroup};
use crate::onestore::parse_store;
use crate::reader::Reader;
use sanitise_file_name::sanitise;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Component, Path, PathBuf};

pub(crate) mod content;
pub(crate) mod embedded_file;
pub(crate) mod iframe;
pub(crate) mod image;
pub(crate) mod ink;
pub(crate) mod list;
pub(crate) mod math_inline_object;
pub(crate) mod note_tag;
pub(crate) mod notebook;
pub(crate) mod outline;
pub(crate) mod page;
pub(crate) mod page_content;
pub(crate) mod page_series;
pub(crate) mod rich_text;
pub(crate) mod section;
pub(crate) mod table;

/// The OneNote file parser.
///
/// Use [`Parser::parse_notebook`] to load a notebook from a `.onetoc2` file or
/// [`Parser::parse_section`] to load a single `.one` section. These methods
/// expect OneDrive downloads (FSSHTTP packaging) and will return an error if the
/// input is not the expected file type.
///
/// # Thread safety
///
/// The parser is stateless and can be shared across threads.
pub struct Parser;

impl Parser {
    /// Create a new OneNote file parser.
    ///
    /// The parser holds no state; reuse a single instance across multiple
    /// parses if desired.
    pub fn new() -> Parser {
        Parser {}
    }

    /// Parse a OneNote notebook.
    ///
    /// The `path` argument must point to a `.onetoc2` file. This will parse the
    /// table of contents of the notebook as well as all contained
    /// sections from the folder that the table of contents file is in.
    ///
    /// Returns [`ErrorKind::NotATocFile`] if the file is not a notebook table of
    /// contents.
    pub fn parse_notebook(&self, path: &Path) -> Result<Notebook> {
        let file = File::open(path)?;
        let data = Parser::read(file)?;
        let packaging = OneStorePackaging::parse(&mut Reader::new(data.as_slice()))?;
        let store = parse_store(&packaging)?;

        if store.schema_guid() != guid!("E4DBFD38-E5C7-408B-A8A1-0E7B421E1F5F") {
            return Err(ErrorKind::NotATocFile {
                file: path.to_string_lossy().to_string(),
            }
            .into());
        }

        let base_dir = path.parent().ok_or_else(|| ErrorKind::InvalidPath {
            message: "path has no parent directory".into(),
        })?;
        let (entries, color) = notebook::parse_toc(store.data_root())?;
        let entries = entries
            .iter()
            .map(|name| resolve_entry_path(base_dir, name))
            .collect::<Result<Vec<_>>>()?;
        let sections = entries
            .into_iter()
            .filter(|p| p.exists())
            .filter(|p| !p.ends_with("OneNote_RecycleBin"))
            .map(|path| {
                if path.is_file() {
                    self.parse_section(&path).map(SectionEntry::Section)
                } else {
                    self.parse_section_group(&path)
                        .map(SectionEntry::SectionGroup)
                }
            })
            .collect::<Result<_>>()?;

        Ok(Notebook {
            entries: sections,
            color,
        })
    }

    /// Parse a OneNote section buffer.
    ///
    /// The `data` argument must contain a OneNote section.
    /// The `file_name` is used to populate section metadata and error messages.
    ///
    /// Returns [`ErrorKind::NotASectionFile`] if the buffer does not contain a
    /// section file.
    pub fn parse_section_buffer(&self, data: &[u8], file_name: &Path) -> Result<Section> {
        let packaging = OneStorePackaging::parse(&mut Reader::new(data))?;
        let store = parse_store(&packaging)?;

        if store.schema_guid() != guid!("1F937CB4-B26F-445F-B9F8-17E20160E461") {
            return Err(ErrorKind::NotASectionFile {
                file: file_name.to_string_lossy().into_owned(),
            }
            .into());
        }

        section::parse_section(store, file_name.to_string_lossy().into_owned())
    }

    /// Parse a OneNote section file.
    ///
    /// The `path` argument must point to a `.one` file that contains a
    /// OneNote section.
    ///
    /// Returns [`ErrorKind::NotASectionFile`] if the file does not contain a
    /// section.
    pub fn parse_section(&self, path: &Path) -> Result<Section> {
        let file = File::open(path)?;
        let data = Parser::read(file)?;
        let packaging = OneStorePackaging::parse(&mut Reader::new(data.as_slice()))?;
        let store = parse_store(&packaging)?;

        if store.schema_guid() != guid!("1F937CB4-B26F-445F-B9F8-17E20160E461") {
            return Err(ErrorKind::NotASectionFile {
                file: path.to_string_lossy().to_string(),
            }
            .into());
        }

        section::parse_section(
            store,
            path.file_name()
                .ok_or_else(|| ErrorKind::InvalidPath {
                    message: "path has no file name".into(),
                })?
                .to_string_lossy()
                .to_string(),
        )
    }

    fn parse_section_group(&self, path: &Path) -> Result<SectionGroup> {
        let display_name = path
            .file_name()
            .ok_or_else(|| ErrorKind::InvalidPath {
                message: "path has no file name".into(),
            })?
            .to_string_lossy()
            .to_string();

        for entry in path.read_dir()? {
            let entry = entry?;
            let is_toc = entry
                .path()
                .extension()
                .map(|ext| ext == OsStr::new("onetoc2"))
                .unwrap_or_default();

            if is_toc {
                return self
                    .parse_notebook(&entry.path())
                    .map(|group| SectionGroup {
                        display_name,
                        entries: group.entries,
                    });
            }
        }

        Err(ErrorKind::TocFileMissing {
            dir: path.as_os_str().to_string_lossy().into_owned(),
        }
        .into())
    }

    fn read(file: File) -> Result<Vec<u8>> {
        let size = file.metadata()?.len();
        let mut data = Vec::with_capacity(size as usize);

        let mut buf = BufReader::new(file);
        buf.read_to_end(&mut data)?;

        Ok(data)
    }
}

fn resolve_entry_path(base_dir: &Path, entry: &str) -> Result<PathBuf> {
    let entry_path = Path::new(entry);
    if entry_path.is_absolute() {
        return Err(ErrorKind::InvalidPath {
            message: "section entry must be a relative path".into(),
        }
        .into());
    }

    let mut sanitized = PathBuf::new();
    for component in entry_path.components() {
        match component {
            Component::Normal(name) => {
                let name = name.to_str().ok_or_else(|| ErrorKind::InvalidPath {
                    message: "section entry contains non-utf8 characters".into(),
                })?;
                let clean = sanitise(name);
                if clean != name {
                    return Err(ErrorKind::InvalidPath {
                        message: format!("section entry contains invalid characters: {name}")
                            .into(),
                    }
                    .into());
                }
                sanitized.push(name);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ErrorKind::InvalidPath {
                    message: "section entry contains invalid path components".into(),
                }
                .into());
            }
        }
    }

    if sanitized.as_os_str().is_empty() {
        return Err(ErrorKind::InvalidPath {
            message: "section entry is empty".into(),
        }
        .into());
    }

    let candidate = base_dir.join(&sanitized);
    if candidate.exists() {
        let base_canon = base_dir
            .canonicalize()
            .map_err(|err| ErrorKind::InvalidPath {
                message: format!("failed to resolve base directory: {err}").into(),
            })?;
        let candidate_canon = candidate
            .canonicalize()
            .map_err(|err| ErrorKind::InvalidPath {
                message: format!("failed to resolve entry path: {err}").into(),
            })?;
        if !candidate_canon.starts_with(&base_canon) {
            return Err(ErrorKind::InvalidPath {
                message: "section entry escapes base directory".into(),
            }
            .into());
        }
    }

    Ok(candidate)
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_entry_path;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_entry_path_rejects_traversal() {
        let dir = tempdir().unwrap();
        let base = dir.path();

        let err = resolve_entry_path(base, "../secret.one").unwrap_err();
        let err = format!("{err}");
        assert!(err.contains("invalid path components"));
    }

    #[test]
    fn test_resolve_entry_path_rejects_absolute() {
        let dir = tempdir().unwrap();
        let base = dir.path();

        let candidate = if cfg!(windows) {
            r"C:\secret.one"
        } else {
            "/etc/passwd"
        };
        let err = resolve_entry_path(base, candidate).unwrap_err();
        let err = format!("{err}");
        assert!(err.contains("relative path"));
    }

    #[test]
    fn test_resolve_entry_path_accepts_relative() {
        let dir = tempdir().unwrap();
        let base = dir.path();

        let resolved = resolve_entry_path(base, "Section 1.one").unwrap();
        assert_eq!(resolved, Path::new(base).join("Section 1.one"));
    }
}
