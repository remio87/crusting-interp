use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

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
    dbg!(&contents);
    run(&contents);
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
    }
}

fn run(code: &str) {
    println!("{}", code);
}
