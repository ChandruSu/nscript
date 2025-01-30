use std::time::Instant;

use ns::compiler::compiler;
use ns::lexer::lexer::{self};
use ns::parser::parser;
use ns::vm::vm;

fn main() {
    let mut env = vm::Env::new();
    let source = match env.sources.load_source_file("./examples/test.ns") {
        Ok(s) => s,
        Err(e) => {
            e.dump_error(&env);
            return;
        }
    };

    let start = Instant::now();
    let mut lexer = lexer::Lexer::new(source);
    let mut parser = parser::Parser::new(&mut lexer);

    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            e.dump_error(&env);
            return;
        }
    };

    if let Err(e) = compiler::Compiler::new(&mut env).compile(&ast) {
        e.dump_error(&env);
        return;
    }

    println!("Execution took: {}ms", start.elapsed().as_millis());

    // println!("{}", ast);

    for (idx, program) in env.segments().iter().enumerate() {
        println!("[idx = {}]\n{:?}", idx, program);
    }

    if let Err(e) = env.execute(0) {
        e.dump_error(&env);
        return;
    }

    // for i in 0..10 {
    //     println!("G({}) = {:?}", i, env.reg_global(i));
    // }

    // println!();
    // for i in 0..16 {
    //     println!("R({}) = {:?}", i, env.reg(i));
    // }

    // println!();
    // env.heap.dump();
    // println!("Done");
}
