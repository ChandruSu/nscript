use ns::compiler::compiler;
use ns::lexer::lexer::{self};
use ns::parser::parser;
use ns::utils::io;
use ns::vm::vm;

fn main() {
    let mut manager = io::SourceManager::new();
    let source = match manager.load_source_file("./examples/test.ns") {
        Ok(s) => s,
        Err(e) => {
            e.dump_error(&manager);
            return;
        }
    };

    let mut lexer = lexer::Lexer::new(source);
    let mut parser = parser::Parser::new(&mut lexer);

    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            e.dump_error(&manager);
            return;
        }
    };

    println!("{}", ast);

    let mut env = vm::Env::new();
    if let Err(e) = compiler::Compiler::new(&mut env).compile(&ast) {
        e.dump_error(&manager);
        return;
    }

    for (idx, program) in env.segments().iter().enumerate() {
        println!("[idx = {}]\n{:?}", idx, program);
    }
}
