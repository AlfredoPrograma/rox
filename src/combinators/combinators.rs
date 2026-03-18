use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone)]
pub struct ParseState<'a> {
    source: Peekable<Chars<'a>>,
    line: i32,
    position: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    CannotGetNext,
    PredicateFailed,
    ChainFailed(Box<ParseError>),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::CannotGetNext => write!(f, "cannot get next character"),
            ParseError::PredicateFailed => write!(f, "predicate failed"),
            ParseError::ChainFailed(err) => write!(f, "chain failed by: {}", err),
        }
    }
}
impl std::error::Error for ParseError {}

type ParseResult<'a, Output> = Result<(Output, ParseState<'a>), ParseError>;

pub trait Parser<'a, Output> {
    fn parse(&self, input: ParseState<'a>) -> ParseResult<'a, Output>;
}

impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(ParseState<'a>) -> ParseResult<'a, Output>,
{
    fn parse(&self, input: ParseState<'a>) -> ParseResult<'a, Output> {
        self(input)
    }
}

pub fn next<'a>() -> impl Parser<'a, char> {
    move |mut state: ParseState<'a>| {
        let ch = state.source.next().ok_or(ParseError::CannotGetNext)?;
        state.position += ch.len_utf8();
        Ok((ch, state))
    }
}

pub fn satisfy<'a, F>(predicate: F) -> impl Parser<'a, char>
where
    F: Fn(char) -> bool,
{
    move |mut state: ParseState<'a>| {
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
    }
}

pub fn chain<'a, T, K>(p1: impl Parser<'a, T>, p2: impl Parser<'a, K>) -> impl Parser<'a, (T, K)> {
    move |state: ParseState<'a>| {
        let (x1, rest) = p1
            .parse(state)
            .map_err(|err| ParseError::ChainFailed(Box::new(err)))?;
        let (x2, rest) = p2
            .parse(rest)
            .map_err(|err| ParseError::ChainFailed(Box::new(err)))?;
        Ok(((x1, x2), rest))
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    fn define_state<'a>(
        input: &'a str,
        position: Option<usize>,
        line: Option<i32>,
    ) -> ParseState<'a> {
        ParseState {
            source: input.chars().peekable(),
            position: 1 + position.unwrap_or(0), // automatically add position 1 offset
            line: line.unwrap_or(1),
        }
    }

    fn compare_states(s1: ParseState, s2: ParseState) -> bool {
        s1.source.collect::<String>() == s2.source.collect::<String>()
            && s1.line == s2.line
            && s1.position == s2.position
    }

    #[test]
    fn test_next() {
        let input = define_state("Hello", None, None);
        let result = next().parse(input);
        let expected_ch = 'H';
        let expected_state = define_state("ello", Some(expected_ch.len_utf8()), None);

        assert!(result.is_ok_and(|(ch, state)| {
            ch == expected_ch && compare_states(state, expected_state)
        }))
    }

    #[test]
    fn test_next_fails_by_cannot_get_next() {
        let input = define_state("", None, None);
        let result = next().parse(input);

        assert!(result.is_err_and(|err| err == ParseError::CannotGetNext))
    }

    #[test]
    fn test_satisfy() {
        let input = define_state("Hello", None, None);
        let result = satisfy(|ch| ch.is_uppercase()).parse(input);
        let expected_ch = 'H';
        let expected_state = define_state("ello", Some(expected_ch.len_utf8()), None);

        assert!(result.is_ok_and(|(ch, state)| {
            ch == expected_ch && compare_states(state, expected_state)
        }))
    }

    #[test]
    fn test_satisfy_fails_by_cannot_get_next() {
        let input = define_state("", None, None);
        let result = satisfy(|ch| ch.is_uppercase()).parse(input);

        assert!(result.is_err_and(|err| err == ParseError::CannotGetNext))
    }

    #[test]
    fn test_satisfy_fails_by_failed_predicate() {
        let input = define_state("hello", None, None);
        let result = satisfy(|ch| ch.is_uppercase()).parse(input);

        assert!(result.is_err_and(|err| err == ParseError::PredicateFailed))
    }

    #[test]
    fn test_chain() {
        let input = define_state("Hello", None, None);
        let result = chain(next(), satisfy(|ch| ch.is_lowercase())).parse(input);
        let expected_parsed = ('H', 'e');
        let expected_state = define_state(
            "llo",
            Some(expected_parsed.0.len_utf8() + expected_parsed.1.len_utf8()),
            None,
        );

        assert!(result.is_ok_and(|((parsed1, parsed2), state)| {
            parsed1 == expected_parsed.0
                && parsed2 == expected_parsed.1
                && compare_states(state, expected_state)
        }))
    }

    #[test]
    fn test_chain_fails_by_first_parser() {
        let input = define_state("", None, None);
        let result = chain(next(), satisfy(|ch| ch.is_lowercase())).parse(input);

        assert!(
            result.is_err_and(
                |err| err == ParseError::ChainFailed(Box::new(ParseError::CannotGetNext))
            )
        );
    }

    #[test]
    fn test_chain_fails_by_second_parser() {
        let input = define_state("HE", None, None);
        let result = chain(next(), satisfy(|ch| ch.is_lowercase())).parse(input);

        assert!(
            result.is_err_and(
                |err| err == ParseError::ChainFailed(Box::new(ParseError::PredicateFailed))
            )
        )
    }
}
