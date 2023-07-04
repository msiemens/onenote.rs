use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::packaging::OneStorePackaging;
use crate::onenote::notebook::Notebook;
use crate::onenote::section::{Section, SectionEntry, SectionGroup};
use crate::onestore::parse_store;
use crate::reader::Reader;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, Read, Cursor};
use std::path::Path;
use std::str::FromStr;

pub(crate) mod content;
pub(crate) mod embedded_file;
pub(crate) mod iframe;
pub(crate) mod image;
pub(crate) mod ink;
pub(crate) mod list;
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
pub struct Parser;

impl Parser {
    /// Create a new OneNote file parser.
    pub fn new() -> Parser {
        Parser {}
    }

    /// Parse a OneNote notebook.
    ///
    /// The `path` argument must point to a `.onetoc2` file. This will parse the
    /// table of contents of the notebook as well as all contained
    /// sections from the folder that the table of contents file is in.
    pub fn parse_notebook(&mut self, path: &Path) -> Result<Notebook> {
        let file = File::open(path)?;
        let data = Parser::read(file)?;
        let packaging = OneStorePackaging::parse(&mut Reader::new(data.as_slice()))?;
        let store = parse_store(&packaging)?;

        if store.schema_guid() != guid!({E4DBFD38-E5C7-408B-A8A1-0E7B421E1F5F}) {
            return Err(ErrorKind::NotATocFile {
                file: path.to_string_lossy().to_string(),
            }
            .into());
        }

        let base_dir = path.parent().expect("no base dir found");
        let sections = notebook::parse_toc(store.data_root())?
            .iter()
            .map(|name| {
                let mut file = base_dir.to_path_buf();
                file.push(name);

                file
            })
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

        Ok(Notebook { entries: sections })
    }

    /// Parse a OneNote section buffer.
    ///
    /// The `data` argument must contain a OneNote section.
    pub fn parse_section_buffer(&mut self, data: &[u8], file_name: &Path) -> Result<Section> {
        let packaging = OneStorePackaging::parse(&mut Reader::new(data))?;
        let store = parse_store(&packaging)?;

        if store.schema_guid() != guid!({1F937CB4-B26F-445F-B9F8-17E20160E461}) {
            return Err(ErrorKind::NotASectionFile {
                file: file_name.to_string_lossy().into_owned(),
            }
            .into());
        }

        section::parse_section(
            store,
            file_name.to_string_lossy().into_owned(),
        )
    }

    /// Parse a OneNote section file.
    ///
    /// The `path` argument must point to a `.one` file that contains a
    /// OneNote section.
    pub fn parse_section(&mut self, path: &Path) -> Result<Section> {
        let file = File::open(path)?;
        let data = Parser::read(file)?;
        let packaging = OneStorePackaging::parse(&mut Reader::new(data.as_slice()))?;
        let store = parse_store(&packaging)?;

        if store.schema_guid() != guid!({1F937CB4-B26F-445F-B9F8-17E20160E461}) {
            return Err(ErrorKind::NotASectionFile {
                file: path.to_string_lossy().to_string(),
            }
            .into());
        }

        section::parse_section(
            store,
            path.file_name()
                .expect("file without file name")
                .to_string_lossy()
                .to_string(),
        )
    }

    fn parse_section_group(&mut self, path: &Path) -> Result<SectionGroup> {
        let display_name = path
            .file_name()
            .expect("file without file name")
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

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
