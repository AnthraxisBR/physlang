use clap::{Parser, Subcommand};
use physlang_core::run_program;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "physlang")]
#[command(about = "PhysLang - A physics-based programming language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a PhysLang program
    Run {
        /// Path to the PhysLang source file
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            match run_file(&file) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn run_file(file: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(file)?;
    let result = run_program(&source)?;

    // Print detector results
    for detector in result.detectors {
        println!("{} = {}", detector.name, detector.value);
    }

    Ok(())
}

