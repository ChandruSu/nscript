use crate::{
    error,
    vm::{
        heap::{Alloc, GCObject},
        Env, ModuleFnRecord, Value,
    },
};

fn assert_arg_count(env: &Env, rec: usize, exp: usize) -> Result<(), error::Error> {
    if rec != exp {
        error::Error::argument_error(rec as u32, exp as u32)
            .with_pos(env.last_call_pos())
            .err()
    } else {
        Ok(())
    }
}

fn std_println(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 1)?;
    println!("{}", env.reg(arg0).to_string(env));
    Ok(Value::Null)
}

fn std_print(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 1)?;
    print!("{}", env.reg(arg0).to_string(env));
    Ok(Value::Null)
}

fn std_typeof(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 1)?;
    Ok(Value::from_string(env.reg(arg0).type_name()))
}

fn std_len(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 1)?;
    env.reg(arg0).length(env).map(|len| Value::Int(len as i32))
}

fn std_str(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 1)?;
    Ok(Value::String(Box::new(env.reg(arg0).to_string(env))))
}

fn std_array_append(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 2)?;
    let v = env.reg(arg0 + 1).clone();
    match env.reg(arg0) {
        Value::Array(arr) => match env.heap.access_mut(*arr) {
            GCObject::Array { mark: _, vec } => vec.push(v),
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        v => error::Error::type_error(v, &Value::Array(0))
            .with_pos(env.last_call_pos())
            .err()?,
    }
    Ok(Value::Null)
}

fn std_array_pop(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 1)?;
    match env.reg(arg0) {
        Value::Array(arr) => match env.heap.access_mut(*arr) {
            GCObject::Array { mark: _, vec } => vec
                .pop()
                .ok_or(error::Error::array_length_error(0).with_pos(env.last_call_pos())),
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        v => error::Error::type_error(v, &Value::Array(0))
            .with_pos(env.last_call_pos())
            .err(),
    }
}

fn std_insert(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 3)?;
    let key = env.reg(arg0 + 1).clone();
    let val = env.reg(arg0 + 2).clone();
    match env.reg(arg0) {
        Value::Array(p) => match env.heap.access_mut(*p) {
            GCObject::Array { mark: _, vec } => match key {
                Value::Int(i) if 0 <= i && (i as usize) < vec.len() => {
                    vec.insert(i as usize, val);
                    Ok(Value::Null)
                }
                Value::Int(i) => error::Error::array_index_error(i as u32)
                    .with_pos(env.last_call_pos())
                    .err(),
                v => error::Error::type_error(&v, &Value::Int(0))
                    .with_pos(env.last_call_pos())
                    .err(),
            },
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        Value::Object(p) => match env.heap.access_mut(*p) {
            GCObject::Object { mark: _, map } => {
                map.insert(key, val);
                Ok(Value::Null)
            }
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        v => error::Error::type_error_any(v)
            .with_pos(env.last_call_pos())
            .err(),
    }
}

fn std_remove(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 2)?;
    let key = env.reg(arg0 + 1).clone();
    match env.reg(arg0) {
        Value::Array(p) => match env.heap.access_mut(*p) {
            GCObject::Array { mark: _, vec } => match key {
                Value::Int(i) if 0 <= i && (i as usize) < vec.len() => Ok(vec.remove(i as usize)),
                Value::Int(i) => error::Error::array_index_error(i as u32)
                    .with_pos(env.last_call_pos())
                    .err(),
                v => error::Error::type_error(&v, &Value::Int(0))
                    .with_pos(env.last_call_pos())
                    .err(),
            },
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        Value::Object(p) => match env.heap.access_mut(*p) {
            GCObject::Object { mark: _, map } => Ok(map.remove(&key).unwrap_or(Value::Null)),
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        v => error::Error::type_error_any(v)
            .with_pos(env.last_call_pos())
            .err(),
    }
}

fn std_object_keys(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 2)?;
    match env.reg(arg0) {
        Value::Object(p) => match env.heap.access_mut(*p) {
            GCObject::Object { mark: _, map } => {
                let keys = map.keys().map(|v| v.clone()).collect();
                Ok(Value::Array(env.heap.alloc(GCObject::array(keys))))
            }
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        v => error::Error::type_error(v, &Value::Object(0))
            .with_pos(env.last_call_pos())
            .err(),
    }
}

pub fn register_standard_library(env: &mut Env) {
    env.register_module(
        "std".to_string(),
        vec![
            ModuleFnRecord {
                name: "println".to_string(),
                arg_count: 1,
                function: std_println,
            },
            ModuleFnRecord {
                name: "print".to_string(),
                arg_count: 1,
                function: std_print,
            },
            ModuleFnRecord {
                name: "typeof".to_string(),
                arg_count: 1,
                function: std_typeof,
            },
            ModuleFnRecord {
                name: "len".to_string(),
                arg_count: 1,
                function: std_len,
            },
            ModuleFnRecord {
                name: "str".to_string(),
                arg_count: 1,
                function: std_str,
            },
            ModuleFnRecord {
                name: "append".to_string(),
                arg_count: 2,
                function: std_array_append,
            },
            ModuleFnRecord {
                name: "insert".to_string(),
                arg_count: 3,
                function: std_insert,
            },
            ModuleFnRecord {
                name: "remove".to_string(),
                arg_count: 2,
                function: std_remove,
            },
            ModuleFnRecord {
                name: "pop".to_string(),
                arg_count: 1,
                function: std_array_pop,
            },
            ModuleFnRecord {
                name: "keys".to_string(),
                arg_count: 1,
                function: std_object_keys,
            },
            ModuleFnRecord {
                name: "gc".to_string(),
                arg_count: 0,
                function: Env::gc,
            },
        ],
    )
}
