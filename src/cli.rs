use std::collections::HashSet;

use clap::Parser;
use clap::Subcommand;

use crate::Interpreter;

// NS interpreter
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable debug output
    #[arg(short = 'd', long = "debug")]
    debug: bool,

    /// Enable verbose output
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a file
    Run {
        /// Path to file to execute
        file: String,
    },

    /// Start a REPL session
    Repl,

    /// Evaluate an expression
    Eval {
        // Expression to evaluate
        expr: String,
    },
}

pub fn execute() {
    let args = Cli::parse();

    let mut interpreter = Interpreter::new(args.verbose, args.debug);

    match args.command {
        Commands::Run { file } => {
            if let Err(e) = interpreter.execute_from_file(&file) {
                e.dump_error(interpreter.environment());
            }
        }
        Commands::Eval { expr } => match interpreter.evaluate_from_string(&expr) {
            Ok(v) => {
                println!("{}", v.repr(interpreter.environment(), &mut HashSet::new()));
            }
            Err(e) => {
                e.dump_error(interpreter.environment());
            }
        },
        Commands::Repl => interpreter.repl(),
    }
}
