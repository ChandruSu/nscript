use clap::Parser;

// NS interpreter
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to file to execute
    #[arg(required = true, value_name = "FILE")]
    pub file_name: String,
}
