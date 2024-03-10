use std::{error::Error, f64::NEG_INFINITY, fmt::Display, mem::discriminant};

use crate::{lexer::token::Token, parser::ast::{self, LiteralValue}, report_runtime_error};

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: String) -> Self {
        Self { token, message }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let location = if self.token == Token::Eof {
            "at end".to_string()
        } else {
            format!("at '{token}'", token = self.token)
        };
        write!(f, "[line {line}] Error {location}: {message}", line = 0, location = location, message = self.message)
    }
}

impl Error for RuntimeError {}

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self
    }
    fn cast_to_num_nil(&self, literal: ast::LiteralValue) -> ast::LiteralValue {
        match literal {
            ast::LiteralValue::Number(n) => ast::LiteralValue::Number(n),
            ast::LiteralValue::String(s) => if let Ok(num) = s.parse::<f64>() {
                ast::LiteralValue::Number(num)
            } else {
                ast::LiteralValue::Nil
            }
            ast::LiteralValue::Boolean(b) => if b {
                ast::LiteralValue::Number(1.0)
            } else {
                ast::LiteralValue::Number(0.0)
            },
            ast::LiteralValue::Nil => ast::LiteralValue::Nil
        }
    }
    fn cast_to_num(&self, literal: ast::LiteralValue) -> f64 {
        let value = self.cast_to_num_nil(literal);
        if let ast::LiteralValue::Number(n) = value {
            n
        } else {
            NEG_INFINITY
        }
    }
    fn cast_to_bool(&self, literal: ast::LiteralValue) -> bool {
        match literal {
            ast::LiteralValue::Number(n) => if n == 0.0 {
                false
            } else {
                true
            },
            ast::LiteralValue::String(s) => if s.is_empty() {
                false
            } else {
                true
            },
            ast::LiteralValue::Boolean(b) => b,
            ast::LiteralValue::Nil => false
        }
    }
    fn cast_to_string(&self, literal: ast::LiteralValue) -> String {
        match literal {
            ast::LiteralValue::Number(n) => n.to_string(),
            ast::LiteralValue::String(s) => s,
            ast::LiteralValue::Boolean(b) => b.to_string(),
            ast::LiteralValue::Nil => "nil".to_string()
        }
    }
    pub fn interpret(&mut self, expression: ast::Expression) -> ast::LiteralValue {
        match self.evaluate(expression) {
            Ok(value) => value,
            Err(error) => {
                report_runtime_error(error);

                ast::LiteralValue::Nil
            }
        }
    }
    fn evaluate(&mut self, expression: ast::Expression) -> Result<ast::LiteralValue, RuntimeError> {
        Ok(match expression {
            ast::Expression::Literal(literal) => literal,
            ast::Expression::Grouping(expr) => self.evaluate(*expr)?,
            ast::Expression::Unary { operator, right } => self.evaluate_unary(operator, *right)?,
            ast::Expression::Binary { left, operator, right } => self.evaluate_binary(*left, operator, *right)?,
        })
    }
    fn evaluate_unary(&mut self, operator: Token, right: ast::Expression) -> Result<ast::LiteralValue, RuntimeError> {
        let value = self.evaluate(right)?;
        Ok(match operator {
            Token::Minus => {
                if let ast::LiteralValue::Number(num) = self.cast_to_num_nil(value) {
                    ast::LiteralValue::Number(-num)
                } else {
                    ast::LiteralValue::Nil
                }
            }
            Token::Bang => {
                ast::LiteralValue::Boolean(!self.cast_to_bool(value))
            }
            _ => ast::LiteralValue::Nil
        })
    }
    fn evaluate_binary(&mut self, left: ast::Expression, operator: Token, right: ast::Expression) -> Result<ast::LiteralValue, RuntimeError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        let dummy_string = ast::LiteralValue::String("".to_string());
        Ok(match operator {
            Token::Minus => {
                if let (ast::LiteralValue::Number(left), ast::LiteralValue::Number(right)) = (self.cast_to_num_nil(left), self.cast_to_num_nil(right)) {
                    ast::LiteralValue::Number(left - right)
                } else {
                    ast::LiteralValue::Nil
                }
            }
            Token::Plus if
                discriminant(&left) == discriminant(&dummy_string) ||
                discriminant(&right) == discriminant(&dummy_string) => {
                ast::LiteralValue::String(format!("{}{}", self.cast_to_string(left), self.cast_to_string(right)))
            }
            Token::Plus => {
                if let (ast::LiteralValue::Number(left), ast::LiteralValue::Number(right)) = (self.cast_to_num_nil(left), self.cast_to_num_nil(right)) {
                    ast::LiteralValue::Number(left + right)
                } else {
                    ast::LiteralValue::Nil
                }
            }
            Token::Slash => {
                if let (ast::LiteralValue::Number(left), ast::LiteralValue::Number(right)) = (self.cast_to_num_nil(left), self.cast_to_num_nil(right)) {
                    ast::LiteralValue::Number(left / right)
                } else {
                    ast::LiteralValue::Nil
                }
            }
            Token::Star => {
                if let (ast::LiteralValue::Number(left), ast::LiteralValue::Number(right)) = (self.cast_to_num_nil(left), self.cast_to_num_nil(right)) {
                    ast::LiteralValue::Number(left * right)
                } else {
                    ast::LiteralValue::Nil
                }
            }
            Token::Greater => ast::LiteralValue::Boolean(self.cast_to_num(left) > self.cast_to_num(right)),
            Token::GreaterEqual => ast::LiteralValue::Boolean(self.cast_to_num(left) >= self.cast_to_num(right)),
            Token::Less => ast::LiteralValue::Boolean(self.cast_to_num(left) < self.cast_to_num(right)),
            Token::LessEqual => ast::LiteralValue::Boolean(self.cast_to_num(left) <= self.cast_to_num(right)),
            Token::EqualEqual => ast::LiteralValue::Boolean(left == right),
            Token::BangEqual => ast::LiteralValue::Boolean(left != right),

            _ => ast::LiteralValue::Nil
        })
    }
}