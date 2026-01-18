use crate::FileSystem;
use crate::errors::{ErrorKind, Result};
#[cfg(feature = "native-fs")]
use crate::fs::NativeFs;
use crate::fsshttpb::packaging::{OneStorePackaging, embedded_packaging_offset};
use crate::onenote::notebook::Notebook;
use crate::onenote::section::{Section, SectionEntry, SectionGroup};
use crate::onestore::fsshttpb::parse_store;
use crate::onestore::legacy::one_store_file::OneStoreFile;
use crate::onestore::legacy::parse::Parse;
use crate::onestore::{ObjectSpace, OneStore, OneStoreType};
use crate::reader::Reader;
use crate::shared::guid::Guid;
use sanitise_file_name::sanitise;
use std::ffi::OsStr;
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
/// auto-detect OneNote 2016 (legacy) and OneDrive (FSSHTTP) formats and will
/// return an error if the input is not the expected file type.
///
/// # Thread safety
///
/// The parser is stateless and can be shared across threads.
#[cfg(feature = "native-fs")]
pub struct Parser<FS: FileSystem = NativeFs> {
    fs: FS,
}

/// The OneNote file parser.
///
/// Use [`Parser::parse_notebook`] to load a notebook from a `.onetoc2` file or
/// [`Parser::parse_section`] to load a single `.one` section. These methods
/// auto-detect OneNote 2016 (legacy) and OneDrive (FSSHTTP) formats and will
/// return an error if the input is not the expected file type.
///
/// # Thread safety
///
/// The parser is stateless and can be shared across threads.
#[cfg(not(feature = "native-fs"))]
pub struct Parser<FS: FileSystem> {
    fs: FS,
}

#[cfg(feature = "native-fs")]
impl Parser<NativeFs> {
    /// Create a new OneNote file parser.
    ///
    /// The parser holds no state; reuse a single instance across multiple
    /// parses if desired.
    pub fn new() -> Parser<NativeFs> {
        Parser { fs: NativeFs {} }
    }
}

impl<FS: FileSystem> Parser<FS> {
    /// Create a new instance of the `Parser` struct using the provided file system.
    ///
    /// # Parameters
    /// - `fs`: An instance of an object implementing the `FileSystem` trait.
    ///   This parameter provides the necessary file system operations for the `Parser`.
    pub fn new_with_fs(fs: FS) -> Parser<FS> {
        Parser { fs }
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
        let data = self.fs.read_file(path)?;
        let store = parse_store_auto(&data)?;

        if store.get_type() != OneStoreType::TableOfContents {
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
            .filter(|p| self.fs.exists(p).unwrap_or(false))
            .filter(|p| !p.ends_with("OneNote_RecycleBin"))
            .map(|path| {
                if self.fs.is_directory(&path)? {
                    self.parse_section_group(&path)
                        .map(SectionEntry::SectionGroup)
                } else {
                    self.parse_section(&path).map(SectionEntry::Section)
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
    pub fn parse_section_buffer(self, data: &[u8], file_name: &Path) -> Result<Section> {
        let store = parse_store_auto(data)?;

        if store.get_type() != OneStoreType::Section {
            return Err(ErrorKind::NotASectionFile {
                file: file_name.to_string_lossy().into_owned(),
            }
            .into());
        }

        section::parse_section(
            store.as_onestore(),
            file_name.to_string_lossy().into_owned(),
        )
    }

    /// Parse a OneNote section file.
    ///
    /// The `path` argument must point to a `.one` file that contains a
    /// OneNote section.
    ///
    /// Returns [`ErrorKind::NotASectionFile`] if the file does not contain a
    /// section.
    pub fn parse_section(&self, path: &Path) -> Result<Section> {
        let data = self.fs.read_file(path)?;
        let store = parse_store_auto(&data)?;

        if store.get_type() != OneStoreType::Section {
            return Err(ErrorKind::NotASectionFile {
                file: path.to_string_lossy().to_string(),
            }
            .into());
        }

        section::parse_section(
            store.as_onestore(),
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

        for entry in self.fs.read_dir(path)? {
            let is_toc = entry
                .extension()
                .map(|ext| ext == OsStr::new("onetoc2"))
                .unwrap_or_default();

            if is_toc {
                return self.parse_notebook(&entry).map(|group| SectionGroup {
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
}

enum ParsedStore {
    Legacy(OneStoreFile),
    FssHttpB(crate::onestore::fsshttpb::OneStore),
}

impl ParsedStore {
    fn get_type(&self) -> OneStoreType {
        match self {
            ParsedStore::Legacy(store) => store.get_type(),
            ParsedStore::FssHttpB(store) => store.get_type(),
        }
    }

    fn data_root(&self) -> &dyn ObjectSpace {
        match self {
            ParsedStore::Legacy(store) => store.data_root(),
            ParsedStore::FssHttpB(store) => store.data_root(),
        }
    }

    fn as_onestore(&self) -> &dyn OneStore {
        match self {
            ParsedStore::Legacy(store) => store,
            ParsedStore::FssHttpB(store) => store,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoreFormat {
    Legacy,
    FssHttpB { packaging_offset: usize },
}

fn parse_store_auto(data: &[u8]) -> Result<ParsedStore> {
    match sniff_store_format(data) {
        Some(StoreFormat::FssHttpB { packaging_offset }) => {
            let mut reader = Reader::new(data);
            reader.advance(packaging_offset)?;

            let packaging = OneStorePackaging::parse(&mut reader)?;
            let store = parse_store(&packaging)?;

            Ok(ParsedStore::FssHttpB(store))
        }
        Some(StoreFormat::Legacy) => {
            let mut reader = Reader::new(data);
            let store = OneStoreFile::parse(&mut reader)?;
            Ok(ParsedStore::Legacy(store))
        }
        None => {
            let fss_err = match OneStorePackaging::parse(&mut Reader::new(data))
                .and_then(|packaging| parse_store(&packaging))
            {
                Ok(store) => return Ok(ParsedStore::FssHttpB(store)),
                Err(err) => err,
            };

            let mut reader = Reader::new(data);
            match OneStoreFile::parse(&mut reader) {
                Ok(store) => Ok(ParsedStore::Legacy(store)),
                Err(_) => Err(fss_err),
            }
        }
    }
}

fn sniff_store_format(data: &[u8]) -> Option<StoreFormat> {
    // TODO: Read header directly?

    let mut reader = Reader::new(data);
    let file_type = Guid::parse(&mut reader).ok()?;
    let file = Guid::parse(&mut reader).ok()?;
    let legacy_file_version = Guid::parse(&mut reader).ok()?;
    let file_format = Guid::parse(&mut reader).ok()?;

    let revision_store_format = guid!("109ADD3F-911B-49F5-A5D0-1791EDC8AED8");
    let package_store_format = guid!("638DE92F-A6D4-4BC1-9A36-B3FC2511A5B7");

    log::debug!(
        "sniff_store_format header: file_type={:?} file={:?} legacy_file_version={:?} file_format={:?}",
        file_type,
        file,
        legacy_file_version,
        file_format
    );

    if file_format == package_store_format {
        return Some(StoreFormat::FssHttpB {
            packaging_offset: 0,
        });
    }

    if file_format == revision_store_format {
        if legacy_file_version.is_nil() {
            if let Some(packaging_offset) = embedded_packaging_offset(data) {
                return Some(StoreFormat::FssHttpB { packaging_offset });
            }

            return Some(StoreFormat::Legacy);
        }

        return Some(StoreFormat::FssHttpB {
            packaging_offset: 0,
        });
    }

    None
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

#[cfg(feature = "native-fs")]
impl Default for Parser<NativeFs> {
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
