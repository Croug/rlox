use std::{fs, io::Write, path::PathBuf};

use clap::Parser;
use lexer::{token::Token, Scanner};

mod lexer;
mod parser;

#[derive(Parser, Debug)]
struct Cli {
    pub file: Option<PathBuf>,
}

fn run(code: String) {
    let mut scanner = Scanner::new(code);
    let tokens = scanner.scan_tokens();
    let mut parser = parser::Parser::new(tokens);
    _ = dbg!(parser.parse());
}

fn report_lex_error(line: usize, message: &str) {
    report(line, "", message);
}

fn report_parse_error(token: Token, message: &str) {
    if token == Token::Eof {
        report(0, "at end", message)
    } else {
        report(0, format!("at '{token}'").as_str(), message)
    }
}

fn report(line: usize, location: &str, message: &str) {
    eprintln!("[line {line}] Error {location}: {message}");
}

fn run_file(file: PathBuf) {
    run(fs::read_to_string(file).expect("Failed to read file"));
}

fn run_prompt() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    loop {
        let mut input = String::new();
        print!("> ");
        stdout.flush().unwrap();
        stdin.read_line(&mut input).unwrap();
        run(input);
    }
}

fn main() {
    let args = Cli::parse();
    match args.file {
        Some(file) => run_file(file),
        None => run_prompt(),
    }
}
