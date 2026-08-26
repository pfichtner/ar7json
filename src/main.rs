use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;

use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;
use clap_complete::generate;
use clap_complete::Shell;

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

    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        shell: Shell,
        /// Output file (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate man page
    Man {
        /// Output file (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show version information
    Version,

    /// Create symlinks for short command names in the binary's directory
    Setup {
        /// Directory to create symlinks in (defaults to the binary's directory)
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
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
    let result = match resolve_symlink_command() {
        Some(cmd) => dispatch(cmd),
        None => {
            let cli = Cli::parse();
            dispatch(cli.command)
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

fn resolve_symlink_command() -> Option<Commands> {
    let arg0 = env::args().next()?;
    let name = Path::new(&arg0)
        .file_name()?
        .to_str()?;

    let args: Vec<String> = env::args().skip(1).collect();
    let mut output = None;
    let mut input = None;
    let mut simple = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                output = args.get(i).map(PathBuf::from);
            }
            "--simple" => simple = true,
            _ if input.is_none() => input = Some(PathBuf::from(&args[i])),
            _ => {}
        }
        i += 1;
    }

    match name {
        "ar7-to-json" => Some(Commands::ToJson {
            input,
            output,
            simple,
        }),
        "json-to-ar7" => Some(Commands::ToAr7 { input, output }),
        "ar7-check" => Some(Commands::Check { input }),
        "ar7-fmt" => Some(Commands::Format { input, output }),
        _ => None,
    }
}

fn dispatch(cmd: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        Commands::ToJson {
            input,
            output,
            simple,
        } => cmd_to_json(&input, &output, simple),
        Commands::ToAr7 { input, output } => cmd_to_ar7(&input, &output),
        Commands::Check { input } => cmd_check(&input),
        Commands::Format { input, output } => cmd_format(&input, &output),
        Commands::Completions { shell, output } => cmd_completions(shell, &output),
        Commands::Man { output } => cmd_man(&output),
        Commands::Version => {
            println!("ar7json {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Setup { dir } => cmd_setup(dir.as_ref()),
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

fn cmd_completions(
    shell: Shell,
    output: &Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Cli::command();
    let mut buf = Vec::new();
    generate(shell, &mut cmd, "ar7json", &mut buf);
    write_output(output, &String::from_utf8(buf)?)?;
    Ok(())
}

fn cmd_man(output: &Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    clap_mangen::Man::new(Cli::command()).render(&mut buf)?;
    write_output(output, &String::from_utf8(buf)?)?;
    Ok(())
}

const SYMLINK_NAMES: &[&str] = &["ar7-to-json", "json-to-ar7", "ar7-check", "ar7-fmt"];

fn cmd_setup(dir: Option<&PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let target_dir = match dir {
        Some(d) => d.clone(),
        None => env::current_exe()?
            .parent()
            .ok_or("cannot determine binary directory")?
            .to_path_buf(),
    };

    let exe = env::current_exe()?;
    let exe_name = exe
        .file_name()
        .ok_or("cannot determine binary name")?
        .to_os_string();

    fs::create_dir_all(&target_dir)?;

    let mut created = Vec::new();

    for name in SYMLINK_NAMES {
        let link_path = target_dir.join(name);
        if link_path.exists() || link_path.symlink_metadata().is_ok() {
            fs::remove_file(&link_path)?;
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(&exe_name, &link_path)?;
        #[cfg(not(unix))]
        fs::copy(&exe, &link_path)?;
        created.push(name);
    }

    eprintln!("Created {} symlinks in {}:", created.len(), target_dir.display());
    for name in &created {
        eprintln!("  {} -> {}", name, exe.display());
    }

    Ok(())
}
