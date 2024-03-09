use std::fmt;

use crate::lexer::token::Token;

#[derive(Debug)]
pub enum Expression {
    Literal(LiteralValue),
    Unary {
        operator: Token,
        right: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Grouping(Box<Expression>),
}

#[derive(Debug)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralValue::Number(value) => write!(f, "{}", value),
            LiteralValue::String(value) => write!(f, "{}", value),
            LiteralValue::Boolean(value) => write!(f, "{}", value),
            LiteralValue::Nil => write!(f, "NIL"),
        }
    }

}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(value) => write!(f, "{value}"),
            Expression::Unary { operator, right } => write!(f, "({operator} {right})"),
            Expression::Binary { left, operator, right } => write!(f, "({left} {operator} {right})"),
            Expression::Grouping(expr) => write!(f, "({expr})"),
        }
    }
}
