use ns::{error::ErrorType, Alloc, HeapNode, Interpreter, Value};

#[test]
pub fn test_array_creation() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let result = nsi.evaluate_from_string("[1, 2, 3]");
    assert!(result.is_ok(), "Expression should succeed");
    let value = result.unwrap();
    assert!(matches!(value, Value::Array(_)));

    if let Value::Array(ptr) = value {
        let array = nsi.environment().heap.access(ptr);
        assert!(matches!(array, HeapNode::Array { mark: _, vec: _ }));

        if let HeapNode::Array { mark: _, vec } = array {
            assert!(vec.len() == 3);
            assert_eq!(vec.last(), Some(&Value::Int(3)));
        }
    }
}

#[test]
pub fn test_array_subscript() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let arr = [1, 2, 3]; _ = arr[2];");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(3));
}

#[test]
pub fn test_array_subscript_invalid_type() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let result = nsi.evaluate_from_string("[1, 2][\"a\"]");
    assert!(result.is_err(), "Statement should fail");
    assert_eq!(result.unwrap_err().err_type, ErrorType::TypeError("String"));
}

#[test]
pub fn test_invalid_type_subscript() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let result = nsi.evaluate_from_string("null[1]");
    assert!(result.is_err(), "Statement should fail");
    assert_eq!(result.unwrap_err().err_type, ErrorType::TypeError("Null"));
}

#[test]
pub fn test_array_subscript_assign() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let arr = [1, 2, 3]; arr[2] = 5; _ = arr[2];");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(5));
}

#[test]
pub fn test_array_subscript_assign_invalid_index() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let arr = [1, 2, 3]; arr[5] = 5;");
    assert!(state.is_err(), "Statement should fail");
    assert_eq!(state.unwrap_err().err_type, ErrorType::IndexError(5));
}

#[test]
pub fn test_array_subscript_assign_invalid_type() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let arr = [1, 2, 3]; arr[\"a\"] = 5;");
    assert!(state.is_err(), "Statement should fail");
    assert_eq!(state.unwrap_err().err_type, ErrorType::TypeError("String"));
}

#[test]
pub fn test_array_subscript_index_error() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let arr = [1, 2, 3]; _ = arr[4];");
    assert!(state.is_err(), "Statement should fail");
    assert_eq!(state.unwrap_err().err_type, ErrorType::IndexError(4));
}

#[test]
pub fn test_object_creation() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let result = nsi.evaluate_from_string("{\"a\": 3}");
    assert!(result.is_ok(), "Expression should succeed");
    let value = result.unwrap();
    assert!(matches!(value, Value::Object(_)));

    if let Value::Object(ptr) = value {
        let obj = nsi.environment().heap.access(ptr);
        assert!(matches!(obj, HeapNode::Object { mark: _, map: _ }));

        if let HeapNode::Object { mark: _, map } = obj {
            assert!(map.len() == 1);
            assert_eq!(map.get(&Value::from_string("a")), Some(&Value::Int(3)));
        }
    }
}

#[test]
pub fn test_object_subscript() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let obj = {\"a\": 5}; _ = obj[\"a\"];");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(5));
}

#[test]
pub fn test_object_member_syntax() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let obj = {\"a\": 5}; _ = obj.a;");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(5));
}

#[test]
pub fn test_object_subscript_assign() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let obj = {\"a\": 5}; obj[\"a\"] = 4; _ = obj[\"a\"];");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(4));
}

#[test]
pub fn test_object_member_assign() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let obj = {\"a\": 5}; obj.a = 4; _ = obj.a;");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Int(4));
}

#[test]
pub fn test_object_subscript_missing_key() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("let obj = {}; _ = obj.a;");
    assert!(state.is_ok(), "Statement should succeed");

    let val = nsi.environment().get_global(&"_".to_string());
    assert_eq!(val.unwrap(), &Value::Null);
}

#[test]
pub fn test_non_object_subscript() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("_ = null; _[0] = null;");
    assert!(state.is_err(), "Statement should succeed");
    assert_eq!(state.unwrap_err().err_type, ErrorType::TypeError("Null"));
}

#[test]
pub fn test_gc() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let state = nsi.execute_from_string("_ = {\"a\": 3}.a; import(\"std\").gc();");
    assert!(state.is_ok(), "Statement should succeed");
}
