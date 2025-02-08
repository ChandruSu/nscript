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

    fn assert_arg_count(rec: usize, exp: usize) -> Result<(), error::Error> {
        if rec != exp {
            Err(error::Error::argument_error(rec as u32, exp as u32))
        } else {
            Ok(())
        }
    }

    fn std_print(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
        assert_arg_count(argc, 1)?;
        println!("{}", env.reg(arg0).to_string(env));
        Ok(Value::Null)
    }

    fn std_typeof(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
        assert_arg_count(argc, 1)?;
        Ok(Value::from_string(env.reg(arg0).type_name()))
    }

    fn std_len(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
        assert_arg_count(argc, 1)?;
        env.reg(arg0).length(env).map(|len| Value::Int(len as i32))
    }

    fn std_str(env: &mut Env, arg0: usize, argc: usize) -> Result<Value, error::Error> {
        assert_arg_count(argc, 1)?;
        Ok(Value::String(Box::new(env.reg(arg0).to_string(env))))
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
                ("print".to_string(), 1, std_print),
                ("typeof".to_string(), 1, std_typeof),
                ("len".to_string(), 1, std_len),
                ("str".to_string(), 1, std_str),
            ],
        )
    }
}
