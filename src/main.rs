use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ar7json",
    about = "Standalone AR7 ↔ JSON converter for AVM FRITZ!Box ar7.cfg configuration files",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert AR7 configuration to JSON
    ToJson {
        /// Input file (reads stdin if omitted)
        input: Option<PathBuf>,
        /// Output file (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Use simplified JSON representation (lossy, read-only)
        #[arg(long)]
        simple: bool,
    },

    /// Convert JSON back to AR7 configuration
    ToAr7 {
        /// Input JSON file (reads stdin if omitted)
        input: Option<PathBuf>,
        /// Output file (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Check AR7 configuration for syntax errors
    Check {
        /// Input file (reads stdin if omitted)
        input: Option<PathBuf>,
    },

    /// Format AR7 configuration with canonical formatting
    Format {
        /// Input file (reads stdin if omitted)
        input: Option<PathBuf>,
        /// Output file (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show version information
    Version,
}

fn read_input(input: &Option<PathBuf>) -> Result<String, Box<dyn std::error::Error>> {
    match input {
        Some(path) => Ok(fs::read_to_string(path)?),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

fn write_output(output: &Option<PathBuf>, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    match output {
        Some(path) => {
            fs::write(path, content)?;
        }
        None => {
            io::stdout().write_all(content.as_bytes())?;
        }
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::ToJson {
            input,
            output,
            simple,
        } => cmd_to_json(&input, &output, simple),
        Commands::ToAr7 { input, output } => cmd_to_ar7(&input, &output),
        Commands::Check { input } => cmd_check(&input),
        Commands::Format { input, output } => cmd_format(&input, &output),
        Commands::Version => {
            println!("ar7json {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    };

    match result {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("ar7json: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_to_json(
    input: &Option<PathBuf>,
    output: &Option<PathBuf>,
    simple: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = read_input(input)?;
    let doc = ar7json::parse(&source)?;

    let json = if simple {
        ar7json::json::document_to_simple_json(&doc)?
    } else {
        ar7json::json::document_to_json(&doc)?
    };

    let formatted = serde_json::to_string_pretty(&json)?;
    let mut result = formatted;
    result.push('\n');

    write_output(output, &result)?;
    Ok(())
}

fn cmd_to_ar7(
    input: &Option<PathBuf>,
    output: &Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = read_input(input)?;
    let json_value: serde_json::Value = serde_json::from_str(&source)?;
    let doc = ar7json::json::json_to_document(&json_value)?;
    let ar7 = ar7json::serialize(&doc)?;

    write_output(output, &ar7)?;
    Ok(())
}

fn cmd_check(input: &Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let source = read_input(input)?;
    match ar7json::parse(&source) {
        Ok(_) => {
            eprintln!("ar7json: OK");
            Ok(())
        }
        Err(e) => Err(Box::new(e)),
    }
}

fn cmd_format(
    input: &Option<PathBuf>,
    output: &Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = read_input(input)?;
    let doc = ar7json::parse(&source)?;
    let formatted = ar7json::serialize(&doc)?;

    write_output(output, &formatted)?;
    Ok(())
}
