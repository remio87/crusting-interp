mod class;
mod environment;
mod expr;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod token;
mod token_type;
mod value;

use interpreter::Interpreter;
use scanner::Scanner;
use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::exit;

use crate::parser::Parser;
use crate::resolver::Resolver;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: rlox: [script]");
    } else if args.len() == 2 {
        run_file(Path::new(&args[1]));
    } else {
        run_prompt();
    }
}

fn run_file(path: &Path) {
    let contents = fs::read_to_string(path).expect("Should have been able to read the file");
    let mut interpreter = Interpreter::new();
    if let Err(e) = run(&mut interpreter, &contents) {
        eprintln!("{}", e);
        exit(65);
    }
}

fn run_prompt() {
    let mut interpreter = Interpreter::new();
    loop {
        let mut buf = String::new();
        print!("> ");
        std::io::stdout().flush().expect("failed to flush stdout");
        std::io::stdin()
            .read_line(&mut buf)
            .expect("failed to read stdin");
        let buf = buf.trim();
        if buf.is_empty() {
            break;
        }
        if let Err(e) = run(&mut interpreter, buf) {
            eprintln!("{}", e)
        }
    }
}

fn run(interpreter: &mut Interpreter, code: &str) -> Result<(), Box<dyn Error>> {
    let mut scanner = Scanner::new(code);
    scanner.scan_tokens();
    let tokens = scanner.into_tokens()?;
    let parser = Parser::new(tokens);
    let statements = parser.parse().map_err(|errors| {
        errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    Resolver::new(interpreter)
        .resolve(&statements)
        .map_err(|errors| {
            errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        })?;
    interpreter.interpret(statements)?;
    Ok(())
}
