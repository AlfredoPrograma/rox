use core::fmt;
use std::error::Error;

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
    EQ,
    NEQ,
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
    NoneParserMatched,
    UnrecognizedUnaryOperator,
    UnrecognizedFactorInfixOperator,
    UnrecognizedTermInfixOperator,
    UnrecognizedComparisonInfixOperator,
    UnrecognizedEqualityInfixOperator,
}

impl fmt::Display for ASTParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTParseError::NoTokensLeft => write!(f, "No more tokens left"),
            ASTParseError::PredicateFailed => write!(f, "Predicate failed"),
            ASTParseError::NoneParserMatched => write!(f, "None parser matched"),
            ASTParseError::UnrecognizedUnaryOperator => write!(f, "Unrecognized unary operator"),
            ASTParseError::UnrecognizedFactorInfixOperator => {
                write!(f, "Unrecognized factor infix operator")
            }
            ASTParseError::UnrecognizedTermInfixOperator => {
                write!(f, "Unrecognized term infix operator")
            }
            ASTParseError::UnrecognizedComparisonInfixOperator => {
                write!(f, "Unrecognized comparison infix operator")
            }
            ASTParseError::UnrecognizedEqualityInfixOperator => {
                write!(f, "Unrecognized equality infix operator")
            }
        }
    }
}

impl std::error::Error for ASTParseError {}

type ASTParseResult = Result<(ASTExpression, Vec<Token>), ASTParseError>;

pub trait ASTParser {
    fn parse(&self, input: Vec<Token>) -> ASTParseResult;
}

impl<F> ASTParser for F
where
    F: Fn(Vec<Token>) -> ASTParseResult,
{
    fn parse(&self, input: Vec<Token>) -> ASTParseResult {
        self(input)
    }
}

fn or(ps: Vec<Box<dyn ASTParser>>) -> Box<dyn ASTParser> {
    Box::new(move |state: Vec<Token>| {
        for p in ps.iter() {
            let result = p.parse(state.clone());
            if result.is_ok() {
                return result;
            }
        }

        Err(ASTParseError::NoneParserMatched)
    })
}

// TODO: discriminate lhs and rhs and avoid operate terms with non number
fn equality() -> Box<dyn ASTParser> {
    Box::new(|state: Vec<Token>| {
        let (lhs, state) =
            or(vec![comparison(), term(), factor(), unary(), primary()]).parse(state)?;
        let mut state = state.into_iter();
        let next = state.next().ok_or(ASTParseError::NoTokensLeft)?;
        let operator = match next.kind {
            TokenKind::DoubleEqual => Ok(InfixOperator::EQ),
            TokenKind::BangEqual => Ok(InfixOperator::NEQ),
            _ => Err(ASTParseError::UnrecognizedEqualityInfixOperator),
        }?;

        let (rhs, state) = or(vec![
            equality(),
            comparison(),
            term(),
            factor(),
            unary(),
            primary(),
        ])
        .parse(state.collect())?;

        Ok((
            ASTExpression::Equality(InfixOperation {
                lhs: Box::new(lhs),
                op: operator,
                rhs: Box::new(rhs),
            }),
            state,
        ))
    })
}

// TODO: discriminate lhs and rhs and avoid operate terms with non number
fn comparison() -> Box<dyn ASTParser> {
    Box::new(|state: Vec<Token>| {
        let (lhs, state) = or(vec![term(), factor(), unary(), primary()]).parse(state)?;
        let mut state = state.into_iter();
        let next = state.next().ok_or(ASTParseError::NoTokensLeft)?;
        let operator = match next.kind {
            TokenKind::Greater => Ok(InfixOperator::GT),
            TokenKind::GreaterEqual => Ok(InfixOperator::GTE),
            TokenKind::Less => Ok(InfixOperator::LT),
            TokenKind::LessEqual => Ok(InfixOperator::LTE),
            _ => Err(ASTParseError::UnrecognizedComparisonInfixOperator),
        }?;

        let (rhs, state) =
            or(vec![comparison(), term(), factor(), unary(), primary()]).parse(state.collect())?;

        Ok((
            ASTExpression::Comparison(InfixOperation {
                lhs: Box::new(lhs),
                op: operator,
                rhs: Box::new(rhs),
            }),
            state,
        ))
    })
}

// TODO: discriminate lhs and rhs and avoid operate terms with non number
fn term() -> Box<dyn ASTParser> {
    Box::new(|state: Vec<Token>| {
        let (lhs, state) = or(vec![factor(), unary(), primary()]).parse(state)?;
        let mut state = state.into_iter();
        let next = state.next().ok_or(ASTParseError::NoTokensLeft)?;
        let operator = match next.kind {
            TokenKind::Plus => Ok(InfixOperator::Add),
            TokenKind::Minus => Ok(InfixOperator::Substract),
            _ => Err(ASTParseError::UnrecognizedTermInfixOperator),
        }?;

        let (rhs, state) = or(vec![term(), factor(), unary(), primary()]).parse(state.collect())?;

        Ok((
            ASTExpression::Term(InfixOperation {
                lhs: Box::new(lhs),
                op: operator,
                rhs: Box::new(rhs),
            }),
            state,
        ))
    })
}

// TODO: discriminate lhs and rhs and avoid operate factors with non number
fn factor() -> Box<dyn ASTParser> {
    Box::new(|state: Vec<Token>| {
        let (lhs, state) = or(vec![unary(), primary()]).parse(state)?;
        let mut state = state.into_iter();
        let next = state.next().ok_or(ASTParseError::NoTokensLeft)?;
        let operator = match next.kind {
            TokenKind::Star => Ok(InfixOperator::Multiply),
            TokenKind::Slash => Ok(InfixOperator::Divide),
            _ => Err(ASTParseError::UnrecognizedFactorInfixOperator),
        }?;

        let (rhs, state) = or(vec![factor(), unary(), primary()]).parse(state.collect())?;

        Ok((
            ASTExpression::Factor(InfixOperation {
                lhs: Box::new(lhs),
                op: operator,
                rhs: Box::new(rhs),
            }),
            state,
        ))
    })
}

fn unary() -> Box<dyn ASTParser> {
    Box::new(|state: Vec<Token>| {
        let mut state = state.into_iter();
        let next = state.next().ok_or(ASTParseError::NoTokensLeft)?;
        let operator = match next.kind {
            TokenKind::Minus => Ok(PrefixOperator::NumberNegate),
            TokenKind::Bang => Ok(PrefixOperator::BoolNegate),
            _ => Err(ASTParseError::UnrecognizedUnaryOperator),
        };

        match operator? {
            PrefixOperator::NumberNegate => {
                let (value, state) = or(vec![unary(), primary()]).parse(state.collect())?;
                match value {
                    ASTExpression::Unary(_) | ASTExpression::Primary(Value::Number(_)) => {
                        return Ok((
                            ASTExpression::Unary(PrefixOperation {
                                op: PrefixOperator::NumberNegate,
                                value: Box::new(value),
                            }),
                            state,
                        ));
                    }
                    _ => panic!("cannot define number negate operation with non numeric argument"),
                }
            }
            PrefixOperator::BoolNegate => {
                let (value, state) = or(vec![unary(), primary()]).parse(state.collect())?;
                match value {
                    ASTExpression::Unary(_) | ASTExpression::Primary(Value::Bool(_)) => {
                        return Ok((
                            ASTExpression::Unary(PrefixOperation {
                                op: PrefixOperator::BoolNegate,
                                value: Box::new(value),
                            }),
                            state,
                        ));
                    }
                    _ => panic!("cannot define bool negate operation with non boolean argument"),
                }
            }
            _ => unreachable!(),
        }
    })
}

fn primary() -> Box<dyn ASTParser> {
    Box::new(|state: Vec<Token>| {
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
    })
}

#[cfg(test)]
mod ast_tests {
    use crate::{
        ast::ast::{
            ASTExpression, ASTParser, InfixOperation, InfixOperator, PrefixOperation,
            PrefixOperator, Value, comparison, equality, factor, primary, term, unary,
        },
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

    #[test]
    fn test_unary_for_negate_number() {
        let input = vec![
            Token::new(TokenKind::Minus, "-".to_string()),
            Token::new(TokenKind::Number, "5.5".to_string()),
        ];
        let (expr, state) = unary().parse(input).unwrap();
        let expected_expr = ASTExpression::Unary(PrefixOperation {
            op: PrefixOperator::NumberNegate,
            value: Box::new(ASTExpression::Primary(Value::Number(5.5))),
        });
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_unary_for_nested_negate_numbers() {
        let input = vec![
            Token::new(TokenKind::Minus, "-".to_string()),
            Token::new(TokenKind::Minus, "-".to_string()),
            Token::new(TokenKind::Number, "10".to_string()),
        ];
        let (expr, state) = unary().parse(input).unwrap();
        let expected_expr = ASTExpression::Unary(PrefixOperation {
            op: PrefixOperator::NumberNegate,
            value: Box::new(ASTExpression::Unary(PrefixOperation {
                op: PrefixOperator::NumberNegate,
                value: Box::new(ASTExpression::Primary(Value::Number(10.0))),
            })),
        });
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_unary_for_negate_boolean() {
        let input = vec![
            Token::new(TokenKind::Bang, "!".to_string()),
            Token::new(TokenKind::True, "true".to_string()),
        ];
        let (expr, state) = unary().parse(input).unwrap();
        let expected_expr = ASTExpression::Unary(PrefixOperation {
            op: PrefixOperator::BoolNegate,
            value: Box::new(ASTExpression::Primary(Value::Bool(true))),
        });
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_unary_for_nested_negate_booleans() {
        let input = vec![
            Token::new(TokenKind::Bang, "!".to_string()),
            Token::new(TokenKind::Bang, "!".to_string()),
            Token::new(TokenKind::Bang, "!".to_string()),
            Token::new(TokenKind::True, "true".to_string()),
        ];
        let (expr, state) = unary().parse(input).unwrap();
        let expected_expr = ASTExpression::Unary(PrefixOperation {
            op: PrefixOperator::BoolNegate,
            value: Box::new(ASTExpression::Unary(PrefixOperation {
                op: PrefixOperator::BoolNegate,
                value: Box::new(ASTExpression::Unary(PrefixOperation {
                    op: PrefixOperator::BoolNegate,
                    value: Box::new(ASTExpression::Primary(Value::Bool(true))),
                })),
            })),
        });
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_factor() {
        let input = vec![
            Token::new(TokenKind::Number, "10".to_string()),
            Token::new(TokenKind::Slash, "/".to_string()),
            Token::new(TokenKind::Number, "2".to_string()),
        ];
        let (expr, state) = factor().parse(input).unwrap();
        let expected_expr = ASTExpression::Factor(InfixOperation {
            lhs: Box::new(ASTExpression::Primary(Value::Number(10.0))),
            op: InfixOperator::Divide,
            rhs: Box::new(ASTExpression::Primary(Value::Number(2.0))),
        });
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_nested_factors() {
        let input = vec![
            Token::new(TokenKind::Number, "10".to_string()),
            Token::new(TokenKind::Slash, "/".to_string()),
            Token::new(TokenKind::Number, "2".to_string()),
            Token::new(TokenKind::Star, "*".to_string()),
            Token::new(TokenKind::Number, "7".to_string()),
        ];
        let (expr, state) = factor().parse(input).unwrap();
        let expected_expr = ASTExpression::Factor(InfixOperation {
            lhs: Box::new(ASTExpression::Primary(Value::Number(10.0))),
            op: InfixOperator::Divide,
            rhs: Box::new(ASTExpression::Factor(InfixOperation {
                lhs: Box::new(ASTExpression::Primary(Value::Number(2.0))),
                op: InfixOperator::Multiply,
                rhs: Box::new(ASTExpression::Primary(Value::Number(7.0))),
            })),
        });
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_term() {
        let input = vec![
            Token::new(TokenKind::Number, "8".to_string()),
            Token::new(TokenKind::Plus, "+".to_string()),
            Token::new(TokenKind::Number, "4".to_string()),
        ];
        let (expr, state) = term().parse(input).unwrap();
        let expected_expr = ASTExpression::Term(InfixOperation {
            lhs: Box::new(ASTExpression::Primary(Value::Number(8.0))),
            op: InfixOperator::Add,
            rhs: Box::new(ASTExpression::Primary(Value::Number(4.0))),
        });
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_nested_terms() {
        let input = vec![
            Token::new(TokenKind::Number, "2".to_string()),
            Token::new(TokenKind::Minus, "-".to_string()),
            Token::new(TokenKind::Number, "5".to_string()),
            Token::new(TokenKind::Minus, "-".to_string()),
            Token::new(TokenKind::Number, "9".to_string()),
        ];
        let (expr, state) = term().parse(input).unwrap();
        let expected_expr = ASTExpression::Term(InfixOperation {
            lhs: Box::new(ASTExpression::Primary(Value::Number(2.0))),
            op: InfixOperator::Substract,
            rhs: Box::new(ASTExpression::Term(InfixOperation {
                lhs: Box::new(ASTExpression::Primary(Value::Number(5.0))),
                op: InfixOperator::Substract,
                rhs: Box::new(ASTExpression::Primary(Value::Number(9.0))),
            })),
        });
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_comparison() {
        let input = vec![
            Token::new(TokenKind::Number, "8".to_string()),
            Token::new(TokenKind::Greater, ">".to_string()),
            Token::new(TokenKind::Number, "4".to_string()),
        ];
        let (expr, state) = comparison().parse(input).unwrap();
        let expected_expr = ASTExpression::Comparison(InfixOperation {
            lhs: Box::new(ASTExpression::Primary(Value::Number(8.0))),
            op: InfixOperator::GT,
            rhs: Box::new(ASTExpression::Primary(Value::Number(4.0))),
        });
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_nested_comparisons() {
        let input = vec![
            Token::new(TokenKind::Number, "10".to_string()),
            Token::new(TokenKind::Greater, ">".to_string()),
            Token::new(TokenKind::Number, "5".to_string()),
            Token::new(TokenKind::GreaterEqual, ">=".to_string()),
            Token::new(TokenKind::Number, "15".to_string()),
            Token::new(TokenKind::Less, "<".to_string()),
            Token::new(TokenKind::Number, "8".to_string()),
            Token::new(TokenKind::LessEqual, "<=".to_string()),
            Token::new(TokenKind::Number, "1.05".to_string()),
        ];
        let (expr, state) = comparison().parse(input).unwrap();
        let expected_expr = ASTExpression::Comparison(InfixOperation {
            lhs: Box::new(ASTExpression::Primary(Value::Number(10.0))),
            op: InfixOperator::GT,
            rhs: Box::new(ASTExpression::Comparison(InfixOperation {
                lhs: Box::new(ASTExpression::Primary(Value::Number(5.0))),
                op: InfixOperator::GTE,
                rhs: Box::new(ASTExpression::Comparison(InfixOperation {
                    lhs: Box::new(ASTExpression::Primary(Value::Number(15.0))),
                    op: InfixOperator::LT,
                    rhs: Box::new(ASTExpression::Comparison(InfixOperation {
                        lhs: Box::new(ASTExpression::Primary(Value::Number(8.0))),
                        op: InfixOperator::LTE,
                        rhs: Box::new(ASTExpression::Primary(Value::Number(1.05))),
                    })),
                })),
            })),
        });
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_equality() {
        let input = vec![
            Token::new(TokenKind::Number, "5".to_string()),
            Token::new(TokenKind::DoubleEqual, "==".to_string()),
            Token::new(TokenKind::Number, "2".to_string()),
        ];
        let (expr, state) = equality().parse(input).unwrap();
        let expected_expr = ASTExpression::Equality(InfixOperation {
            lhs: Box::new(ASTExpression::Primary(Value::Number(5.0))),
            op: InfixOperator::EQ,
            rhs: Box::new(ASTExpression::Primary(Value::Number(2.0))),
        });
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }

    #[test]
    fn test_nested_equalities() {
        let input = vec![
            Token::new(TokenKind::Number, "3".to_string()),
            Token::new(TokenKind::BangEqual, "!=".to_string()),
            Token::new(TokenKind::Number, "5".to_string()),
            Token::new(TokenKind::DoubleEqual, "==".to_string()),
            Token::new(TokenKind::Number, "8".to_string()),
        ];
        let (expr, state) = equality().parse(input).unwrap();
        let expected_expr = ASTExpression::Equality(InfixOperation {
            lhs: Box::new(ASTExpression::Primary(Value::Number(3.0))),
            op: InfixOperator::NEQ,
            rhs: Box::new(ASTExpression::Equality(InfixOperation {
                lhs: Box::new(ASTExpression::Primary(Value::Number(5.0))),
                op: InfixOperator::EQ,
                rhs: Box::new(ASTExpression::Primary(Value::Number(8.0))),
            })),
        });
        let expected_state = vec![];

        assert_eq!(expr, expected_expr);
        assert_eq!(state, expected_state);
    }
}
