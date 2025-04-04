use ns::{error::ErrorType, Interpreter, Value};

#[test]
pub fn test_function_declaration() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("fun add(a, b) { return a + b; }");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"add".to_string());
    assert!(matches!(val.unwrap(), &Value::Func(_, 0)));
}

#[test]
pub fn test_closure_declaration() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state =
        nsi.execute_from_string("fun mult(n) { return fun(x) { return x * n; }; } _ = mult(3);");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string()).unwrap();
    assert!(matches!(val, &Value::Func(_, _)));
    if let Value::Func(_, c) = val {
        assert_ne!(*c, 0, "Function value must be a closure");
    }
}

#[test]
pub fn test_function_call() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("fun add(a, b) { return a + b; } _ = add(7, 4);");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(11));
}

#[test]
pub fn test_function_closure_call() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state =
        nsi.execute_from_string("fun mult(n) { return fun(x) { return x * n; }; } _ = mult(3)(5);");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(15));
}

#[test]
pub fn test_function_void() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("fun test() { return; } _ = test();");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Null);
}

#[test]
pub fn test_function_void_implicit() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("fun test() { } _ = test();");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Null);
}

#[test]
pub fn test_function_constant() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("fun double(x) { return 2 * x; } _ = double(5);");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(10));
}

#[test]
pub fn test_function_locals() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string(
        "fun fib(n) { \
          let x = 1; 
          while n > 0 { x *= n; n -= 1; } 
          return x; \
        } \
        _ = fib(5);",
    );
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(120));
}

#[test]
pub fn test_function_global() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string(
        "let x = 3;
        fun inc(n) { \
          x += n; \
          return x; \
        } \
        _ = inc(2);",
    );
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(5));
}

#[test]
pub fn test_function_parse_failure_missing_signature() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("fun add { return a + b; }");
    assert!(state.is_err(), "Statement should fail");
    assert_eq!(state.unwrap_err().err_type, ErrorType::SyntaxError);
}

#[test]
pub fn test_function_parse_failure_missing_body() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("fun add(a, b)");
    assert!(state.is_err(), "Statement should fail");
    assert_eq!(state.unwrap_err().err_type, ErrorType::SyntaxError);
}

#[test]
pub fn test_function_parse_failure_unknown_symbol() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("fun add(a, b) { return x; }");
    assert!(state.is_err(), "Statement should fail");
    assert_eq!(
        state.unwrap_err().err_type,
        ErrorType::NameError("x".to_string())
    );
}

#[test]
pub fn test_function_un_callable() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let x = null; x();");
    assert!(state.is_err(), "Statement should fail");
    assert_eq!(state.unwrap_err().err_type, ErrorType::TypeError("Null"));
}

#[test]
pub fn test_function_return_invalid_position() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("return;");
    assert!(state.is_err(), "Statement should fail");

    assert_eq!(state.unwrap_err().err_type, ErrorType::SyntaxError);
}
