#![allow(unused, dead_code)]
use std::{iter::Peekable, str::Chars};

pub struct ParseStateBuilder<'a> {
    source: Option<Peekable<Chars<'a>>>,
    line: Option<i32>,
    position: Option<usize>,
}

impl<'a> std::default::Default for ParseStateBuilder<'a> {
    fn default() -> Self {
        Self {
            source: None,
            line: None,
            position: None,
        }
    }
}

impl<'a> ParseStateBuilder<'a> {
    pub fn source(mut self, source: &'a str) -> Self {
        self.source = Some(source.chars().peekable());
        self
    }

    pub fn line(mut self, line: i32) -> Self {
        self.line = Some(line);
        self
    }

    pub fn position(mut self, position: usize) -> Self {
        self.position = Some(position);
        self
    }

    pub fn build(self) -> ParseState<'a> {
        ParseState {
            source: self.source.unwrap_or("".chars().peekable()),
            line: self.line.unwrap_or(1),
            position: self.position.unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseState<'a> {
    pub source: Peekable<Chars<'a>>,
    pub line: i32,
    pub position: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    CannotGetNext,
    PredicateFailed,
    ChainFailed(Box<ParseError>),
    NoneParserMatched,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::CannotGetNext => write!(f, "cannot get next character"),
            ParseError::PredicateFailed => write!(f, "predicate failed"),
            ParseError::ChainFailed(err) => write!(f, "chain failed by: {}", err),
            ParseError::NoneParserMatched => write!(f, "none of parsers matched"),
        }
    }
}
impl std::error::Error for ParseError {}

type ParseResult<'a, Output> = Result<(Output, ParseState<'a>), ParseError>;

pub trait Parser<'a, Output> {
    fn parse(&self, input: ParseState<'a>) -> ParseResult<'a, Output>;
}

pub fn map<'a, Output, MapFn, MapInto>(
    parser: Box<dyn Parser<'a, Output> + 'a>,
    transform: MapFn,
) -> Box<dyn Parser<'a, MapInto> + 'a>
where
    Output: 'a,
    MapFn: Fn(Output) -> MapInto + 'a,
{
    Box::new(move |state| parser.parse(state).map(|(x, rest)| (transform(x), rest)))
}

pub fn map_with_rest<'a, Output, MapFn, MapInto>(
    parser: Box<dyn Parser<'a, Output> + 'a>,
    transform: MapFn,
) -> Box<dyn Parser<'a, MapInto> + 'a>
where
    Output: 'a,
    MapFn: Fn((Output, ParseState<'a>)) -> (MapInto, ParseState<'a>) + 'a,
{
    Box::new(move |state: ParseState<'a>| parser.parse(state).map(|(x, rest)| transform((x, rest))))
}

impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(ParseState<'a>) -> ParseResult<'a, Output> + 'a,
{
    fn parse(&self, input: ParseState<'a>) -> ParseResult<'a, Output> {
        self(input)
    }
}

pub fn next<'a>() -> Box<dyn Parser<'a, char> + 'a> {
    Box::new(move |mut state: ParseState<'a>| {
        let ch = state.source.next().ok_or(ParseError::CannotGetNext)?;
        state.position += ch.len_utf8();
        Ok((ch, state))
    })
}

pub fn satisfy<'a, F>(predicate: F) -> Box<dyn Parser<'a, char> + 'a>
where
    F: Fn(char) -> bool + 'a,
{
    Box::new(move |mut state: ParseState<'a>| {
        let ch = state
            .source
            .peek()
            .ok_or(ParseError::CannotGetNext)?
            .to_owned();

        if !predicate(ch) {
            return Err(ParseError::PredicateFailed);
        }

        state.source.next();
        state.position += ch.len_utf8();
        return Ok((ch, state));
    })
}

pub fn char<'a>(target: char) -> Box<dyn Parser<'a, char> + 'a> {
    satisfy(move |ch| ch == target)
}

pub fn chain<'a, T, K>(
    p1: Box<dyn Parser<'a, T> + 'a>,
    p2: Box<dyn Parser<'a, K> + 'a>,
) -> Box<dyn Parser<'a, (T, K)> + 'a>
where
    T: 'a,
    K: 'a,
{
    Box::new(move |state: ParseState<'a>| {
        let (x1, rest) = p1
            .parse(state)
            .map_err(|err| ParseError::ChainFailed(Box::new(err)))?;
        let (x2, rest) = p2
            .parse(rest)
            .map_err(|err| ParseError::ChainFailed(Box::new(err)))?;
        Ok(((x1, x2), rest))
    })
}

pub fn or<'a, T>(ps: Vec<Box<dyn Parser<'a, T> + 'a>>) -> Box<dyn Parser<'a, T> + 'a>
where
    T: 'a,
{
    Box::new(move |state: ParseState<'a>| {
        for p in &ps {
            let result = p.parse(state.clone());

            if result.is_ok() {
                return result;
            }
        }

        Err(ParseError::NoneParserMatched)
    })
}

pub fn many1<'a, T>(p: Box<dyn Parser<'a, T> + 'a>) -> Box<dyn Parser<'a, Vec<T>> + 'a>
where
    T: 'a,
{
    Box::new(move |state: ParseState<'a>| {
        let mut results = vec![];
        let mut input = state;

        while let Ok((r, rest)) = p.parse(input.clone()) {
            results.push(r);
            input = rest
        }

        if results.is_empty() {}
        Ok((results, input))
    })
}

#[cfg(test)]
mod combinators_tests {
    use super::*;

    fn compare_states(s1: ParseState, s2: ParseState) -> bool {
        s1.source.collect::<String>() == s2.source.collect::<String>()
            && s1.line == s2.line
            && s1.position == s2.position
    }

    #[test]
    fn test_next() {
        let input = ParseStateBuilder::default().source("Hello").build();
        let result = next().parse(input);
        let expected_ch = 'H';
        let expected_state = ParseStateBuilder::default()
            .source("ello")
            .position(expected_ch.len_utf8())
            .build();

        assert!(result.is_ok_and(|(ch, state)| {
            ch == expected_ch && compare_states(state, expected_state)
        }))
    }

    #[test]
    fn test_next_fails_by_cannot_get_next() {
        let input = ParseStateBuilder::default().build();
        let result = next().parse(input);

        assert!(result.is_err_and(|err| err == ParseError::CannotGetNext))
    }

    #[test]
    fn test_satisfy() {
        let input = ParseStateBuilder::default().source("Hello").build();
        let result = satisfy(|ch| ch.is_uppercase()).parse(input);
        let expected_ch = 'H';
        let expected_state = ParseStateBuilder::default()
            .source("ello")
            .position(expected_ch.len_utf8())
            .build();

        assert!(result.is_ok_and(|(ch, state)| {
            ch == expected_ch && compare_states(state, expected_state)
        }))
    }

    #[test]
    fn test_satisfy_fails_by_cannot_get_next() {
        let input = ParseStateBuilder::default().build();
        let result = satisfy(|ch| ch.is_uppercase()).parse(input);

        assert!(result.is_err_and(|err| err == ParseError::CannotGetNext))
    }

    #[test]
    fn test_satisfy_fails_by_failed_predicate() {
        let input = ParseStateBuilder::default().source("hello").build();
        let result = satisfy(|ch| ch.is_uppercase()).parse(input);

        assert!(result.is_err_and(|err| err == ParseError::PredicateFailed))
    }

    #[test]
    fn test_chain() {
        let input = ParseStateBuilder::default().source("Hello").build();
        let result = chain(next(), satisfy(|ch| ch.is_lowercase())).parse(input);
        let expected_parsed = ('H', 'e');
        let expected_state = ParseStateBuilder::default()
            .source("llo")
            .position(expected_parsed.0.len_utf8() + expected_parsed.1.len_utf8())
            .build();

        assert!(result.is_ok_and(|((parsed1, parsed2), state)| {
            parsed1 == expected_parsed.0
                && parsed2 == expected_parsed.1
                && compare_states(state, expected_state)
        }))
    }

    #[test]
    fn test_chain_fails_by_first_parser() {
        let input = ParseStateBuilder::default().build();
        let result = chain(next(), satisfy(|ch| ch.is_lowercase())).parse(input);

        assert!(
            result.is_err_and(
                |err| err == ParseError::ChainFailed(Box::new(ParseError::CannotGetNext))
            )
        );
    }

    #[test]
    fn test_chain_fails_by_second_parser() {
        let input = ParseStateBuilder::default().source("HE").build();
        let result = chain(next(), satisfy(|ch| ch.is_lowercase())).parse(input);

        assert!(
            result.is_err_and(
                |err| err == ParseError::ChainFailed(Box::new(ParseError::PredicateFailed))
            )
        )
    }
}
