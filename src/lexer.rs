use crate::report_lex_error;

pub mod token;
use token::TokenType;

use self::token::Token;

pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self{
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        while self.more_tokens() {
            self.start = self.current;
            if let Some(token) = self.scan_token() {
                tokens.push(token)
            }
        }
    
        tokens.push(Token::new(TokenType::Eof, self.line));
        tokens
    }

    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.source.chars().nth(self.current - 1)
    }

    fn match_next(&mut self, match_char: char) -> bool {
        if self.more_tokens() && self.source.chars().nth(self.current).unwrap() == match_char {
            self.current += 1;

            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.current)
    }

    fn peek_next(&self) -> Option<char> {
        self.source.chars().nth(self.current + 1)
    }

    fn consume_line(&mut self) {
        while self.more_tokens() && self.peek().unwrap() != '\n' {
            self.advance();
        }
    }

    fn scan_string(&mut self) -> Option<TokenType> {
        while self.more_tokens() && self.peek().unwrap() != '"' {
            if self.peek().unwrap() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if !self.more_tokens() {
            report_lex_error(self.line, "Unterminated string");
            return None;
        }

        self.advance();

        Some(TokenType::String(self.source[self.start + 1..self.current - 1].to_string()))
    }

    fn scan_int(&mut self) {
        while self.more_tokens() && self.peek().unwrap().is_digit(10) {
            self.advance();
        }
    }

    fn scan_number(&mut self) -> TokenType {
        self.scan_int();

        if self.current + 1 < self.source.len() && self.peek().unwrap() == '.' && self.peek_next().unwrap().is_digit(10) {
            self.advance();
            self.scan_int();
        }

        TokenType::Number(self.source[self.start..self.current].parse().unwrap())
    }

    fn scan_identifier(&mut self) -> TokenType {
        while self.more_tokens() && (self.peek().unwrap().is_alphanumeric() || self.peek().unwrap() == '_') {
            self.advance();
        }

        let text = &self.source[self.start..self.current];

        match text.to_lowercase().as_str() {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,

            text => TokenType::Identifier(text.to_string()),
        }
    }

    fn scan_token(&mut self) -> Option<Token> {
        let line = self.line;
        return Some(Token::new(match self.advance()? {
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            '-' => TokenType::Minus,
            '+' => TokenType::Plus,
            ';' => TokenType::SemiColon,
            '*' => TokenType::Star,

            '!' if self.match_next('=') => TokenType::BangEqual,
            '!' => TokenType::Bang,
            '=' if self.match_next('=') => TokenType::EqualEqual,
            '=' => TokenType::Equal,
            '<' if self.match_next('=') => TokenType::LessEqual,
            '<' => TokenType::Less,
            '>' if self.match_next('=') => TokenType::GreaterEqual,
            '>' => TokenType::Greater,

            '"' => self.scan_string()?,

            c if c.is_digit(10) => self.scan_number(),
            c if c.is_alphabetic() || c == '_' => self.scan_identifier(),

            '/' if self.match_next('/') => {
                self.consume_line();
                return None;
            }
            '/' => TokenType::Slash,

            ' ' | '\r' | '\t' => return None,

            '\n' => {
                self.line += 1;
                return None;
            }

            c => {
                report_lex_error(self.line, format!("Unexpected character: {c}").as_str());
                return None;
            }
        }, line)) 
    }

    fn more_tokens(&self) -> bool {
        self.current < self.source.len()
    }
}