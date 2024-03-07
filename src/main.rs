use std::{fs, io::Write, path::PathBuf};

use clap::Parser;
use lexer::Scanner;

mod lexer;

#[derive(Parser, Debug)]
struct Cli {
    pub file: Option<PathBuf>,
}

fn run(code: String) {
    let mut scanner = Scanner::new(code);
    let tokens = scanner.scan_tokens();

    for token in tokens.iter() {
        dbg!(token);
    }
}

fn report_error(line: usize, message: &str) {
    report(line, "", message);
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
