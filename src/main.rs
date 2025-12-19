use clap::{Parser, Subcommand};
use rlox::scanner::Scanner;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "lox")]
#[command(version, about= "A lox interpreter", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Run a .lox file or start a REPL if no file is given
    Run {
        /// Path to the .lox file
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { path } => {
            if let Some(path) = path {
                run_file(&path)
            } else {
                run_prompt()
            }
        }
    }
}

pub fn run_file(path: &Path) {
    let source = std::fs::read_to_string(path).unwrap();
    run(&source);
}

pub fn run_prompt() {
    println!("Lox REPL. Press Ctrl+D to exit.");
    let stdin = io::stdin();
    loop {
        print!("lox> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        let bytes_read = stdin.read_line(&mut line).unwrap();

        if bytes_read == 0 {
            println!();
            break;
        }

        let src = line.trim();
        if src.is_empty() {
            continue;
        }

        run(src);
    }
}

pub fn run(src: &str) {
    let scanner = Scanner::new();
    let tokens = scanner.scan_tokens(src);

    for token in tokens {
        println!("{:?}", token);
    }
}
