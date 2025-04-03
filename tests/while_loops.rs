use ns::{error::ErrorType, Interpreter, Value};

#[test]
pub fn test_while() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let x = 5; let y = 0; while x > 0 { y += x; x -= 1; }");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"y".to_string());
    assert_eq!(val.unwrap(), &Value::Int(15));
}

#[test]
pub fn test_while_no_run() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let x = 5; let y = 0; while x > 6 { y += x; x -= 1; }");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"y".to_string());
    assert_eq!(val.unwrap(), &Value::Int(0));
}

#[test]
pub fn test_while_parse_error() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let x = 5; while { y += x; }");
    assert!(state.is_err(), "Statement should fail");
    assert_eq!(state.unwrap_err().err_type, ErrorType::SyntaxError);
}

#[test]
pub fn test_while_break() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string(
        "let x = 0; while true { x += 1; if x > 10 && x % 7 == 0 { break; } }",
    );
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"x".to_string());
    assert_eq!(val.unwrap(), &Value::Int(14));
}

#[test]
pub fn test_while_continue() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string(
        "let x = 0; let y = 0; while x < 10 { x += 1; if x % 2 { continue; } y += x; }",
    );
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"y".to_string());
    assert_eq!(val.unwrap(), &Value::Int(30));
}
