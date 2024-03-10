use std::{error::Error, fs, io::Write, path::PathBuf};

use clap::Parser;
use interpreter::Interpreter;
use lexer::{
    token::{Token, TokenType},
    Scanner,
};

mod interpreter;
mod lexer;
mod parser;

#[derive(Parser, Debug)]
struct Cli {
    pub file: Option<PathBuf>,
}

fn run(code: String, interpreter: &mut Interpreter) -> Result<(), Box<dyn Error>> {
    let mut scanner = Scanner::new(code);
    let tokens = scanner.scan_tokens();
    let mut parser = parser::Parser::new(tokens);
    let asts = parser.parse()?;
    _ = interpreter.interpret(asts);

    Ok(())
}

fn report_lex_error(line: usize, message: &str) {
    report(line, "", message);
}

fn report_parse_error(token: Token, message: &str) {
    if token.token_type == TokenType::Eof {
        report(token.line, "at end", message)
    } else {
        report(
            token.line,
            format!("at '{}'", token.token_type).as_str(),
            message,
        )
    }
}

fn report_runtime_error(error: interpreter::RuntimeError) {
    report(
        error.token.line,
        format!("at '{}'", error.token.token_type).as_str(),
        &error.message,
    )
}

fn report(line: usize, location: &str, message: &str) {
    eprintln!("[line {line}] Error {location}: {message}");
}

fn run_file(file: PathBuf) {
    _ = run(
        fs::read_to_string(file).expect("Failed to read file"),
        &mut Interpreter::new(),
    );
}

fn run_prompt() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut interpreter = Interpreter::new();
    loop {
        let mut input = String::new();
        print!("> ");
        stdout.flush().unwrap();
        stdin.read_line(&mut input).unwrap();
        _ = run(input, &mut interpreter)
    }
}

fn main() {
    let args = Cli::parse();
    match args.file {
        Some(file) => run_file(file),
        None => run_prompt(),
    }
}
