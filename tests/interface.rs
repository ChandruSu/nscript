use ns::{error::Error, Interpreter, ModuleFnRecord, NativeFnPtr, Value};

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
