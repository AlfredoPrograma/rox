use core::fmt;

use crate::lexer::lexer::{Token, TokenKind};

#[derive(Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Str(String),
    Number(f64),
    Group(Box<ASTExpression>),
}

#[derive(Debug, PartialEq)]
enum PrefixOperator {
    BoolNegate,   // "!"
    NumberNegate, // "-"
}

#[derive(Debug, PartialEq)]
pub struct PrefixOperation {
    op: PrefixOperator,
    value: Box<ASTExpression>,
}

#[derive(Debug, PartialEq)]
enum InfixOperator {
    Add,
    Substract,
    Multiply,
    Divide,
    GT,
    GTE,
    LT,
    LTE,
}

#[derive(Debug, PartialEq)]
pub struct InfixOperation {
    lhs: Box<ASTExpression>,
    op: InfixOperator,
    rhs: Box<ASTExpression>,
}

#[derive(Debug, PartialEq)]
pub enum ASTExpression {
    Equality(InfixOperation),
    Comparison(InfixOperation),
    Term(InfixOperation),
    Factor(InfixOperation),
    Unary(PrefixOperation),
    Primary(Value),
}

#[derive(Debug)]
pub enum ASTParseError {
    NoTokensLeft,
    PredicateFailed,
}

impl fmt::Display for ASTParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTParseError::NoTokensLeft => write!(f, "No more tokens left"),
            ASTParseError::PredicateFailed => write!(f, "Predicate failed"),
        }
    }
}

impl std::error::Error for ASTParseError {}

type ASTParseResult = Result<(ASTExpression, Vec<Token>), ASTParseError>;

pub trait ASTParser {
    fn parse(self, input: Vec<Token>) -> ASTParseResult;
}

impl<F> ASTParser for F
where
    F: FnOnce(Vec<Token>) -> ASTParseResult,
{
    fn parse(self, input: Vec<Token>) -> ASTParseResult {
        self(input)
    }
}

fn primary() -> impl ASTParser
where {
    |state: Vec<Token>| {
        let mut state = state.into_iter();
        let next = state.next().ok_or(ASTParseError::NoTokensLeft)?;
        let state = state.collect();

        match next.kind {
            TokenKind::Nil => Ok((ASTExpression::Primary(Value::Nil), state)),
            TokenKind::True => Ok((ASTExpression::Primary(Value::Bool(true)), state)),
            TokenKind::False => Ok((ASTExpression::Primary(Value::Bool(false)), state)),
            TokenKind::Str => Ok((
                ASTExpression::Primary(Value::Str(next.lexeme.clone())),
                state,
            )),
            TokenKind::Number => {
                let parsed = next.lexeme.clone().parse::<f64>().expect(
                    format!(
                        "captured lexeme {} from token cannot be parsed as f64",
                        next.lexeme.clone()
                    )
                    .as_str(),
                );

                Ok((ASTExpression::Primary(Value::Number(parsed)), state))
            }

            _ => Err(ASTParseError::PredicateFailed),
        }
    }
}

#[cfg(test)]
mod ast_tests {
    use crate::{
        ast::ast::{ASTExpression, ASTParser, Value, primary},
        lexer::lexer::{Token, TokenKind},
    };

    #[test]
    fn test_primary_for_nil() {
        let input = vec![Token::new(TokenKind::Nil, "nil".to_string())];
        let (expr, state) = primary().parse(input).unwrap();
        let expected_expr = ASTExpression::Primary(Value::Nil);
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_primary_for_bool_true() {
        let input = vec![Token::new(TokenKind::True, "true".to_string())];
        let (expr, state) = primary().parse(input).unwrap();
        let expected_expr = ASTExpression::Primary(Value::Bool(true));
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_primary_for_bool_false() {
        let input = vec![Token::new(TokenKind::False, "false".to_string())];
        let (expr, state) = primary().parse(input).unwrap();
        let expected_expr = ASTExpression::Primary(Value::Bool(false));
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_primary_for_string() {
        let input = vec![Token::new(TokenKind::Str, "hello world".to_string())];
        let (expr, state) = primary().parse(input).unwrap();
        let expected_expr = ASTExpression::Primary(Value::Str("hello world".to_string()));
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_primary_for_number() {
        let input = vec![Token::new(TokenKind::Number, "10".to_string())];
        let (expr, state) = primary().parse(input).unwrap();
        let expected_expr = ASTExpression::Primary(Value::Number(10.0));
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }
}
