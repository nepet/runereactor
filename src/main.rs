use std::io::Read as _;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use rune_forge::compiler::compile;
use rune_forge::parser::parse_policy;
use rune_forge::types::Op;

#[derive(Clone, Debug, ValueEnum)]
enum Format {
    /// JSON array-of-arrays
    Json,
    /// lightning-cli createrune command
    Cln,
    /// Raw restriction string
    Raw,
}

#[derive(Parser, Debug)]
#[command(
    name = "rune-forge",
    about = "Compile .rf rune policies into CLN rune restrictions"
)]
struct Cli {
    /// Path to .rf policy file, or - for stdin
    input: String,

    /// Output format
    #[arg(long, default_value = "json")]
    format: Format,
}

fn read_input(path: &str) -> Result<String> {
    if path == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("Failed to read stdin")?;
        Ok(buf)
    } else {
        std::fs::read_to_string(path).with_context(|| format!("Failed to read {path}"))
    }
}

fn condition_to_string(c: &rune_forge::types::Condition) -> String {
    if c.op == Op::Missing {
        format!("{}{}", c.field, c.op.as_char())
    } else {
        format!("{}{}{}", c.field, c.op.as_char(), c.value)
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let input = read_input(&cli.input)?;

    let policy = parse_policy(&input).context("Failed to parse policy")?;
    let rune_policy = compile(&policy).context("Failed to compile policy")?;

    let restrictions: Vec<Vec<String>> = rune_policy
        .restrictions
        .iter()
        .map(|r| r.alternatives.iter().map(condition_to_string).collect())
        .collect();

    match cli.format {
        Format::Json => {
            let json = serde_json::to_string(&restrictions)?;
            println!("{json}");
        }
        Format::Cln => {
            let json = serde_json::to_string(&restrictions)?;
            println!("lightning-cli createrune -k \"restrictions\"='{json}'");
        }
        Format::Raw => {
            let raw: Vec<String> = restrictions.iter().map(|alts| alts.join("|")).collect();
            println!("{}", raw.join("&"));
        }
    }

    Ok(())
}
