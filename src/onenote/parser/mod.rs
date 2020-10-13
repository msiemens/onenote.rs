use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::packaging::Packaging;
use crate::onenote::parser::notebook::Notebook;
use crate::onenote::parser::section::Section;
use crate::onestore::parse_store;
use crate::reader::Reader;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub(crate) mod content;
pub(crate) mod embedded_file;
pub(crate) mod image;
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

pub struct Parser {}

impl Parser {
    pub fn new() -> Parser {
        Parser {}
    }

    pub fn parse_notebook(&mut self, path: &Path) -> Result<Notebook> {
        let file = File::open(path)?;
        let data = Parser::read(file)?;
        let packaging = Packaging::parse(&mut Reader::new(data.as_slice()))?;
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
            // .inspect(|path| {
            //     dbg!(path.display());
            // })
            .filter(|p| p.is_file())
            .map(|path| self.parse_section(&path))
            .collect::<Result<_>>()?;

        Ok(Notebook { sections })
    }

    pub fn parse_section(&mut self, path: &Path) -> Result<Section> {
        let file = File::open(path)?;
        let data = Parser::read(file)?;
        let packaging = Packaging::parse(&mut Reader::new(data.as_slice()))?;
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
