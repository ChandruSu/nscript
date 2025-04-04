use ns::{
    error::{Error, ErrorType},
    Interpreter, ModuleFnRecord, NativeFnPtr, Value,
};

#[test]
pub fn test_module_embed() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let square: NativeFnPtr = |env, arg0, _argc| match env.reg(arg0) {
        Value::Int(i) => Ok(Value::Int(i * i)),
        _ => Error::custom_error("Can't square that").err(),
    };

    nsi.environment_mut().register_module(
        "math".to_string(),
        vec![ModuleFnRecord::new("square".to_string(), 1, square)],
    );

    let result = nsi.evaluate_from_string("import(\"math\").square(3)");
    assert!(result.is_ok(), "Evaluation should succeed");
    assert_eq!(result.unwrap(), Value::Int(9));
}

#[test]
pub fn test_module_embed_failure() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let square: NativeFnPtr = |env, arg0, _argc| match env.reg(arg0) {
        Value::Int(i) => Ok(Value::Int(i * i)),
        _ => Error::custom_error("Can't square that").err(),
    };

    nsi.environment_mut().register_module(
        "math".to_string(),
        vec![ModuleFnRecord::new("square".to_string(), 1, square)],
    );

    let result = nsi.evaluate_from_string("import(\"math\").square(null)");
    assert!(result.is_err(), "Evaluation should fail");

    let err = result.unwrap_err();
    assert_eq!(err.err_type, ErrorType::CustomError);
    assert_eq!(err.msg, "Can't square that");
}

#[test]
pub fn test_interpreter_execute() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let result = nsi.execute_from_string("let x = 0; while x < 10 { x += 3; }");
    assert!(result.is_ok(), "Evaluation should succeed");

    let x = nsi.environment().get_global(&"x".to_string());
    assert!(x.is_some(), "Symbol 'x' should be defined in global scope");
    assert_eq!(x.unwrap(), &Value::Int(12));
    nsi.environment_mut().set_reg(0, Value::Null);
}

#[test]
pub fn test_interpreter_execute_file() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let result = nsi.execute_from_file("examples/close.ns");
    assert!(result.is_ok(), "Evaluation should succeed");

    let x = nsi.environment().get_global(&"six".to_string());
    assert!(
        x.is_some(),
        "Symbol 'six' should be defined in global scope"
    );
    assert_eq!(x.unwrap(), &Value::Int(6));
}

#[test]
pub fn test_undefined_symbol() {
    let nsi = Interpreter::new(false, false, vec![]);

    let x = nsi.environment().get_global(&"x".to_string());
    assert!(
        x.is_none(),
        "Symbol 'x' should not be defined in global scope"
    );
}

#[test]
pub fn test_debug_mode_output() {
    let mut nsi = Interpreter::new(true, true, vec![]);

    let result = nsi.execute_from_string("fun inc(x) { return x + 1; }");
    assert!(result.is_ok(), "Evaluation should succeed");
}

#[test]
pub fn test_error_trace() {
    let mut nsi = Interpreter::new(true, true, vec![]);
    let result = nsi.execute_from_string("std.println(x);");
    assert!(result.is_err(), "Evaluation should fail");
    result.unwrap_err().dump_error(nsi.environment());
}

#[test]
pub fn test_stress_gc() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let result = nsi.execute_from_file("examples/garbage.ns");
    assert!(result.is_ok(), "Evaluation should succeed");
}

#[test]
pub fn test_stress_heap() {
    let mut nsi = Interpreter::new(false, false, vec![]);

    let result = nsi.execute_from_string(
        "\
        let arr = [];\
        let std = import(\"std\");\
        while std.len(arr) < 1000 {\
            std.append(arr, {\"n\": std.len(arr)});\
        }\
        arr = null;\
        std.gc();\
    ",
    );

    assert!(result.is_ok(), "Evaluation should succeed");
}
