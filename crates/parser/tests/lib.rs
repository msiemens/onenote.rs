use bytes::Bytes;
use insta::assert_debug_snapshot;
use onenote_parser::Parser;
use onenote_parser::contents::{Content, OutlineElement, OutlineItem, TextHyperlink};
use onenote_parser::fs::native_fs::NativeFs;
use onenote_parser::fs::{FileSource, FileSystem};
use std::io;
use std::sync::Arc;
use typed_path::{TypedPath, TypedPathBuf};
use yare::parameterized;

fn tp(s: &str) -> TypedPath<'_> {
    TypedPath::derive(s)
}

#[test]
fn test_parse_section() {
    let path = tp("tests/samples/New Section 1.one");

    let parser = Parser::new();
    assert_debug_snapshot!(parser.parse_section(path).unwrap());
}

#[test]
fn test_parse_notebook() {
    let path = tp("tests/samples/Open Notebook.onetoc2");

    let parser = Parser::new();
    assert_debug_snapshot!(parser.parse_notebook(path).unwrap());
}

#[test]
fn test_parse_notebook_new() {
    let path = tp("tests/samples/non-legacy/Open Notebook.onetoc2");

    let parser = Parser::new();
    assert_debug_snapshot!(parser.parse_notebook(path).unwrap());
}

/// Regression test for parsing a `.onetoc2` exported from OneNote Desktop after copying pages
/// between sections. Such files describe their table of contents using object revisions
/// (`ObjectRevisionWithRefCountFNDX`, [MS-ONESTORE] 2.5.13) that inherit their GUID from a
/// dependency revision's global ID table via `GlobalIdTableEntry2FNDX` ([MS-ONESTORE] 2.5.11).
/// The parser used to reject those nodes outright. See issue #31.
#[test]
fn test_parse_notebook_object_revision() {
    let path = tp("tests/samples/object-revision/Open Notebook.onetoc2");

    let parser = Parser::new();
    assert_debug_snapshot!(parser.parse_notebook(path).unwrap());
}

#[test]
fn test_parse_section_with_image_missing_last_modified() {
    let path = tp("tests/samples/Schnelle Notizen.one");

    let parser = Parser::new();
    assert_debug_snapshot!(parser.parse_section(path).unwrap());
}

#[test]
fn test_readme_example_parse_notebook() {
    let parser = Parser::new();
    let notebook = parser
        .parse_notebook(tp("tests/samples/Open Notebook.onetoc2"))
        .unwrap();

    assert!(!notebook.entries().is_empty());
}

#[test]
fn test_parse_section_handwriting_recognition() {
    let path = tp("tests/samples/handwriting_recognition.one");

    let parser = Parser::new();
    assert_debug_snapshot!(parser.parse_section(path).unwrap());
}

#[test]
fn test_onenote_2016_parse_notebook() {
    let path = tp("tests/samples/onenote-2016/OneWithFileData.one");
    let parser = Parser::new();

    assert_debug_snapshot!(parser.parse_section(path).unwrap())
}

#[parameterized(
    checkboxes_and_unicode = { "checkboxes_and_unicode.one" },
    math = { "Math.one" },
    onenote_desktop = { "onenote_desktop.one" },
    quick_notes = { "aaa/Quick Notes.one" },
    audio_test = { "audio-test/Quick Notes.one" },
    hyperlink_is_broken = { "hyperlink_is_broken/Quick Notes.one" },
    subsections_subpages = { "Notebook with subsections and subpages/Section 1.one" },
    notebook_with_chinese_char_on_link = { "notebook_with_chinese_char_on_link/Quick Notes.one" },
    simple_notebook = { "Simple notebook/Quick Notes.one" },
    new_section = { "new_section.one" },
    scaled_ink = { "scaled_ink.one" },
    desktop_missing_ink = { "desktop_missing_ink.one" }
)]
fn test_onenote_joplin_examples_section(path: &str) {
    let parser = Parser::new();
    let full = format!("tests/samples/joplin/{path}");
    assert_debug_snapshot!(parser.parse_section(tp(&full)).unwrap());
}

#[parameterized(
    default_edited = { "default-edited/Открыть записную книжку.onetoc2" },
    onenote_app = { "Notebook created on OneNote App/Abrir Bloco de Anotações.onetoc2" }
)]
fn test_onenote_joplin_examples_notebook(path: &str) {
    let parser = Parser::new();
    let full = format!("tests/samples/joplin/{path}");
    assert_debug_snapshot!(parser.parse_notebook(tp(&full)).unwrap());
}

#[test]
fn test_structured_hyperlinks_from_public_fixture() {
    let section = Parser::new()
        .parse_section(tp(
            "tests/samples/joplin/notebook_with_chinese_char_on_link/Quick Notes.one",
        ))
        .unwrap();
    let mut links = Vec::new();
    for page in section
        .page_series()
        .iter()
        .flat_map(|series| series.pages())
    {
        for outline in page
            .contents()
            .iter()
            .filter_map(|content| content.outline())
        {
            collect_outline_links(outline.items(), &mut links);
        }
    }

    assert!(links.len() >= 2);
    assert!(
        links
            .iter()
            .any(|link| link.target().starts_with("onenote:"))
    );
    assert!(
        links
            .iter()
            .any(|link| link.target().starts_with("https://"))
    );
    assert!(links.iter().all(|link| link.start() < link.end()));
}

fn collect_outline_links(items: &[OutlineItem], output: &mut Vec<TextHyperlink>) {
    for item in items {
        match item {
            OutlineItem::Element(element) => collect_element_links(element, output),
            OutlineItem::Group(group) => collect_outline_links(group.outlines(), output),
        }
    }
}

fn collect_element_links(element: &OutlineElement, output: &mut Vec<TextHyperlink>) {
    for content in element.contents() {
        if let Content::RichText(text) = content {
            output.extend(text.hyperlinks());
        }
    }
    collect_outline_links(element.children(), output);
}

// ---------------------------------------------------------------------------
// Lazy-source coverage
// ---------------------------------------------------------------------------
//
// Mirrors a hypothetical WASM consumer that holds the file's bytes elsewhere
// (e.g. a JS Blob) and only serves them on demand: `as_bytes()` returns
// `None`, so the parser exercises the `read_at`-only path in Reader and
// FileBlob. We then assert that the parsed output is byte-for-byte identical
// to the eager (NativeFs / mmap) path against the same fixtures.

/// A [`FileSource`] that holds the buffer but refuses to expose it via
/// `as_bytes`, forcing every read through `read_at`.
struct LazyBytesSource(Bytes);

impl FileSource for LazyBytesSource {
    fn byte_length(&self) -> u64 {
        self.0.len() as u64
    }

    fn read_at(&self, offset: u64, len: usize) -> Result<Bytes, io::Error> {
        let start = offset as usize;
        let end = start.checked_add(len).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "LazyBytesSource overflow")
        })?;
        if end > self.0.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "LazyBytesSource read past end",
            ));
        }
        Ok(self.0.slice(start..end))
    }
    // Deliberately no `as_bytes` override — uses the trait default (`None`).
}

/// A `FileSystem` that delegates everything to [`NativeFs`] except
/// `open_file`, which wraps the file contents in a [`LazyBytesSource`].
#[derive(Clone, Copy)]
struct LazyFs;

impl FileSystem for LazyFs {
    fn is_directory(&self, path: TypedPath) -> io::Result<bool> {
        NativeFs {}.is_directory(path)
    }
    fn read_dir(&self, path: TypedPath) -> io::Result<Vec<TypedPathBuf>> {
        NativeFs {}.read_dir(path)
    }
    fn read_file(&self, path: TypedPath) -> io::Result<Vec<u8>> {
        NativeFs {}.read_file(path)
    }
    fn write_file(&self, path: TypedPath, data: &[u8]) -> io::Result<()> {
        NativeFs {}.write_file(path, data)
    }
    fn stream_to_file(&self, path: TypedPath, reader: &mut dyn io::Read) -> io::Result<()> {
        NativeFs {}.stream_to_file(path, reader)
    }
    fn make_dir(&self, path: TypedPath) -> io::Result<()> {
        NativeFs {}.make_dir(path)
    }
    fn canonicalize(&self, path: TypedPath) -> io::Result<TypedPathBuf> {
        NativeFs {}.canonicalize(path)
    }
    fn exists(&self, path: TypedPath) -> io::Result<bool> {
        NativeFs {}.exists(path)
    }
    fn open_file(&self, path: TypedPath) -> io::Result<Arc<dyn FileSource>> {
        let bytes = Bytes::from(self.read_file(path)?);
        Ok(Arc::new(LazyBytesSource(bytes)))
    }
}

#[parameterized(
    checkboxes_and_unicode = { "checkboxes_and_unicode.one" },
    math = { "Math.one" },
    onenote_desktop = { "onenote_desktop.one" },
    quick_notes = { "aaa/Quick Notes.one" },
    audio_test = { "audio-test/Quick Notes.one" },
    hyperlink_is_broken = { "hyperlink_is_broken/Quick Notes.one" },
    subsections_subpages = { "Notebook with subsections and subpages/Section 1.one" },
    notebook_with_chinese_char_on_link = { "notebook_with_chinese_char_on_link/Quick Notes.one" },
    simple_notebook = { "Simple notebook/Quick Notes.one" },
    new_section = { "new_section.one" },
    scaled_ink = { "scaled_ink.one" },
    desktop_missing_ink = { "desktop_missing_ink.one" }
)]
fn test_lazy_source_section_matches_eager(path: &str) {
    let full = format!("tests/samples/joplin/{path}");
    let eager = Parser::new().parse_section(tp(&full)).unwrap();
    let lazy = Parser::new_with_fs(LazyFs)
        .parse_section(tp(&full))
        .unwrap();
    assert_eq!(format!("{:#?}", eager), format!("{:#?}", lazy));
}

#[parameterized(
    default_edited = { "default-edited/Открыть записную книжку.onetoc2" },
    onenote_app = { "Notebook created on OneNote App/Abrir Bloco de Anotações.onetoc2" }
)]
fn test_lazy_source_notebook_matches_eager(path: &str) {
    let full = format!("tests/samples/joplin/{path}");
    let eager = Parser::new().parse_notebook(tp(&full)).unwrap();
    let lazy = Parser::new_with_fs(LazyFs)
        .parse_notebook(tp(&full))
        .unwrap();
    assert_eq!(format!("{:#?}", eager), format!("{:#?}", lazy));
}
