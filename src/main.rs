use std::time::Instant;

use ns::compiler::compiler;
use ns::lexer::lexer::{self};
use ns::parser::parser;
use ns::utils::io;
use ns::vm::vm;

fn main() {
    let mut manager = io::SourceManager::new();
    let source = match manager.load_source_file("./examples/primes.ns") {
        Ok(s) => s,
        Err(e) => {
            e.dump_error(&manager);
            return;
        }
    };

    let start = Instant::now();
    let mut lexer = lexer::Lexer::new(source);
    let mut parser = parser::Parser::new(&mut lexer);

    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            e.dump_error(&manager);
            return;
        }
    };

    let mut env = vm::Env::new();
    if let Err(e) = compiler::Compiler::new(&mut env).compile(&ast) {
        e.dump_error(&manager);
        return;
    }

    if let Err(e) = env.execute(0) {
        e.dump_error(&manager);
        return;
    }

    println!("Execution took: {}ms", start.elapsed().as_millis());

    println!("{}", ast);

    for (idx, program) in env.segments().iter().enumerate() {
        println!("[idx = {}]\n{:?}", idx, program);
    }

    for i in 0..10 {
        println!("G({}) = {:?}", i, env.reg_global(i));
    }

    println!();
    for i in 0..16 {
        println!("R({}) = {:?}", i, env.reg(i));
    }

    println!();
    env.heap.dump();
    println!("Done");
}
