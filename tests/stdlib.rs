use std::{
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use ns::{error::ErrorType, Alloc, HeapNode, Interpreter, Value};

#[test]
pub fn test_std_len() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").len([1, 2])");
    assert!(result.is_ok(), "Statement should succeed");
    assert_eq!(result.unwrap(), Value::Int(2));
}

#[test]
pub fn test_std_print() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").print(3)");
    assert!(result.is_ok(), "Statement should succeed");
}

#[test]
pub fn test_std_time() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let t0 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let result = nsi.evaluate_from_string("import(\"std\").time()");
    let t1 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    assert!(result.is_ok(), "Statement should succeed");
    let t = result.unwrap();
    assert!(&t >= &Value::Int(t0 as i64), "Time is in the past");
    assert!(&t <= &Value::Int(t1 as i64), "Time is in the future");
}

#[test]
pub fn test_std_typeof() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let v0 = nsi.evaluate_from_string("import(\"std\").typeOf(null)");
    let v1 = nsi.evaluate_from_string("import(\"std\").typeOf(5)");
    let v2 = nsi.evaluate_from_string("import(\"std\").typeOf(1.5)");
    let v3 = nsi.evaluate_from_string("import(\"std\").typeOf(false)");
    let v4 = nsi.evaluate_from_string("import(\"std\").typeOf([1,2])");
    let v5 = nsi.evaluate_from_string("import(\"std\").typeOf({\"a\":3})");
    assert_eq!(v0.unwrap(), Value::String(Rc::new("Null".to_string())));
    assert_eq!(v1.unwrap(), Value::String(Rc::new("Int".to_string())));
    assert_eq!(v2.unwrap(), Value::String(Rc::new("Float".to_string())));
    assert_eq!(v3.unwrap(), Value::String(Rc::new("Boolean".to_string())));
    assert_eq!(v4.unwrap(), Value::String(Rc::new("Array".to_string())));
    assert_eq!(v5.unwrap(), Value::String(Rc::new("Object".to_string())));
}

#[test]
pub fn test_std_len_fail() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").len(null)");
    assert!(result.is_err(), "Statement should fail");
    assert_eq!(result.unwrap_err().err_type, ErrorType::TypeError("Null"));
}

#[test]
pub fn test_std_str() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let v0 = nsi.evaluate_from_string("import(\"std\").str(null)");
    let v1 = nsi.evaluate_from_string("import(\"std\").str(5)");
    let v2 = nsi.evaluate_from_string("import(\"std\").str(1.5)");
    let v3 = nsi.evaluate_from_string("import(\"std\").str(false)");
    let v4 = nsi.evaluate_from_string("import(\"std\").str([1,2])");
    let v5 = nsi.evaluate_from_string("import(\"std\").str({\"a\":3})");
    assert_eq!(v0.unwrap(), Value::String(Rc::new("null".to_string())));
    assert_eq!(v1.unwrap(), Value::String(Rc::new("5".to_string())));
    assert_eq!(v2.unwrap(), Value::String(Rc::new("1.5".to_string())));
    assert_eq!(v3.unwrap(), Value::String(Rc::new("false".to_string())));
    assert_eq!(v4.unwrap(), Value::String(Rc::new("[1, 2]".to_string())));
    assert_eq!(
        v5.unwrap(),
        Value::String(Rc::new("{ 'a': 3 }".to_string()))
    );
}

#[test]
pub fn test_std_str_nested_obj() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let state = nsi.execute_from_string(
        "
		let v0 = {}; v0.a = v0; \
		let v1 = {}; v1.a = v1;
	",
    );
    assert!(state.is_ok(), "Statement should succeed");

    let v0 = nsi.evaluate_from_string("v0");
    let v1 = nsi.evaluate_from_string("v1");
    assert!(v0.is_ok(), "Expression should succeed");
    assert!(v1.is_ok(), "Expression should succeed");

    if let Value::String(s0) = v0.unwrap() {
        assert!(s0.contains("..."), "String indicate recursion");
    }

    if let Value::String(s1) = v1.unwrap() {
        assert!(s1.contains("..."), "String indicate recursion");
    }
}

#[test]
pub fn test_std_append() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.execute_from_string("_ = [1, 2]; import(\"std\").append(_, 3);");
    assert!(result.is_ok(), "Statement should succeed");

    let value = nsi.environment().get_global(&"_".to_string()).unwrap();

    if let Value::Object(p) = value {
        if let HeapNode::Array { mark: _, vec } = nsi.environment().heap.access(*p) {
            assert_eq!(vec.len(), 3, "Array should have 3 elements");
            assert_eq!(vec.last(), Some(&Value::Int(3)));
        }
    }
}

#[test]
pub fn test_std_append_fail() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.execute_from_string("_ = null; import(\"std\").append(_, 3);");
    assert!(result.is_err(), "Statement should fail");
}

#[test]
pub fn test_std_insert_arr() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.execute_from_string("_ = [1, 2]; import(\"std\").insert(_, 1, 3);");
    assert!(result.is_ok(), "Statement should succeed");

    let value = nsi.environment().get_global(&"_".to_string()).unwrap();

    if let Value::Array(p) = value {
        if let HeapNode::Array { mark: _, vec } = nsi.environment().heap.access(*p) {
            assert_eq!(vec.len(), 3, "Array should have 3 elements");
            assert_eq!(vec[1], Value::Int(3));
        }
    }
}

#[test]
pub fn test_std_insert_obj() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.execute_from_string("_ = {}; import(\"std\").insert(_, 1, true);");
    assert!(result.is_ok(), "Statement should succeed");

    let value = nsi.environment().get_global(&"_".to_string()).unwrap();

    if let Value::Object(p) = value {
        if let HeapNode::Object { mark: _, map } = nsi.environment().heap.access(*p) {
            assert_eq!(map.len(), 1, "Object should have 1 elements");
            assert_eq!(map.get(&Value::Int(1)), Some(&Value::Bool(true)));
        }
    }
}

#[test]
pub fn test_std_insert_null() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.execute_from_string("_ = null; import(\"std\").insert(_, 1, true);");
    assert!(result.is_err(), "Statement should fail");
}

#[test]
pub fn test_std_pop() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").pop([1, 2, 3])");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(3));
}

#[test]
pub fn test_std_pop_empty() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").pop([])");
    assert!(result.is_err(), "Expression should fail");
    assert_eq!(result.unwrap_err().err_type, ErrorType::IndexError(0));
}

#[test]
pub fn test_std_pop_invalid_type() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").pop(null)");
    assert!(result.is_err(), "Expression should fail");
    assert_eq!(result.unwrap_err().err_type, ErrorType::TypeError("Null"));
}

#[test]
pub fn test_std_remove_arr() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").remove([1, 2, 3], 1)");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(2));
}

#[test]
pub fn test_std_remove_arr_index_err() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").remove([1, 2, 3], 5)");
    assert!(result.is_err(), "Expression should fail");
    assert_eq!(result.unwrap_err().err_type, ErrorType::IndexError(5));
}

#[test]
pub fn test_std_remove_obj() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").remove({\"a\": 5}, \"a\")");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(5));
}

#[test]
pub fn test_std_remove_missing_key() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").remove({\"a\": 5}, \"b\")");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
pub fn test_std_remove_invalid_type() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").remove(null, 0)");
    assert!(result.is_err(), "Expression should fail");
    assert_eq!(result.unwrap_err().err_type, ErrorType::TypeError("Null"));
}

#[test]
pub fn test_std_keys() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").keys({1: true, 2: false})");
    assert!(result.is_ok(), "Expression should succeed");

    if let Value::Array(p) = result.unwrap() {
        if let HeapNode::Array { mark: _, vec } = nsi.environment().heap.access(p) {
            assert_eq!(vec.len(), 2, "Object should have 2 keys");
            assert!(vec.contains(&Value::Int(1)), "Key '2' should be found");
        }
    }
}

#[test]
pub fn test_std_parse_int() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").parseInt(\"3\")");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Int(3));
}

#[test]
pub fn test_std_parse_int_err() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").parseInt(\"a\")");
    assert!(result.is_err(), "Expression should fail");
    assert_eq!(result.unwrap_err().err_type, ErrorType::ValueError);
}

#[test]
pub fn test_std_parse_float() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").parseFloat(\"1.5\")");
    assert!(result.is_ok(), "Expression should succeed");
    assert_eq!(result.unwrap(), Value::Float(1.5));
}

#[test]
pub fn test_std_parse_float_err() {
    let mut nsi = Interpreter::new(false, false, vec![]);
    let result = nsi.evaluate_from_string("import(\"std\").parseFloat(\"a\")");
    assert!(result.is_err(), "Expression should fail");
    assert_eq!(result.unwrap_err().err_type, ErrorType::ValueError);
}
