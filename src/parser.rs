use std::{mem::discriminant, vec};

use crate::{
    lexer::token::{Token, TokenType},
    report, report_lex_error, report_parse_error,
};

use self::ast::{Expression, Statement};

pub mod ast;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = vec![];

        while self.more_tokens() {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(_) => self.synchronize(),
            }
        }

        Ok(statements)
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.current)
            .expect("Read header beyond end of file")
    }

    fn previous(&self) -> &Token {
        self.tokens
            .get(self.current - 1)
            .expect("Read header at beginning of file")
    }

    fn advance(&mut self) -> &Token {
        if self.more_tokens() {
            self.current += 1;
        }
        self.previous()
    }

    fn consume(&mut self, token: TokenType, message: &str) -> Result<Token, String> {
        if self.check(token.clone()) {
            Ok(self.advance().clone())
        } else {
            Self::error(Token::new(token, self.peek().line), message)
        }
    }

    fn error<T>(token: Token, message: &str) -> Result<T, String> {
        report_parse_error(token, message);

        Err(message.to_string())
    }

    fn synchronize(&mut self) {
        while self.more_tokens() {
            if self.previous().token_type == TokenType::SemiColon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn check(&self, token: TokenType) -> bool {
        self.more_tokens() && discriminant(&token) == discriminant(&self.peek().token_type)
    }

    fn match_tokens(&mut self, tokens: Vec<TokenType>) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn more_tokens(&self) -> bool {
        self.peek().token_type != TokenType::Eof
    }

    fn declaration(&mut self) -> Result<Statement, String> {
        if self.match_tokens(vec![TokenType::Var]) {
            self.varDeclaration()
        } else {
            self.statement()
        }
    }

    fn varDeclaration(&mut self) -> Result<Statement, String> {
        let identifier = self.consume(TokenType::Identifier("".to_string()), "Expect variable name")?;
        let mut initializer = None;

        if self.match_tokens(vec![TokenType::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(TokenType::SemiColon, "Expect ';' after variable declaration")?;

        Ok(Statement::Var { identifier, initializer })
    }

    fn statement(&mut self) -> Result<Statement, String> {
        if self.match_tokens(vec![TokenType::Print]) {
            self.printStatement()
        } else {
            self.expressionStatement()
        }
    }

    fn expressionStatement(&mut self) -> Result<Statement, String> {
        let expr = self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after expression")?;

        Ok(Statement::Expression(expr))
    }

    fn printStatement(&mut self) -> Result<Statement, String> {
        let expr = self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after value")?;

        Ok(Statement::Print(expr))
    }

    fn expression(&mut self) -> Result<Expression, String> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expression, String> {
        let mut expr = self.comparison()?;
        while self.match_tokens(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone().token_type;
            let right = self.comparison()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.term()?;

        while self.match_tokens(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone().token_type;
            let right = self.term()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression, String> {
        let mut expr = self.factor()?;

        while (self.match_tokens(vec![TokenType::Minus, TokenType::Plus])) {
            let operator = self.previous().clone().token_type;
            let right = self.factor()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression, String> {
        let mut expr = self.unary()?;

        while self.match_tokens(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone().token_type;
            let right = self.unary()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, String> {
        if self.match_tokens(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone().token_type;
            let right = self.unary()?;

            Ok(Expression::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expression, String> {
        let ret = match self.peek().clone().token_type {
            TokenType::False => Expression::Literal(ast::LiteralValue::Boolean(false)),
            TokenType::True => Expression::Literal(ast::LiteralValue::Boolean(true)),
            TokenType::Nil => Expression::Literal(ast::LiteralValue::Nil),
            TokenType::Number(value) => Expression::Literal(ast::LiteralValue::Number(value)),
            TokenType::String(value) => {
                Expression::Literal(ast::LiteralValue::String(value.clone()))
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression")?;

                return Ok(Expression::Grouping(Box::new(expr)));
            }
            TokenType::Identifier(_) => Expression::Variable(self.peek().clone()),
            _ => return Self::error(self.peek().clone(), "Expect expression"),
        };

        self.advance();

        Ok(ret)
    }
}
