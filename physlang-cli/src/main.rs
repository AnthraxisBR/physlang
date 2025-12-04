use clap::{Parser, Subcommand};
use physlang_core::{
    analyze_program, parse_program, run_program, Diagnostic, DiagnosticSeverity,
};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "physlang")]
#[command(about = "PhysLang - A physics-based programming language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run a PhysLang program
    Run {
        /// Path to the PhysLang source file
        file: PathBuf,
    },
    /// Check a PhysLang program for errors without running it
    Check {
        /// Path to the PhysLang source file
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Command::Run { file } => {
            match run_file(&file) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    1
                }
            }
        }
        Command::Check { file } => {
            match check_file(&file) {
                Ok(has_errors) => {
                    if has_errors {
                        1
                    } else {
                        0
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    1
                }
            }
        }
    };

    std::process::exit(exit_code);
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

fn check_file(file: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
    let source = fs::read_to_string(file)?;

    // Parse the program
    let program = match parse_program(&source) {
        Ok(program) => program,
        Err(parse_error) => {
            // Convert parse error to diagnostic and print
            let diagnostic = Diagnostic::error(
                format!("{}", parse_error),
                parse_error.span(),
            );
            print_diagnostics(&source, &[diagnostic]);
            return Ok(true); // Has errors
        }
    };

    // Analyze the program
    let diagnostics = analyze_program(&program);

    if diagnostics.is_empty() {
        println!("No issues found.");
        return Ok(false); // No errors
    }

    // Print diagnostics
    let diagnostics_vec: Vec<Diagnostic> = diagnostics.iter().cloned().collect();
    print_diagnostics(&source, &diagnostics_vec);

    // Return true if there are any errors
    Ok(diagnostics.has_errors())
}

/// Print diagnostics with source location information
fn print_diagnostics(source: &str, diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        let severity_str = match diagnostic.severity {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
        };

        print!("{}: {}", severity_str, diagnostic.message);

        if let Some(location) = diagnostic.location(source) {
            println!(" at line {}, column {}", location.line, location.column);

            // Try to show the line with a caret
            let lines: Vec<&str> = source.lines().collect();
            if location.line > 0 && location.line <= lines.len() {
                let line_content = lines[location.line - 1];
                println!("  {}", line_content);
                
                // Show caret at the column position
                if location.column > 0 {
                    let caret_pos = location.column.saturating_sub(1);
                    let caret = " ".repeat(caret_pos.min(line_content.len())) + "^";
                    println!("  {}", caret);
                }
            }
        } else {
            println!();
        }
    }
}
