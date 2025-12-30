use onenote_parser::Parser;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

fn main() {
    let path = env::args().nth(1).expect("usage: parse <file>");
    let path = PathBuf::from(path);

    let parser = Parser::new();
    if path.extension() == Some(&OsString::from("onetoc2".to_string())) {
        let notebook = parser.parse_notebook(&path).unwrap();
        println!("{:#?}", notebook);
    } else {
        let section = parser.parse_section(&path).unwrap();
        println!("{:#?}", section);
    }
}
