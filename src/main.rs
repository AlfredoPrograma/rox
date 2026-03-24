use std::fs::read_to_string;

use crate::{
    ast::ast::expression, combinators::combinators::ParseState, lexer::lexer::scan_tokens,
};

mod ast;
mod combinators;
mod lexer;

pub fn main() {
    let content = read_to_string("./example.rox").unwrap();

    let state = ParseState {
        source: content.chars().peekable(),
        line: 1,
        position: 0,
    };

    let (tokens, _) = scan_tokens().parse(state).unwrap();
    match expression().parse(tokens) {
        Ok((expr, rest)) => {
            println!("{}", expr);
            println!("{:#?}", rest)
        }
        Err(err) => eprintln!("{}", err),
    }
}
