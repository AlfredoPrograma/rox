use crate::combinators::combinators::{
    Parser, bracket, chain, char, many1, map, map_with_rest, or, satisfy,
};

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

    Bang,
    BangEqual,
    Equal,
    DoubleEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Str,
    Number,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    kind: TokenKind,
    lexeme: String,
    line: i32,
    position: usize,
}

pub fn scan_tokens<'a>() -> Box<dyn Parser<'a, Vec<Token>> + 'a> {
    many1(or(vec![paired_chars(), single_char()]))
}

fn string<'a>() -> Box<dyn Parser<'a, Token> + 'a> {
    map_with_rest(
        bracket(
            char('"'),
            map(many1(satisfy(|ch| ch != '"')), |chs| {
                chs.into_iter().collect::<String>()
            }),
            char('"'),
        ),
        |(str, rest)| {
            (
                Token {
                    kind: TokenKind::Str,
                    lexeme: str,
                    line: rest.line,
                    position: rest.position,
                },
                rest,
            )
        },
    )
}

fn linebreaks<'a>() -> Box<dyn Parser<'a, ()> + 'a> {
    map(
        many1(map_with_rest(satisfy(|ch| ch == '\n'), |(_, mut rest)| {
            rest.line += 1;
            rest.position = 1;
            ((), rest)
        })),
        |_| (),
    )
}

fn whitespaces<'a>() -> Box<dyn Parser<'a, ()> + 'a> {
    map(
        many1(satisfy(|ch| ch == '\t' || ch == '\r' || ch == ' ')),
        |_| (),
    )
}

fn paired_chars<'a>() -> Box<dyn Parser<'a, Token> + 'a> {
    let concat_chars = |(x, y)| format!("{x}{y}");
    let char_as_str = |ch| format!("{ch}");

    let two_or_singles = or(vec![
        or(vec![
            map(chain(char('!'), char('=')), concat_chars),
            map(char('!'), char_as_str),
        ]),
        or(vec![
            map(chain(char('='), char('=')), concat_chars),
            map(char('='), char_as_str),
        ]),
        or(vec![
            map(chain(char('>'), char('=')), concat_chars),
            map(char('>'), char_as_str),
        ]),
        or(vec![
            map(chain(char('<'), char('=')), concat_chars),
            map(char('<'), char_as_str),
        ]),
    ]);

    map_with_rest(two_or_singles, |(parsed, rest)| {
        let lexeme = parsed;

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

fn single_char<'a>() -> Box<dyn Parser<'a, Token> + 'a> {
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

    map_with_rest(allowed_chars, |(ch, rest)| {
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
        "!=" => TokenKind::BangEqual,
        "==" => TokenKind::DoubleEqual,
        ">=" => TokenKind::GreaterEqual,
        "<=" => TokenKind::LessEqual,
        "!" => TokenKind::Bang,
        "=" => TokenKind::Equal,
        ">" => TokenKind::Greater,
        "<" => TokenKind::Less,
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
    use std::result;

    use crate::combinators::combinators::{ParseState, ParseStateBuilder};

    use super::*;

    fn compare_states(s1: ParseState, s2: ParseState) -> bool {
        s1.source.collect::<String>() == s2.source.collect::<String>()
            && s1.line == s2.line
            && s1.position == s2.position
    }

    #[test]
    fn test_single_char() {
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

    #[test]
    fn test_two_or_single_char() {
        let source = "!><!===>=<==";
        let input = ParseStateBuilder::default().source(source).build();
        let result = many1(paired_chars()).parse(input);
        let expected_parsed = vec![
            Token {
                kind: TokenKind::Bang,
                lexeme: "!".to_string(),
                position: 1,
                line: 1,
            },
            Token {
                kind: TokenKind::Greater,
                lexeme: ">".to_string(),
                position: 2,
                line: 1,
            },
            Token {
                kind: TokenKind::Less,
                lexeme: "<".to_string(),
                position: 3,
                line: 1,
            },
            Token {
                kind: TokenKind::BangEqual,
                lexeme: "!=".to_string(),
                position: 5,
                line: 1,
            },
            Token {
                kind: TokenKind::DoubleEqual,
                lexeme: "==".to_string(),
                position: 7,
                line: 1,
            },
            Token {
                kind: TokenKind::GreaterEqual,
                lexeme: ">=".to_string(),
                position: 9,
                line: 1,
            },
            Token {
                kind: TokenKind::LessEqual,
                lexeme: "<=".to_string(),
                position: 11,
                line: 1,
            },
            Token {
                kind: TokenKind::Equal,
                lexeme: "=".to_string(),
                position: 12,
                line: 1,
            },
        ];
        let expected_state = ParseStateBuilder::default()
            .source("")
            .position(source.len())
            .build();

        assert!(result.is_ok_and(|(parsed, state)| {
            parsed == expected_parsed && compare_states(state, expected_state)
        }))
    }

    #[test]
    fn test_whitespaces() {
        let source = " \r\t ";
        let input = ParseStateBuilder::default().source(source).build();
        let result = whitespaces().parse(input);
        let expected_parsed = ();
        let expected_state = ParseStateBuilder::default()
            .source("")
            .position(source.len())
            .build();

        assert!(result.is_ok_and(|(parsed, state)| {
            parsed == expected_parsed && compare_states(state, expected_state)
        }))
    }

    #[test]
    fn test_linebreaks() {
        let source = "\n\n\n";
        let input = ParseStateBuilder::default().source(source).build();
        let result = linebreaks().parse(input);
        let expected_parsed = ();
        let expected_state = ParseStateBuilder::default()
            .source("")
            .position(1)
            .line(4)
            .build();

        assert!(result.is_ok_and(|(parsed, state)| {
            parsed == expected_parsed && compare_states(state, expected_state)
        }))
    }

    #[test]
    fn test_string() {
        let source = "\"Hello world\"";
        let input = ParseStateBuilder::default().source(source).build();
        let (parsed, state) = string().parse(input).unwrap();
        let expected_parsed = Token {
            kind: TokenKind::Str,
            lexeme: "Hello world".to_string(),
            position: source.len(),
            line: 1,
        };
        let expected_state = ParseStateBuilder::default()
            .source("")
            .position(source.len())
            .build();

        assert_eq!(parsed, expected_parsed);
        assert!(compare_states(state, expected_state))
    }
}
