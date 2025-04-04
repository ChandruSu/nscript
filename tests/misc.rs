use std::rc::Rc;

use ns::{error::ErrorType, Interpreter, Value};

#[test]
pub fn test_comment() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let state = nsi.execute_from_string("# This is a comment");
    assert!(state.is_ok(), "Expression should succeed");
}

#[test]
pub fn test_invalid_token() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let state = nsi.execute_from_string("@");
    assert!(state.is_err(), "Statement should fail");
    assert_eq!(state.unwrap_err().err_type, ErrorType::SyntaxError)
}

#[test]
pub fn test_escape_chars() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("\"\\n \\r \\t \\\" \\\\\"");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(
        result.unwrap(),
        Value::String(Rc::new("\n \r \t \" \\".to_string()))
    );
}

#[test]
pub fn test_invalid_escape_char() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("\"\\a\"");
    assert!(result.is_err(), "Expression should fail");
    assert_eq!(result.unwrap_err().err_type, ErrorType::SyntaxError);
}

#[test]
pub fn test_str_subscript() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("\"hello\"[3]");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::String(Rc::new("l".to_string())));
}

#[test]
pub fn test_str_subscript_invalid_index_range() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("\"hello\"[5]");
    assert!(result.is_err(), "Expression should fail");
    assert_eq!(result.unwrap_err().err_type, ErrorType::IndexError(5));
}

#[test]
pub fn test_str_subscript_invalid_index_type() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("\"hello\"[false]");
    assert!(result.is_err(), "Expression should fail");
    assert_eq!(
        result.unwrap_err().err_type,
        ErrorType::TypeError("Boolean")
    );
}

#[test]
pub fn test_invalid_import() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"math\")");
    assert!(result.is_err(), "Expression should fail");
    assert_eq!(
        result.unwrap_err().err_type,
        ErrorType::NameError("math".to_string())
    );
}
