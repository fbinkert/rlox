use clap::{Parser, Subcommand};
use owo_colors::{OwoColorize, Style};
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

    Lex {
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { path } => {
            if let Some(path) = path {
                run_file(&path)?;
            } else {
                run_prompt();
            }
        }
        Commands::Lex { path } => {
            let source = std::fs::read_to_string(path).unwrap();

            Scanner::new(&source).for_each(|token| {
                println!("{token:?}");
            });
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error if scanning/parsing fails
/// # Panics
/// If the contents of the file are not valid UTF-8
pub fn run_file(path: &Path) -> miette::Result<()> {
    let source = std::fs::read_to_string(path).unwrap();
    run(&source)
}

/// # Panics
/// It is considered an error if not all bytes could be written due to
/// I/O errors or EOF being reached.
pub fn run_prompt() {
    println!("Lox REPL. Press Ctrl+D to exit.");
    let stdin = io::stdin();

    loop {
        print!("{} ", "lox>".bright_cyan());
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

        match run(src) {
            Err(report) => {
                eprintln!("{} {:?}", "err>".bright_red().bold(), report); //{:?} triggers miette report handler
            }
            _ => (),
        }
    }
}

fn print_block(prefix: &str, s: &str) {
    let style = Style::new().bright_green().bold();
    let styled_prefix = prefix.style(style);

    let pad = " ".repeat(prefix.len());
    for (i, line) in s.lines().enumerate() {
        if i == 0 {
            println!("{styled_prefix} {line}");
        } else {
            println!("{pad} {line}");
        }
    }
}
