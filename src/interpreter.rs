use std::{
    collections::HashSet,
    io::{self, Write},
    time::Instant,
};

use colored::Colorize;

use crate::{
    backend::compiler::Compiler,
    error,
    frontend::{lexer::Lexer, parser::Parser},
    vm::{self, Env},
};

pub struct Interpreter {
    env: Env,
    verbose: bool,
    debug: bool,
}

impl Interpreter {
    pub fn new(verbose: bool, debug: bool) -> Self {
        let mut env = Env::new();
        env.get_segment_mut(0)
            .symbol_table_mut()
            .insert("_".to_string(), 0);
        Self {
            env,
            verbose,
            debug,
        }
    }

    pub fn environment(&self) -> &Env {
        &self.env
    }

    pub fn environment_mut(&mut self) -> &mut Env {
        &mut self.env
    }

    fn run(&mut self, source_id: u32) -> Result<(), error::Error> {
        let src = self.env.sources.get_source(source_id).unwrap();

        if !self.debug && !self.verbose {
            Ok(src)
                .and_then(|src| Parser::new(&mut Lexer::new(src)).parse())
                .and_then(|ast| Compiler::new(&mut self.env).compile(&ast).map(drop))
                .and_then(|_| self.env.execute(0))
        } else {
            let mut start = Instant::now();

            let ast = Parser::new(&mut Lexer::new(src)).parse()?;
            if self.verbose {
                println!(
                    "[{}] Parsing took: {} microseconds",
                    "verbose".purple(),
                    start.elapsed().as_micros()
                );
            }

            if self.debug {
                println!("[{}] {}", "debug".red(), ast);
            }

            start = Instant::now();
            Compiler::new(&mut self.env).compile(&ast).map(drop)?;
            if self.verbose {
                println!(
                    "[{}] Bytecode compilation took: {} microseconds",
                    "verbose".purple(),
                    start.elapsed().as_micros()
                );
            }

            start = Instant::now();
            let result = self.env.execute(0);
            if self.verbose {
                println!(
                    "[{}] Execution took: {} ms",
                    "verbose".purple(),
                    start.elapsed().as_millis()
                );
            }

            result
        }
    }

    pub fn execute_from_file(&mut self, file_path: &str) -> Result<(), error::Error> {
        self.env.get_segment_mut(0).clear_definition();
        self.env
            .sources
            .load_source_file(file_path)
            .map(|src| src.id())
            .and_then(|src_id| self.run(src_id))
    }

    pub fn execute_from_string(&mut self, source: &str) -> Result<(), error::Error> {
        self.env.get_segment_mut(0).clear_definition();
        self.env
            .sources
            .load_source_string(source)
            .map(|src| src.id())
            .and_then(|src_id| self.run(src_id))
    }

    pub fn evaluate_from_string(&mut self, source: &str) -> Result<vm::Value, error::Error> {
        self.env.get_segment_mut(0).clear_definition();
        self.env
            .sources
            .load_source_string(&format!("_ = {};", source))
            .map(|src| src.id())
            .and_then(|src_id| self.run(src_id))
            .map(|_| self.env.reg(0).clone())
    }

    pub fn repl(&mut self) {
        println!(
            "Welcome to the NewScript REPL. To execute statements, type command, terminate \
             with ';' and hit enter. To evaluate expressions, prefix commands with '=' (no \
             semicolon needed). Type 'exit' to kill REPL."
        );

        let _ = self.execute_from_string("let std = import(\"std\");");

        let mut input = String::new();
        loop {
            print!(">> ");
            io::stdout().flush().unwrap();

            input.clear();
            if let Err(e) = io::stdin().read_line(&mut input) {
                eprintln!("Failed to read from standard input: {}", e);
                break;
            }

            match input.trim() {
                "exit" => {
                    println!("Closing REPL session. Goodbye :)");
                    break;
                }
                e if e.starts_with('=') => match self.evaluate_from_string(&e[1..]) {
                    Err(e) => e.dump_error(&self.env),
                    Ok(_) => {
                        let v = self.env.reg(0);
                        println!("{}", v.repr(&self.env))
                    }
                },
                e => {
                    if let Err(e) = self.execute_from_string(e) {
                        e.dump_error(&self.env)
                    }
                }
            }
        }
    }
}
