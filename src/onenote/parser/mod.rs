use bytes::Bytes;

use crate::errors::Result;
use crate::fsshttpb::packaging::Packaging;

use crate::onestore::parse_store;

use crate::onenote::parser::notebook::Notebook;
use crate::onenote::parser::section::Section;
use crate::types::guid::Guid;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

mod content;
mod embedded_file;
mod image;
mod list;
mod notebook;
mod outline;
mod page;
mod page_content;
mod page_series;
mod rich_text;
mod section;
mod table;

pub struct Parser {}

impl Parser {
    pub fn new() -> Parser {
        Parser {}
    }

    pub fn parse_notebook(&mut self, path: &Path) -> Result<Notebook> {
        let file = File::open(path)?;
        let data = Parser::read(file)?;
        let packaging = Packaging::parse(&mut Bytes::from(data))?;
        let store = parse_store(packaging)?;

        assert_eq!(
            store.schema_guid(),
            Guid::from_str("E4DBFD38-E5C7-408B-A8A1-0E7B421E1F5F").unwrap()
        );

        let root_revisions: Vec<_> = store
            .data_root()
            .revisions()
            .values()
            .filter(|rev| !rev.roots().is_empty())
            .collect();

        assert_eq!(root_revisions.len(), 1);

        let root_revision = root_revisions.first().unwrap();

        // FIXME: Try interpreting UTF-8 as Latin1 if file not found!

        let base_dir = path.parent().expect("no base dir found");
        let sections = notebook::parse_toc(root_revision, &store)
            .iter()
            .map(|name| {
                let mut file = base_dir.to_path_buf();
                file.push(name);

                file
            })
            .inspect(|path| {
                dbg!(path.display());
            })
            .filter(|p| p.exists() && p.is_file())
            .map(|path| self.parse_section(&path))
            .collect::<Result<_>>()?;

        Ok(Notebook { sections })
    }

    pub fn parse_section(&mut self, path: &Path) -> Result<Section> {
        let file = File::open(path)?;
        let data = Parser::read(file)?;
        let packaging = Packaging::parse(&mut Bytes::from(data))?;
        let store = parse_store(packaging)?;

        assert_eq!(
            store.schema_guid(),
            Guid::from_str("1F937CB4-B26F-445F-B9F8-17E20160E461").unwrap()
        );

        let root_revisions: Vec<_> = store
            .data_root()
            .revisions()
            .values()
            .filter(|rev| !rev.roots().is_empty())
            .collect();

        assert_eq!(root_revisions.len(), 1);

        let root_revision = root_revisions.first().unwrap();

        Ok(section::parse_section(root_revision, &store))
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
