use insta::assert_debug_snapshot;
use onenote_parser::Parser;
use std::path::PathBuf;

#[test]
fn test_parse_section() {
    let path = PathBuf::from("tests/samples/New Section 1.one");

    let mut parser = Parser::new();
    assert_debug_snapshot!(parser.parse_section(&path).unwrap());
}

#[test]
fn test_parse_notebook() {
    let path = PathBuf::from("tests/samples/Open Notebook.onetoc2");

    let mut parser = Parser::new();
    assert_debug_snapshot!(parser.parse_notebook(&path).unwrap());
}
