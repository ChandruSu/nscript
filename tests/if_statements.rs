use ns::{error::ErrorType, Interpreter, Value};

#[test]
pub fn test_if_statement_on_true() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let x = 2; if x > 1 { _ = true; }");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Bool(true));
}

#[test]
pub fn test_if_statement_on_false() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let x = 2; if x > 1 && x < 0 { _ = true; }");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_ne!(val.unwrap(), &Value::Bool(true));
}

#[test]
pub fn test_if_else_statement_on_true() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let x = 2; if x == 2 { _ = true; } else { _ = false; }");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Bool(true));
}

#[test]
pub fn test_if_else_statement_on_false() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let x = 2; if x != 2 { _ = true; } else { _ = false; }");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Bool(false));
}

#[test]
pub fn test_if_elseif_statement_b0() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string(
        "let x = 6; if x % 3 == 0 { _ = 3; } else if x % 2 == 0 { _ = 2; } else { _ = 1; }",
    );
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(3));
}

#[test]
pub fn test_if_elseif_statement_b1() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string(
        "let x = 4; if x % 3 == 0 { _ = 3; } else if x % 2 == 0 { _ = 2; } else { _ = 1; }",
    );
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(2));
}
#[test]
pub fn test_if_elseif_statement_b2() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string(
        "let x = 5; if x % 3 == 0 { _ = 3; } else if x % 2 == 0 { _ = 2; } else { _ = 1; }",
    );
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(1));
}

#[test]
pub fn test_if_elseif_parse_failure() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let x = 6; if x % 3 == 0 { _ = 3; } else if { _ = 2; }");
    assert!(state.is_err(), "Statement should fail");
    assert_eq!(state.unwrap_err().err_type, ErrorType::SyntaxError);
}
