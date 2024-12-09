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

    match parser.parse() {
        Ok(ast) => println!("{}", ast),
        Err(e) => e.dump_error(&manager),
    }
}
