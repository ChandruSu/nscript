use std::{
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    error,
    vm::{
        heap::{Alloc, HeapNode},
        Env, ModuleFnRecord, Value,
    },
};

fn assert_arg_count(_env: &Env, rec: usize, exp: usize) -> Result<(), error::Error> {
    if rec != exp {
        error::Error::argument_error(rec as u32, exp as u32).err()
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
    env.reg(arg0).length(env).map(|len| Value::Int(len as i64))
}

fn std_str(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 1)?;
    Ok(Value::String(Rc::new(env.reg(arg0).to_string(env))))
}

fn std_array_append(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 2)?;
    let v = env.reg(arg0 + 1).clone();
    match env.reg(arg0) {
        Value::Array(arr) => match env.heap.access_mut(*arr) {
            HeapNode::Array { mark: _, vec } => vec.push(v),
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        v => error::Error::type_error(&Value::Array(0), v).err()?,
    }
    Ok(Value::Null)
}

fn std_array_pop(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 1)?;
    match env.reg(arg0) {
        Value::Array(arr) => match env.heap.access_mut(*arr) {
            HeapNode::Array { mark: _, vec } => {
                vec.pop().ok_or(error::Error::array_length_error(0))
            }
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        v => error::Error::type_error(&Value::Array(0), v).err(),
    }
}

fn std_insert(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 3)?;
    let key = env.reg(arg0 + 1).clone();
    let val = env.reg(arg0 + 2).clone();
    match env.reg(arg0) {
        Value::Array(p) => match env.heap.access_mut(*p) {
            HeapNode::Array { mark: _, vec } => match key {
                Value::Int(i) if 0 <= i && (i as usize) < vec.len() => {
                    vec.insert(i as usize, val);
                    Ok(Value::Null)
                }
                Value::Int(i) => error::Error::array_index_error(i as u32).err(),
                v => error::Error::type_error(&Value::Int(0), &v).err(),
            },
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        Value::Object(p) => match env.heap.access_mut(*p) {
            HeapNode::Object { mark: _, map } => {
                map.insert(key, val);
                Ok(Value::Null)
            }
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        v => error::Error::type_error_any(v).err(),
    }
}

fn std_remove(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 2)?;
    let key = env.reg(arg0 + 1).clone();
    match env.reg(arg0) {
        Value::Array(p) => match env.heap.access_mut(*p) {
            HeapNode::Array { mark: _, vec } => match key {
                Value::Int(i) if 0 <= i && (i as usize) < vec.len() => Ok(vec.remove(i as usize)),
                Value::Int(i) => error::Error::array_index_error(i as u32).err(),
                v => error::Error::type_error(&Value::Int(0), &v).err(),
            },
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        Value::Object(p) => match env.heap.access_mut(*p) {
            HeapNode::Object { mark: _, map } => Ok(map.remove(&key).unwrap_or(Value::Null)),
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        v => error::Error::type_error_any(v).err(),
    }
}

fn std_object_keys(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 1)?;
    match env.reg(arg0) {
        Value::Object(p) => match env.heap.access_mut(*p) {
            HeapNode::Object { mark: _, map } => {
                let keys = map.keys().map(|v| v.clone()).collect();
                Ok(Value::Array(env.heap.allocate(HeapNode::array(keys))))
            }
            _ => unreachable!("value-pointer heap-object type mismatch"),
        },
        v => error::Error::type_error(&Value::Object(0), v).err(),
    }
}

fn std_time(_env: &mut Env, _arg0: usize, _argc: usize) -> Result<Value, error::Error> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    Ok(Value::Int(millis as i64))
}

fn std_parse_int(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 1)?;
    match env.reg(arg0) {
        Value::String(s) => match s.parse().into() {
            Ok(i) => Ok(Value::Int(i)),
            Err(_) => error::Error::invalid_string_parse_input(s).err(),
        },
        v => error::Error::type_error(&Value::String(Rc::default()), v).err(),
    }
}

fn std_parse_float(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
    assert_arg_count(env, argc, 1)?;
    match env.reg(arg0) {
        Value::String(s) => match s.parse().into() {
            Ok(f) => Ok(Value::Float(f)),
            Err(_) => error::Error::invalid_string_parse_input(s).err(),
        },
        v => error::Error::type_error(&Value::String(Rc::default()), v).err(),
    }
}

pub fn register_standard_library(env: &mut Env) {
    env.register_module(
        "std".to_string(),
        vec![
            ModuleFnRecord::new("println".to_string(), 1, std_println),
            ModuleFnRecord::new("print".to_string(), 1, std_print),
            ModuleFnRecord::new("typeOf".to_string(), 1, std_typeof),
            ModuleFnRecord::new("len".to_string(), 1, std_len),
            ModuleFnRecord::new("str".to_string(), 1, std_str),
            ModuleFnRecord::new("append".to_string(), 2, std_array_append),
            ModuleFnRecord::new("insert".to_string(), 3, std_insert),
            ModuleFnRecord::new("remove".to_string(), 2, std_remove),
            ModuleFnRecord::new("pop".to_string(), 1, std_array_pop),
            ModuleFnRecord::new("keys".to_string(), 1, std_object_keys),
            ModuleFnRecord::new("gc".to_string(), 0, Env::gc),
            ModuleFnRecord::new("time".to_string(), 0, std_time),
            ModuleFnRecord::new("parseInt".to_string(), 1, std_parse_int),
            ModuleFnRecord::new("parseFloat".to_string(), 1, std_parse_float),
        ],
    )
}
