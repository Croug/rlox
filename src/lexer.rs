use crate::report_error;

pub mod token;
use token::Token;

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
    
        tokens.push(Token::Eof);
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

    fn scan_string(&mut self) -> Option<Token> {
        while self.more_tokens() && self.peek().unwrap() != '"' {
            if self.peek().unwrap() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if !self.more_tokens() {
            report_error(self.line, "Unterminated string");
            return None;
        }

        self.advance();

        Some(Token::String(self.source[self.start + 1..self.current - 1].to_string()))
    }

    fn scan_int(&mut self) {
        while self.more_tokens() && self.peek().unwrap().is_digit(10) {
            self.advance();
        }
    }

    fn scan_number(&mut self) -> Token {
        self.scan_int();

        if self.current + 1 < self.source.len() && self.peek().unwrap() == '.' && self.peek_next().unwrap().is_digit(10) {
            self.advance();
            self.scan_int();
        }

        Token::Number(self.source[self.start..self.current].parse().unwrap())
    }

    fn scan_identifier(&mut self) -> Token {
        while self.more_tokens() && (self.peek().unwrap().is_alphanumeric() || self.peek().unwrap() == '_') {
            self.advance();
        }

        let text = &self.source[self.start..self.current];

        match text {
            "and" => Token::And,
            "class" => Token::Class,
            "else" => Token::Else,
            "false" => Token::False,
            "for" => Token::For,
            "fun" => Token::Fun,
            "if" => Token::If,
            "nil" => Token::Nil,
            "or" => Token::Or,
            "print" => Token::Print,
            "return" => Token::Return,
            "super" => Token::Super,
            "this" => Token::This,
            "true" => Token::True,
            "var" => Token::Var,
            "while" => Token::While,

            text => Token::Identifier(text.to_string()),
        }
    }

    fn scan_token(&mut self) -> Option<Token> {
        return Some(match self.advance()? {
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            ',' => Token::Comma,
            '.' => Token::Dot,
            '-' => Token::Minus,
            '+' => Token::Plus,
            ';' => Token::SemiColon,
            '*' => Token::Star,

            '!' if self.match_next('=') => Token::BangEqual,
            '!' => Token::Bang,
            '=' if self.match_next('=') => Token::EqualEqual,
            '=' => Token::Equal,
            '<' if self.match_next('=') => Token::LessEqual,
            '<' => Token::Less,
            '>' if self.match_next('=') => Token::GreaterEqual,
            '>' => Token::Greater,

            '"' => self.scan_string()?,

            c if c.is_digit(10) => self.scan_number(),
            c if c.is_alphabetic() || c == '_' => self.scan_identifier(),

            '/' if self.match_next('/') => {
                self.consume_line();
                return None;
            }
            '/' => Token::Slash,

            ' ' | '\r' | '\t' => return None,

            '\n' => {
                self.line += 1;
                return None;
            }

            c => {
                report_error(self.line, format!("Unexpected character: {c}").as_str());
                return None;
            }
        }) 
    }

    fn more_tokens(&self) -> bool {
        self.current < self.source.len()
    }
}