//! Command-line interface for querying the periodic table.

mod output;

use clap::{Args, Parser, Subcommand, ValueEnum};
use output::{print_text, ElementOutput};
use pt_services::{ElementView, PeriodicTable};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "pt", about = "Query the periodic table")]
struct Cli {
    /// Directory containing element YAML files.
    #[arg(long, default_value = "./data/elements", global = true)]
    data_dir: PathBuf,
    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Text, global = true)]
    format: Format,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Look up a single element.
    Get(GetArgs),
    /// List all elements.
    List,
}

#[derive(Args)]
struct GetArgs {
    #[arg(long)]
    number: Option<u8>,
    #[arg(long)]
    symbol: Option<String>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    mass: Option<f64>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
    Yaml,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let table = PeriodicTable::load(&cli.data_dir)?;
    match cli.command {
        Command::Get(args) => match lookup(&table, &args) {
            Some(view) => {
                emit(&ElementOutput::from_view(&view), cli.format)?;
                Ok(ExitCode::SUCCESS)
            }
            None => {
                eprintln!("error: no matching element found");
                Ok(ExitCode::FAILURE)
            }
        },
        Command::List => {
            let mut outputs: Vec<ElementOutput> =
                table.all().map(|v| ElementOutput::from_view(&v)).collect();
            outputs.sort_by_key(|o| o.atomic_number);
            match cli.format {
                Format::Text => {
                    for o in &outputs {
                        println!("{:>3}  {:<3} {}", o.atomic_number, o.symbol, o.name);
                    }
                }
                Format::Json => println!("{}", serde_json::to_string_pretty(&outputs)?),
                Format::Yaml => print!("{}", serde_yaml_ng::to_string(&outputs)?),
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn lookup<'a>(table: &'a PeriodicTable, args: &GetArgs) -> Option<ElementView<'a>> {
    if let Some(z) = args.number {
        return table.by_atomic_number(z);
    }
    if let Some(symbol) = &args.symbol {
        return table.by_symbol(symbol);
    }
    if let Some(name) = &args.name {
        return table.by_name(name);
    }
    if let Some(mass) = args.mass {
        return table.by_atomic_mass(mass, 0.5);
    }
    None
}

fn emit(o: &ElementOutput, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        Format::Text => print_text(o),
        Format::Json => println!("{}", serde_json::to_string_pretty(o)?),
        Format::Yaml => print!("{}", serde_yaml_ng::to_string(o)?),
    }
    Ok(())
}
