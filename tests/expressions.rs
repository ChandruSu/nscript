use ns::{error::ErrorType, Interpreter, Value};

#[test]
pub fn test_addition() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("1 + 2");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(3));
}

#[test]
pub fn test_subtraction() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("5 - 3");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(2));
}

#[test]
pub fn test_multiplication() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("4 * 2");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(8));
}

#[test]
pub fn test_division() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("10 / 2");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(5));
}

#[test]
pub fn test_modulo() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("10 % 3");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(1));
}

#[test]
pub fn test_negative_numbers() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("-5 + 3");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(-2));
}

#[test]
pub fn test_parentheses() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("(2 + 3) * 4");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(20));
}

#[test]
pub fn test_operator_precedence() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("2 + 3 * 4");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(14));
}

#[test]
pub fn test_floating_point_addition() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("1.5 + 2.5");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Float(4.0));
}

#[test]
pub fn test_floating_point_division() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("5.0 / 2.0");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Float(2.5));
}

#[test]
pub fn test_boolean_and() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("true && false");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(false));
}

#[test]
pub fn test_boolean_or() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("true || false");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_boolean_not() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("!true");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(false));
}

#[test]
pub fn test_truthiness() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("!5");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(false));
}

#[test]
pub fn test_equality() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("3 == 3");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_inequality() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("3 != 4");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_greater_than() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("5 > 3");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_less_than() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("2 < 4");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_greater_than_or_equal() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("5 >= 5");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_less_than_or_equal() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("3 <= 3");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_complex_parentheses() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("(2 + 3) * (4 + 5)");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(45));
}

#[test]
pub fn test_nested_parentheses() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("((2 + 3) * 4) + 5");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(25));
}

#[test]
pub fn test_mixed_operations() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("2 + 3 * 4 / 2");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(8));
}

#[test]
pub fn test_chained_operations() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("2 + 3 + 4 * 5");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(25));
}

#[test]
pub fn test_null_exception() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("1 + null");
    assert!(result.is_err(), "Expression should not succeed");
    assert_eq!(result.unwrap_err().err_type, ErrorType::TypeError("Null"));
}

#[test]
pub fn test_modulo_positive() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("10 % 3");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(1));
}

#[test]
pub fn test_modulo_negative() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("-10 % 3");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(-1));
}

#[test]
pub fn test_zero_divisor() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("10 / 0");
    assert!(result.is_err(), "Division by zero should fail");
    assert_eq!(
        result.unwrap_err().err_type,
        ErrorType::ArithmeticError(Value::Int(0))
    );
}

#[test]
pub fn test_and_short_circuit_false() {
    let result =
        Interpreter::new(false, false, vec![]).evaluate_from_string("false && (10 / 0 > 1)");
    assert!(
        result.is_ok(),
        "Short-circuit should prevent division by zero"
    );
    assert_eq!(result.unwrap(), Value::Bool(false));
}

#[test]
pub fn test_or_short_circuit_true() {
    let result =
        Interpreter::new(false, false, vec![]).evaluate_from_string("true || (10 / 0 > 1)");
    assert!(
        result.is_ok(),
        "Short-circuit should prevent division by zero"
    );
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_and_short_circuit_true() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("true && true");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_or_short_circuit_false() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("false || false");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(false));
}

#[test]
pub fn test_floating_point_multiplication() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("2.5 * 4.0");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Float(10.0));
}

#[test]
pub fn test_floating_point_subtraction() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("5.5 - 1.5");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Float(4.0));
}

#[test]
pub fn test_floating_point_modulo() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("5.5 % 2.0");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Float(1.5));
}

#[test]
pub fn test_floating_point_equality() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("2.0 == 2.0");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_floating_point_inequality() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("2.0 != 3.0");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_floating_point_greater_than() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("3.5 > 2.5");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_floating_point_less_than() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("1.5 < 2.5");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_floating_point_greater_than_or_equal() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("3.5 >= 3.5");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_floating_point_less_than_or_equal() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("2.5 <= 3.5");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
pub fn test_bitwise_and() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("6 & 3");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(2));
}

#[test]
pub fn test_bitwise_or() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("6 | 3");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(7));
}

#[test]
pub fn test_bitwise_xor() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("6 ^ 3");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(5));
}

#[test]
pub fn test_left_shift() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("3 << 2");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(12));
}

#[test]
pub fn test_right_shift() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("8 >> 2");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(2))
}

#[test]
pub fn test_bitwise_and_zero() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("5 & 0");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(0));
}

#[test]
pub fn test_bitwise_or_zero() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("5 | 0");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(5));
}

#[test]
pub fn test_bitwise_xor_same_number() {
    let result = Interpreter::new(false, false, vec![]).evaluate_from_string("7 ^ 7");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(0));
}

#[test]
pub fn test_expression_fuzzy() {
    for _ in 0..40 {
        let result = Interpreter::new(false, false, vec![]).evaluate_from_string("7 ^ 7");
        assert!(result.is_ok(), "Expression should succeed");
        assert_eq!(result.unwrap(), Value::Int(0));
    }
}
