use std::mem::discriminant;

use crate::{lexer::token::Token, report, report_lex_error, report_parse_error};

use self::ast::Expression;

mod ast;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Expression, String> {
        self.expression()
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).expect("Read header beyond end of file")
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).expect("Read header at beginning of file")
    }

    fn advance(&mut self) -> &Token {
        if self.more_tokens() {
            self.current += 1;
        }
        self.previous()
    }

    fn consume(&mut self, token: Token, message: &str) -> Result<Token, String> {
        if self.check(token.clone()) {
            Ok(self.advance().clone())
        } else {
            Self::error(token, message)
        }
    }

    fn error<T>(token: Token, message: &str) -> Result<T, String> {
        report_parse_error(token, message);
    
        Err(message.to_string())
    }

    fn synchronize(&mut self) {
        while self.more_tokens() {
            if *self.previous() == Token::SemiColon {
                return;
            }

            match self.peek() {
                Token::Class | Token::Fun | Token::Var | Token::For | Token::If | Token::While | Token::Print | Token::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn check(&self, token: Token) -> bool {
        self.more_tokens() && discriminant(&token) == discriminant(self.peek())
    }

    fn match_tokens(&mut self, tokens: Vec<Token>) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn more_tokens(&self) -> bool {
        *self.peek() != Token::Eof
    }

    fn expression(&mut self) -> Result<Expression, String> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expression, String> {
        let mut expr = self.comparison()?;
        while self.match_tokens(vec![Token::BangEqual, Token::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.term()?;

        while self.match_tokens(vec![Token::Greater, Token::GreaterEqual, Token::Less, Token::LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression, String> {
        let mut expr = self.factor()?;

        while(self.match_tokens(vec![Token::Minus, Token::Plus])) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression, String> {
        let mut expr = self.unary()?;

        while self.match_tokens(vec![Token::Slash, Token::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, String> {
        if self.match_tokens(vec![Token::Bang, Token::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            Ok(Expression::Unary{
                operator,
                right: Box::new(right)
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expression, String> {
        let ret = match self.peek() {
            Token::False => Expression::Literal(ast::LiteralValue::Boolean(false)),
            Token::True => Expression::Literal(ast::LiteralValue::Boolean(true)),
            Token::Nil => Expression::Literal(ast::LiteralValue::Nil),
            Token::Number(value) => Expression::Literal(ast::LiteralValue::Number(*value)),
            Token::String(value) => Expression::Literal(ast::LiteralValue::String(value.clone())),
            Token::LeftParen => {
                let expr = self.expression()?;
                self.consume(Token::RightParen, "Expect ')' after expression")?;

                return Ok(Expression::Grouping(Box::new(expr)));
            }
            _=> return Self::error(self.peek().clone(), "Expect expression")
        };

        self.advance();

        Ok(ret)
    }
}