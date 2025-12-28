use insta::assert_debug_snapshot;
use onenote_parser::Parser;
use std::path::{Path, PathBuf};

#[test]
fn test_parse_section() {
    let path = PathBuf::from("tests/samples/New Section 1.one");

    let parser = Parser::new();
    assert_debug_snapshot!(parser.parse_section(&path).unwrap());
}

#[test]
fn test_parse_notebook() {
    let path = PathBuf::from("tests/samples/Open Notebook.onetoc2");

    let parser = Parser::new();
    assert_debug_snapshot!(parser.parse_notebook(&path).unwrap());
}

#[test]
fn test_parse_notebook_new() {
    let path = PathBuf::from("tests/samples/non-legacy/Open Notebook.onetoc2");

    let parser = Parser::new();
    assert_debug_snapshot!(parser.parse_notebook(&path).unwrap());
}

#[test]
fn test_parse_section_with_image_missing_last_modified() {
    let path = PathBuf::from("tests/samples/Schnelle Notizen.one");

    let parser = Parser::new();
    assert_debug_snapshot!(parser.parse_section(&path).unwrap());
}

#[test]
fn test_readme_example_parse_notebook() {
    let parser = Parser::new();
    let notebook = parser
        .parse_notebook(Path::new("tests/samples/Open Notebook.onetoc2"))
        .unwrap();

    assert!(!notebook.entries().is_empty());
}
