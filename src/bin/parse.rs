use std::env;

use onenote::Parser;

fn main() {
    let path = env::args().nth(1).expect("usage: parse <file>");

    let data = std::fs::read(path).expect("Failed to read file");

    let mut parser = Parser::new(data);

    println!("{:#?}", parser.parse().expect("failed to parse file"))
}
