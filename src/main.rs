use std::fs::read_to_string;

use crate::{combinators::combinators::ParseState, lexer::lexer::scan_tokens};

mod combinators;
mod lexer;

pub fn main() {
    let content = read_to_string("./example.rox").unwrap();

    let state = ParseState {
        source: content.chars().peekable(),
        line: 1,
        position: 0,
    };

    let (tokens, rest) = scan_tokens().parse(state).unwrap();

    println!("{:#?}", tokens);
    println!("{:#?}", rest);
}
