#![deny(clippy::disallowed_macros)]

use crate::FileSystem;
use crate::errors::{ErrorKind, Result};
use crate::fs::FileSource;
use crate::fs::file_source::BytesSource;
#[cfg(feature = "native-fs")]
use crate::fs::native_fs::NativeFs;
use crate::fsshttpb::data::exguid::ExGuid;
use crate::fsshttpb::packaging::{OneStorePackaging, embedded_packaging_offset};
use crate::onenote::ink_recognition::InkRecognizedWord;
use crate::onenote::notebook::Notebook;
use crate::onenote::section::{Section, SectionEntry, SectionGroup};
use crate::onestore::desktop::one_store_file::RevisionStore;
use crate::onestore::desktop::parse::Parse;
use crate::onestore::fsshttpb::parse_store;
use crate::onestore::{ObjectSpace, OneStore, OneStoreType};
use crate::reader::Reader;
use crate::shared::guid::Guid;
use crate::warn::Report;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use typed_path::{
    PathType, TypedComponent, TypedPath, TypedPathBuf, UnixComponent, WindowsComponent,
};
use uuid::Uuid;

pub(crate) mod content;
pub(crate) mod embedded_file;
pub(crate) mod iframe;
pub(crate) mod image;
pub(crate) mod ink;
pub(crate) mod ink_recognition;
pub(crate) mod list;
pub(crate) mod math_inline_object;
pub(crate) mod note_tag;
pub(crate) mod notebook;
pub(crate) mod outline;
pub(crate) mod page;
pub(crate) mod page_content;
pub(crate) mod page_series;
pub(crate) mod picture;
pub(crate) mod rich_text;
pub(crate) mod section;
pub(crate) mod table;

pub(crate) struct ParserContext {
    pub(crate) page: Option<(Uuid, String)>,
    pub(crate) report: Report,

    /// Maps an ink stroke's `ExGuid` to the handwriting-recognition word it
    /// belongs to, for the page currently being parsed.
    ///
    /// Populated from the recognition tree before the page's content is walked
    /// and consumed when ink strokes are parsed, filling
    /// [`InkStroke::recognized_word`](ink::InkStroke::recognized_word).
    pub(crate) recognized_words: HashMap<ExGuid, InkRecognizedWord>,
}

/// The OneNote file parser.
///
/// Use [`Parser::parse_notebook`] to load a notebook from a `.onetoc2` file or
/// [`Parser::parse_section`] to load a single `.one` section. These methods
/// auto-detect OneNote 2016 (desktop) and OneDrive (FSSHTTP) formats and will
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
/// auto-detect OneNote 2016 (desktop) and OneDrive (FSSHTTP) formats and will
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
    /// sections from the folder that the table of contents file is in. Each
    /// returned [`Section`] carries its own [`Report`] of non-fatal warnings,
    /// reachable via [`Section::report`].
    ///
    /// Returns [`ErrorKind::NotATocFile`] if the file is not a notebook table of
    /// contents.
    ///
    /// The notebook files are read on demand; mutating any of them while
    /// a parse is in progress, or while a derived
    /// [`crate::contents::Image`] / [`crate::contents::EmbeddedFile`]
    /// is alive, is unsupported.
    pub fn parse_notebook(&self, path: TypedPath) -> Result<Notebook> {
        let source = self.fs.open_file(path)?;
        let store = parse_store_auto(source)?;

        if store.get_type()? != OneStoreType::TableOfContents {
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
            .map(|name| self.resolve_entry_path(base_dir, name))
            .collect::<Result<Vec<_>>>()?;

        let mut report = Report::new();
        let mut sections = Vec::new();
        for entry_path in entries
            .into_iter()
            .filter(|p| self.fs.exists(p.to_path()).unwrap_or(false))
            .filter(|p| !p.ends_with(b"OneNote_RecycleBin"))
        {
            let entry_ref = entry_path.to_path();
            let result = if self.fs.is_directory(entry_ref).unwrap_or(false) {
                self.parse_section_group(entry_ref)
                    .map(SectionEntry::SectionGroup)
            } else {
                self.parse_section(entry_ref).map(SectionEntry::Section)
            };

            match result {
                Ok(entry) => sections.push(entry),
                Err(err) => {
                    // A single bad section shouldn't take down the whole notebook;
                    // skip it, surface the error on the notebook report.
                    report.push_warning(crate::warn::Warning::new(
                        None,
                        format!(
                            "failed to import section {}: {err}",
                            entry_path.to_string_lossy()
                        ),
                    ));
                }
            }
        }

        Ok(Notebook {
            entries: sections,
            color,
            report,
        })
    }

    /// Parse a OneNote section buffer.
    ///
    /// The `data` argument must contain a OneNote section.
    /// The `file_name` is used to populate section metadata and error messages.
    ///
    /// Returns [`ErrorKind::NotASectionFile`] if the buffer does not contain a
    /// section file.
    pub fn parse_section_buffer(self, data: &[u8], file_name: TypedPath) -> Result<Section> {
        let source: Arc<dyn FileSource> = Arc::new(BytesSource::new(Bytes::copy_from_slice(data)));
        let store = parse_store_auto(source)?;
        let file_name = file_name.to_string_lossy().into_owned();

        if store.get_type()? != OneStoreType::Section {
            return Err(ErrorKind::NotASectionFile { file: file_name }.into());
        }

        section::parse_section(store.as_onestore(), file_name)
    }

    /// Parse the raw OneStore layer from a buffer and return its `Debug`
    /// representation.
    ///
    /// Intended for tooling (e.g. the `inspect` binary) that needs to dump
    /// the low-level OneStore structures of a `.one` or `.onetoc2` file
    /// without resolving them into the high-level [`Section`] / [`Notebook`]
    /// types. The exact format of the returned string is unstable and should
    /// not be parsed by scripts.
    pub fn dump_onestore(&self, data: &[u8]) -> Result<String> {
        let source: Arc<dyn FileSource> = Arc::new(BytesSource::new(Bytes::copy_from_slice(data)));
        let store = parse_store_auto(source)?;
        Ok(format!("{:#?}", store))
    }

    /// Parse a OneNote section file.
    ///
    /// The `path` argument must point to a `.one` file that contains a
    /// OneNote section. The returned [`Section`] carries any non-fatal warnings
    /// encountered while parsing, reachable via [`Section::report`].
    ///
    /// Returns [`ErrorKind::NotASectionFile`] if the file does not contain a
    /// section.
    ///
    /// The file is read on demand; mutating it while a parse is in
    /// progress, or while a derived attachment is alive, is unsupported.
    pub fn parse_section(&self, path: TypedPath) -> Result<Section> {
        let source = self.fs.open_file(path)?;
        let store = parse_store_auto(source)?;

        if store.get_type()? != OneStoreType::Section {
            return Err(ErrorKind::NotASectionFile {
                file: path.to_string_lossy().to_string(),
            }
            .into());
        }

        section::parse_section(store.as_onestore(), file_name_string(path)?)
    }

    /// Parse a OneNote package (`.onepkg`) file.
    ///
    /// `.onepkg` files are CAB archives bundling a `.onetoc2` plus its `.one`
    /// section files. This method decompresses the archive in memory and parses
    /// the contained notebook without writing anything to disk.
    ///
    /// Returns [`ErrorKind::MalformedPackage`] if the file is not a valid
    /// cabinet or does not contain a `.onetoc2` table of contents.
    #[cfg(feature = "onepkg")]
    pub fn parse_package(&self, path: TypedPath) -> Result<Notebook> {
        let data = self.fs.read_file(path)?;
        let store = crate::onepkg::PackageStore::from_bytes(&data)?;
        let inner_fs = crate::onepkg::PackageFs::new(&store);
        let toc_path = store.toc_path().to_path_buf();
        Parser::new_with_fs(inner_fs).parse_notebook(toc_path.to_path())
    }

    fn parse_section_group(&self, path: TypedPath) -> Result<SectionGroup> {
        let display_name = file_name_string(path)?;

        for entry in self.fs.read_dir(path)? {
            let is_toc = entry
                .extension()
                .map(|ext| ext == b"onetoc2")
                .unwrap_or_default();

            if is_toc {
                return self
                    .parse_notebook(entry.to_path())
                    .map(|group| SectionGroup {
                        display_name,
                        entries: group.entries,
                    });
            }
        }

        Err(ErrorKind::TocFileMissing {
            dir: path.to_string_lossy().into_owned(),
        }
        .into())
    }

    fn resolve_entry_path(&self, base_dir: TypedPath, entry: &str) -> Result<TypedPathBuf> {
        // Parse the entry as a Windows path on every host.
        //
        // The string comes from the `.onetoc2` `FolderChildFilename` property,
        // which is **attacker-controlled**: a malicious notebook can put
        // anything here, including separators, `..`, drive letters or
        // reserved device names. In practice OneNote only ever writes a
        // bare leaf name (e.g. "Section 1.one"), so the encoding doesn't
        // affect any real fixture.
        //
        // We pick `PathType::Windows` (not `Unix`, not host-derived, not
        // `TypedPath::derive`) for two reasons:
        //
        // 1. Determinism. Host-derived encoding made the security check
        //    behave differently on Unix vs Windows CI — `"foo\\bar"` was a
        //    single literal component on Unix and a two-component path on
        //    Windows. With a fixed encoding the audit holds on every host.
        // 2. Strictness. Windows encoding treats **both** `/` and `\` as
        //    separators, so an attacker can't bypass the
        //    `Normal`/`CurDir` component whitelist by picking the
        //    separator the host doesn't recognise. (`Unix` would let `\`
        //    sneak through as a literal byte in a filename;
        //    `TypedPath::derive` falls back to `Unix` unless the string
        //    starts with `\` — also bypassable.)
        let entry_path = TypedPath::new(entry, PathType::Windows);
        if entry_path.is_absolute() {
            return Err(ErrorKind::InvalidPath {
                message: "section entry must be a relative path".into(),
            }
            .into());
        }

        let mut sanitized = TypedPathBuf::new(PathType::Windows);
        for component in entry_path.components() {
            match component {
                TypedComponent::Windows(WindowsComponent::Normal(name))
                | TypedComponent::Unix(UnixComponent::Normal(name)) => {
                    let name = std::str::from_utf8(name).map_err(|_| ErrorKind::InvalidPath {
                        message: "section entry contains non-UTF-8 bytes".into(),
                    })?;
                    // windows: false on every host so behaviour is deterministic
                    // across native/WASM/CI; we only strip path-traversal characters
                    // and control codes here. Defence against Windows reserved
                    // device names (CON, COM1, NUL, ...) is the FileSystem impl's
                    // responsibility — see the security contract on the trait.
                    // OneNote (Mac, and Windows via \\?\) legitimately writes
                    // section files with such names and we want to be able to
                    // parse them.
                    let name = sanitize_filename::sanitize_with_options(
                        name,
                        sanitize_filename::Options {
                            windows: false,
                            ..Default::default()
                        },
                    );

                    sanitized.push(name);
                }

                TypedComponent::Windows(WindowsComponent::CurDir)
                | TypedComponent::Unix(UnixComponent::CurDir) => {}

                _ => {
                    return Err(ErrorKind::InvalidPath {
                        message: "section entry contains invalid path components".into(),
                    }
                    .into());
                }
            }
        }

        if sanitized.as_bytes().is_empty() {
            return Err(ErrorKind::InvalidPath {
                message: "section entry is empty".into(),
            }
            .into());
        }

        // `TypedPath::join` and `starts_with` take `impl AsRef<[u8]>` and
        // reinterpret the argument under *self*'s encoding — the argument's
        // own encoding tag is discarded. We therefore have to know, at every
        // byte-level call below, why the two sides agree.
        //
        // `sanitized` is `PathType::Windows` but each component went through
        // `sanitize_filename` (which strips `/`, `\`, and other separators)
        // and we only `push`ed `Normal`/`CurDir` components. So its byte form
        // is a separator-free leaf-or-leaves sequence that parses identically
        // under any encoding — reinterpreting it under `base_dir`'s encoding
        // is a no-op.
        let candidate = base_dir.join(sanitized.as_bytes());
        if self.fs.exists(candidate.to_path()).unwrap_or(false) {
            let base_canon =
                self.fs
                    .canonicalize(base_dir)
                    .map_err(|err| ErrorKind::InvalidPath {
                        message: format!("failed to resolve base directory: {err}").into(),
                    })?;
            let candidate_canon = self.fs.canonicalize(candidate.to_path()).map_err(|err| {
                ErrorKind::InvalidPath {
                    message: format!("failed to resolve entry path: {err}").into(),
                }
            })?;

            // Both paths come from the same `FileSystem::canonicalize` impl
            // on the same call, so they share an encoding by construction
            // (host-native for `NativeFs`, Unix for `PackageFs`). That makes
            // the byte-level `starts_with` a valid component-prefix check —
            // the encoding tag dropped by `AsRef<[u8]>` would have matched
            // self's anyway.
            if !candidate_canon.starts_with(base_canon.as_bytes()) {
                return Err(ErrorKind::InvalidPath {
                    message: "section entry escapes base directory".into(),
                }
                .into());
            }
        }

        Ok(candidate)
    }
}

#[derive(Debug)]
enum ParsedStore {
    Desktop(RevisionStore),
    FssHttpB(crate::onestore::fsshttpb::PackagingStore),
}

impl ParsedStore {
    fn get_type(&self) -> Result<OneStoreType> {
        match self {
            ParsedStore::Desktop(store) => store.get_type(),
            ParsedStore::FssHttpB(store) => store.get_type(),
        }
    }

    fn data_root(&self) -> &dyn ObjectSpace {
        match self {
            ParsedStore::Desktop(store) => store.data_root(),
            ParsedStore::FssHttpB(store) => store.data_root(),
        }
    }

    fn as_onestore(&self) -> &dyn OneStore {
        match self {
            ParsedStore::Desktop(store) => store,
            ParsedStore::FssHttpB(store) => store,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoreFormat {
    Desktop,
    FssHttpB { packaging_offset: usize },
}

fn parse_store_auto(source: Arc<dyn FileSource>) -> Result<ParsedStore> {
    let new_reader = || Reader::from_source(source.clone());
    match sniff_store_format(&source) {
        Some(StoreFormat::FssHttpB { packaging_offset }) => {
            let mut reader = new_reader();
            reader.advance(packaging_offset)?;

            let packaging = OneStorePackaging::parse(&mut reader)?;
            let store = parse_store(&packaging)?;

            Ok(ParsedStore::FssHttpB(store))
        }
        Some(StoreFormat::Desktop) => {
            let mut reader = new_reader();
            let store = RevisionStore::parse(&mut reader)?;
            Ok(ParsedStore::Desktop(store))
        }
        None => {
            let fss_err = match OneStorePackaging::parse(&mut new_reader())
                .and_then(|packaging| parse_store(&packaging))
            {
                Ok(store) => return Ok(ParsedStore::FssHttpB(store)),
                Err(err) => err,
            };

            let mut reader = new_reader();
            match RevisionStore::parse(&mut reader) {
                Ok(store) => Ok(ParsedStore::Desktop(store)),
                Err(_) => Err(fss_err),
            }
        }
    }
}

fn sniff_store_format(source: &Arc<dyn FileSource>) -> Option<StoreFormat> {
    let mut reader = Reader::from_source(source.clone());
    let _file_type = Guid::parse(&mut reader).ok()?;
    let _file = Guid::parse(&mut reader).ok()?;
    let legacy_file_version = Guid::parse(&mut reader).ok()?;
    let file_format = Guid::parse(&mut reader).ok()?;

    let revision_store_format = guid!("109ADD3F-911B-49F5-A5D0-1791EDC8AED8");
    let package_store_format = guid!("638DE92F-A6D4-4BC1-9A36-B3FC2511A5B7");

    if file_format == package_store_format {
        return Some(StoreFormat::FssHttpB {
            packaging_offset: 0,
        });
    }

    if file_format == revision_store_format {
        if legacy_file_version.is_nil() {
            if let Some(packaging_offset) = embedded_packaging_offset(source) {
                return Some(StoreFormat::FssHttpB { packaging_offset });
            }

            return Some(StoreFormat::Desktop);
        }

        return Some(StoreFormat::FssHttpB {
            packaging_offset: 0,
        });
    }

    None
}

fn file_name_string(path: TypedPath) -> Result<String> {
    let name = path.file_name().ok_or_else(|| ErrorKind::InvalidPath {
        message: "path has no file name".into(),
    })?;
    Ok(String::from_utf8_lossy(name).into_owned())
}

#[cfg(feature = "native-fs")]
impl Default for Parser<NativeFs> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::fs::native_fs::NativeFs;
    use tempfile::tempdir;
    use typed_path::TypedPath;

    fn base_path(dir: &std::path::Path) -> typed_path::TypedPathBuf {
        TypedPath::derive(dir.to_str().expect("tempdir path is UTF-8")).to_path_buf()
    }

    #[test]
    fn test_resolve_entry_path_rejects_traversal() {
        let dir = tempdir().unwrap();
        let base = base_path(dir.path());
        let parser = Parser::new_with_fs(NativeFs {});

        let err = parser
            .resolve_entry_path(base.to_path(), "../secret.one")
            .unwrap_err();
        let err = format!("{err}");
        assert!(err.contains("invalid path components"));
    }

    #[test]
    fn test_resolve_entry_path_rejects_absolute() {
        let dir = tempdir().unwrap();
        let base = base_path(dir.path());
        let parser = Parser::new_with_fs(NativeFs {});

        // Entries are parsed with `PathType::Windows` on every host (see
        // the security comment on `resolve_entry_path`). A leading drive
        // letter is rejected as absolute; a leading `/` or `\` without
        // drive is "rooted" but technically relative on Windows, so it
        // falls through to component validation and is rejected as a
        // `RootDir` component. Both rejections are correct — assert
        // either error message.
        for candidate in [r"C:\secret.one", "/etc/passwd", r"\windows\secret"] {
            let err = parser
                .resolve_entry_path(base.to_path(), candidate)
                .unwrap_err();
            let err = format!("{err}");
            assert!(
                err.contains("relative path") || err.contains("invalid path components"),
                "unexpected error for {candidate:?}: {err}"
            );
        }
    }

    #[test]
    fn test_resolve_entry_path_accepts_relative() {
        let dir = tempdir().unwrap();
        let base = base_path(dir.path());
        let parser = Parser::new_with_fs(NativeFs {});

        let resolved = parser
            .resolve_entry_path(base.to_path(), "Section 1.one")
            .unwrap();
        assert_eq!(resolved, base.join("Section 1.one"));
    }
}
