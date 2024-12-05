use ns::lexer::lexer;
use ns::utils::io;

fn main() {
    let mut manager = io::SourceManager::new();
    let source = match manager.load_source_file("./examples/test.ns") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            return;
        }
    };

    println!("Source code\n: {:#?}", source);

    let mut lexer = lexer::Lexer::new(source);

    while let Ok(token) = lexer.next_token() {
        match token.tk {
            lexer::Tk::EOF => break,
            _ => println!("{:?}", token),
        };
    }
}
