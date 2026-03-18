use crate::combinators::combinators::{Parser, char, many1, or};

#[derive(Debug, PartialEq)]
enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    kind: TokenKind,
    lexeme: String,
    line: i32,
    position: usize,
}

pub fn scan_tokens<'a>() -> impl Parser<'a, Vec<Token>> {
    many1(or(vec![single_char()]))
}

fn single_char<'a>() -> impl Parser<'a, Token> {
    let allowed_chars = or(vec![
        char('('),
        char(')'),
        char('{'),
        char('}'),
        char(','),
        char('.'),
        char('-'),
        char('+'),
        char(';'),
        char('/'),
        char('*'),
    ]);

    allowed_chars.map_with_rest(|(ch, rest)| {
        let lexeme = ch.to_string();
        (
            Token {
                kind: lexeme_to_token_kind(lexeme.as_str()),
                lexeme: lexeme,
                position: rest.position,
                line: rest.line,
            },
            rest,
        )
    })
}

fn lexeme_to_token_kind(lexeme: &str) -> TokenKind {
    match lexeme {
        "(" => TokenKind::LeftParen,
        ")" => TokenKind::RightParen,
        "{" => TokenKind::LeftBrace,
        "}" => TokenKind::RightBrace,
        "," => TokenKind::Comma,
        "." => TokenKind::Dot,
        "-" => TokenKind::Minus,
        "+" => TokenKind::Plus,
        ";" => TokenKind::Semicolon,
        "/" => TokenKind::Slash,
        "*" => TokenKind::Star,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod lexer_tests {
    use crate::combinators::combinators::{ParseState, ParseStateBuilder};

    use super::*;

    fn compare_states(s1: ParseState, s2: ParseState) -> bool {
        s1.source.collect::<String>() == s2.source.collect::<String>()
            && s1.line == s2.line
            && s1.position == s2.position
    }

    #[test]
    fn test_single_char_token() {
        let source = "(){},.-+;/*.";
        let input = ParseStateBuilder::default().source(source).build();
        let result = many1(single_char()).parse(input);
        let expected_parsed = source
            .char_indices()
            .map(|(i, ch)| Token {
                kind: lexeme_to_token_kind(ch.to_string().as_str()),
                lexeme: ch.to_string(),
                position: i + 1,
                line: 1,
            })
            .collect::<Vec<Token>>();
        let expected_state = ParseStateBuilder::default()
            .source("")
            .position(source.len())
            .build();

        assert!(
            result.is_ok_and(|(parsed, state)| parsed == expected_parsed
                && compare_states(state, expected_state))
        )
    }
}
