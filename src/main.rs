use clap::{Parser, Subcommand};
use rlox::run;
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

    lex {
        #[arg(value_name = "PATH")]
        path: PathBuf,
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
        Commands::lex { path } => {
            let source = std::fs::read_to_string(path).unwrap();
            Scanner::new(&source).for_each(|token| match token {
                Ok(token) => println!("{:?}", token),
                Err(err) => println!("{}", err),
            });
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
