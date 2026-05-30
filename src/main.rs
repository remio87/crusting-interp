mod scanner;
mod token;
mod token_type;

use scanner::Scanner;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use std::sync::atomic::AtomicBool;

static HAD_ERROR: AtomicBool = AtomicBool::new(false);

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
    run(&contents);
    if HAD_ERROR.load(std::sync::atomic::Ordering::Relaxed) {
        exit(65);
    }
}

fn run_prompt() {
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
        run(buf);
        HAD_ERROR.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

fn run(code: &str) {
    let mut scanner = Scanner::new(code);
    scanner.scan_tokens();
    let tokens = scanner.tokens;
    let tokens = tokens
        .into_iter()
        .map(|tok| format!("{}", tok))
        .collect::<Vec<_>>()
        .join(", ");
    println!("{}", tokens);
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn report(line: usize, place: &str, message: &str) {
    eprintln!("[line {}] Error {}: {}", line, place, message);
    HAD_ERROR.store(true, std::sync::atomic::Ordering::Relaxed);
}
