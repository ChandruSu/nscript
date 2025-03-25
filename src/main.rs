use ns::cli::execute;

fn main() {
    execute();
    // if let Err(e) = interpreter.execute_from_file(&args.file_name) {
    //     e.dump_error(interpreter.environment());
    //     return;
    // }

    // if let Err(e) = interpreter.execute_from_string("x += 1; std.println(x);") {
    //     e.dump_error(interpreter.environment());
    //     return;
    // }

    // match interpreter.evaluate_from_string("std.println(\"A\")") {
    //     Err(e) => {
    //         e.dump_error(interpreter.environment());
    //         return;
    //     }
    //     Ok(v) => {
    //         println!("{}", v.repr(interpreter.environment(), &mut HashSet::new()));
    //     }
    // }

    // let mut env = vm::Env::new();
    // let source = match env.sources.load_source_file(&args.file_name) {
    //     Ok(s) => s,
    //     Err(e) => {
    //         e.dump_error(&env);
    //         return;
    //     }
    // };

    // let start = Instant::now();
    // let mut lexer = lexer::Lexer::new(source);
    // let mut parser = parser::Parser::new(&mut lexer);

    // let ast = match parser.parse() {
    //     Ok(ast) => ast,
    //     Err(e) => {
    //         e.dump_error(&env);
    //         return;
    //     }
    // };

    // if let Err(e) = compiler::Compiler::new(&mut env).compile(&ast) {
    //     e.dump_error(&env);
    //     return;
    // }

    // println!("{}", ast);

    // for (idx, program) in env.segments().iter().enumerate() {
    //     println!("[idx = {}]\n{:?}", idx, program);
    // }

    // println!("<=== STD OUT ===>");

    // if let Err(e) = env.execute(0) {
    //     e.dump_error(&env);
    //     return;
    // }

    // println!("<===============>");

    // println!("Execution took: {}ms", start.elapsed().as_millis());

    // for i in 0..10 {
    //     println!("G({}) = {:?}", i, env.reg_global(i));
    // }

    // println!();
    // for i in 0..16 {
    //     println!("R({}) = {:?}", i, env.reg(i));
    // }

    // println!();
    // env.heap.dump();
}
