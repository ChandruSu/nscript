use ns::compiler::compiler;
use ns::lexer::lexer::{self};
use ns::parser::parser;
use ns::utils::io;

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
        Ok(ast) => {
            println!("{}", ast);
            ast
        }
        Err(e) => {
            e.dump_error(&manager);
            return;
        }
    };

    let _ = match compiler::Compiler::new().compile(&ast) {
        Ok(compiler) => {
            for (idx, program) in compiler.programs().iter().enumerate() {
                println!("[idx = {}]\n{:?}", idx, program);
            }

            compiler
        }
        Err(e) => {
            e.dump_error(&manager);
            return;
        }
    };
}
