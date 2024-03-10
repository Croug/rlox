use std::{error::Error, f64::NEG_INFINITY, fmt::Display, mem::discriminant};

use crate::{
    lexer::token::{Token, TokenType},
    parser::ast::{self, LiteralValue, Statement},
    report_runtime_error,
};

use self::memory::Memory;

pub mod memory;

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
        write!(
            f,
            "[line {line}] Error at '{token}': {message}",
            line = 0,
            token = self.token.token_type,
            message = self.message
        )
    }
}

impl Error for RuntimeError {}

pub struct Interpreter {
    memory: Memory
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            memory: Memory::new()
        }
    }
    fn cast_to_num_nil(&self, literal: ast::LiteralValue) -> ast::LiteralValue {
        match literal {
            ast::LiteralValue::Number(n) => ast::LiteralValue::Number(n),
            ast::LiteralValue::String(s) => {
                if let Ok(num) = s.parse::<f64>() {
                    ast::LiteralValue::Number(num)
                } else {
                    ast::LiteralValue::Nil
                }
            }
            ast::LiteralValue::Boolean(b) => {
                if b {
                    ast::LiteralValue::Number(1.0)
                } else {
                    ast::LiteralValue::Number(0.0)
                }
            }
            ast::LiteralValue::Nil => ast::LiteralValue::Nil,
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
            ast::LiteralValue::Number(n) => {
                if n == 0.0 {
                    false
                } else {
                    true
                }
            }
            ast::LiteralValue::String(s) => {
                if s.is_empty() {
                    false
                } else {
                    true
                }
            }
            ast::LiteralValue::Boolean(b) => b,
            ast::LiteralValue::Nil => false,
        }
    }
    fn cast_to_string(&self, literal: ast::LiteralValue) -> String {
        match literal {
            ast::LiteralValue::Number(n) => n.to_string(),
            ast::LiteralValue::String(s) => s,
            ast::LiteralValue::Boolean(b) => b.to_string(),
            ast::LiteralValue::Nil => "nil".to_string(),
        }
    }
    pub fn interpret(&mut self, statements: Vec<ast::Statement>) {
        for statement in statements {
            if let Err(error) = self.evaluate_statement(statement) {
                report_runtime_error(error);
                return;
            }
        }
    }
    fn evaluate_statement(&mut self, statement: ast::Statement) -> Result<(), RuntimeError> {
        match statement {
            Statement::Expression(expr) => {
                self.evaluate_expression(expr)?;
            }
            Statement::Print(expr) => {
                let value = self.evaluate_expression(expr)?;
                println!("{value}");
            }
            Statement::Var { identifier, initializer } => {
                if let Some(initializer) = initializer {
                    let value = self.evaluate_expression(initializer)?;
                    self.memory.set(identifier, value);
                }
            }
        }

        Ok(())
    }
    fn evaluate_expression(&mut self, expression: ast::Expression) -> Result<ast::LiteralValue, RuntimeError> {
        Ok(match expression {
            ast::Expression::Literal(literal) => literal,
            ast::Expression::Grouping(expr) => self.evaluate_expression(*expr)?,
            ast::Expression::Unary { operator, right } => self.evaluate_unary(operator, *right)?,
            ast::Expression::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary(*left, operator, *right)?,
            ast::Expression::Variable(token) => self.memory.get(token)?,
        })
    }
    fn evaluate_unary(
        &mut self,
        operator: TokenType,
        right: ast::Expression,
    ) -> Result<ast::LiteralValue, RuntimeError> {
        let value = self.evaluate_expression(right)?;
        Ok(match operator {
            TokenType::Minus => {
                if let ast::LiteralValue::Number(num) = self.cast_to_num_nil(value) {
                    ast::LiteralValue::Number(-num)
                } else {
                    ast::LiteralValue::Nil
                }
            }
            TokenType::Bang => ast::LiteralValue::Boolean(!self.cast_to_bool(value)),
            _ => ast::LiteralValue::Nil,
        })
    }
    fn evaluate_binary(
        &mut self,
        left: ast::Expression,
        operator: TokenType,
        right: ast::Expression,
    ) -> Result<ast::LiteralValue, RuntimeError> {
        let left = self.evaluate_expression(left)?;
        let right = self.evaluate_expression(right)?;
        let dummy_string = ast::LiteralValue::String("".to_string());
        Ok(match operator {
            TokenType::Minus => {
                if let (ast::LiteralValue::Number(left), ast::LiteralValue::Number(right)) =
                    (self.cast_to_num_nil(left), self.cast_to_num_nil(right))
                {
                    ast::LiteralValue::Number(left - right)
                } else {
                    ast::LiteralValue::Nil
                }
            }
            TokenType::Plus
                if discriminant(&left) == discriminant(&dummy_string)
                    || discriminant(&right) == discriminant(&dummy_string) =>
            {
                ast::LiteralValue::String(format!(
                    "{}{}",
                    self.cast_to_string(left),
                    self.cast_to_string(right)
                ))
            }
            TokenType::Plus => {
                if let (ast::LiteralValue::Number(left), ast::LiteralValue::Number(right)) =
                    (self.cast_to_num_nil(left), self.cast_to_num_nil(right))
                {
                    ast::LiteralValue::Number(left + right)
                } else {
                    ast::LiteralValue::Nil
                }
            }
            TokenType::Slash => {
                if let (ast::LiteralValue::Number(left), ast::LiteralValue::Number(right)) =
                    (self.cast_to_num_nil(left), self.cast_to_num_nil(right))
                {
                    ast::LiteralValue::Number(left / right)
                } else {
                    ast::LiteralValue::Nil
                }
            }
            TokenType::Star => {
                if let (ast::LiteralValue::Number(left), ast::LiteralValue::Number(right)) =
                    (self.cast_to_num_nil(left), self.cast_to_num_nil(right))
                {
                    ast::LiteralValue::Number(left * right)
                } else {
                    ast::LiteralValue::Nil
                }
            }
            TokenType::Greater => {
                ast::LiteralValue::Boolean(self.cast_to_num(left) > self.cast_to_num(right))
            }
            TokenType::GreaterEqual => {
                ast::LiteralValue::Boolean(self.cast_to_num(left) >= self.cast_to_num(right))
            }
            TokenType::Less => {
                ast::LiteralValue::Boolean(self.cast_to_num(left) < self.cast_to_num(right))
            }
            TokenType::LessEqual => {
                ast::LiteralValue::Boolean(self.cast_to_num(left) <= self.cast_to_num(right))
            }
            TokenType::EqualEqual => ast::LiteralValue::Boolean(left == right),
            TokenType::BangEqual => ast::LiteralValue::Boolean(left != right),

            _ => ast::LiteralValue::Nil,
        })
    }
}
