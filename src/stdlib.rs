pub mod stdlib {
    use std::collections::HashMap;

    use crate::{
        compiler::compiler::Reg,
        error,
        vm::{
            heap::{Alloc, GCObject},
            Env, NativeFnPtr, Segment, Value,
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
                    Value::Int(i) if 0 <= i && (i as usize) < vec.len() => {
                        Ok(vec.remove(i as usize))
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

    fn form_module(env: &mut Env, exports: Vec<(String, Reg, NativeFnPtr)>) -> usize {
        let mut module = HashMap::new();

        for (fname, fargs, fptr) in exports {
            module.insert(
                Value::from_string(&fname),
                Value::Func(env.segments().len() as u32, 0),
            );
            env.segments_mut().push(Segment::native(fname, fargs, fptr));
        }

        env.heap.alloc(GCObject::object(module))
    }

    pub fn load_std_into_env(env: &mut Env) -> usize {
        form_module(
            env,
            vec![
                ("println".to_string(), 1, std_println),
                ("print".to_string(), 1, std_print),
                ("typeof".to_string(), 1, std_typeof),
                ("len".to_string(), 1, std_len),
                ("str".to_string(), 1, std_str),
                ("append".to_string(), 2, std_array_append),
                ("insert".to_string(), 3, std_insert),
                ("remove".to_string(), 2, std_remove),
                ("pop".to_string(), 1, std_array_pop),
                ("keys".to_string(), 1, std_object_keys),
                ("gc".to_string(), 0, Env::gc),
            ],
        )
    }
}
