use clap::Parser;
use json_schema_to_luau::{convert_schema, convert_schema_with_name};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "json-schema-to-luau")]
#[command(about = "Convert JSON Schema to Luau type definitions", long_about = None)]
struct Cli {
    /// Input JSON Schema file (use '-' for stdin)
    #[arg(value_name = "INPUT")]
    input: String,

    /// Output file (defaults to stdout)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Custom type name for the root schema
    #[arg(short, long, value_name = "NAME")]
    type_name: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Read input
    let input_content = if cli.input == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&cli.input)?
    };

    // Convert schema
    let luau_types = if let Some(type_name) = cli.type_name {
        convert_schema_with_name(&input_content, &type_name)?
    } else {
        convert_schema(&input_content)?
    };

    // Write output
    if let Some(output_path) = cli.output {
        fs::write(output_path, luau_types)?;
    } else {
        println!("{}", luau_types);
    }

    Ok(())
}
