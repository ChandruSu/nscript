use std::collections::HashSet;

use clap::Parser;
use clap::Subcommand;

use ns::Interpreter;

#[derive(Parser, Debug)]
#[command(version, about = "The NewScript interpreter.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Enable debug output
    #[arg(short = 'd', long = "debug", global = true)]
    debug: bool,

    /// Enable verbose output
    #[arg(short = 'v', long = "verbose", global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
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

fn main() {
    let args = Cli::parse();

    let mut interpreter = Interpreter::new(args.verbose, args.debug);

    match args.command {
        Command::Run { file } => {
            if let Err(e) = interpreter.execute_from_file(&file) {
                e.dump_error(interpreter.environment());
            }
        }
        Command::Eval { expr } => match interpreter.evaluate_from_string(&expr) {
            Ok(v) => {
                println!("{}", v.repr(interpreter.environment()));
            }
            Err(e) => {
                e.dump_error(interpreter.environment());
            }
        },
        Command::Repl => interpreter.repl(),
    }
}
