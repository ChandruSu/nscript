use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    ops,
    rc::Rc,
};

use crate::{error, frontend::operator};

use super::{
    env::Env,
    heap::{Alloc, GCObject},
};

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(Rc<String>),
    Func(u32, usize),
    Object(usize),
    Array(usize),
}

impl Value {
    pub fn truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Int(v) => *v != 0,
            Value::Float(v) => *v != 0.0,
            Value::Bool(v) => *v,
            Value::Func(_, _) => true,
            Value::String(v) => v.len() > 0,
            Value::Object(_) => true,
            Value::Array(_) => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "Null",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Bool(_) => "Boolean",
            Value::Func(_, _) => "Function",
            Value::String(_) => "String",
            Value::Object(_) => "Object",
            Value::Array(_) => "Array",
        }
    }

    pub fn bit_flip(&self) -> Result<Self, error::Error> {
        match self {
            Value::Int(v) => Ok(Value::Int(!v)),
            t0 => error::Error::op_type_mismatch_un(operator::Op::BitNot, t0).err(),
        }
    }

    pub fn from_string(s: &str) -> Value {
        Value::String(Rc::new(s.to_string()))
    }

    pub fn repr(&self, env: &Env) -> String {
        self.repr_safe(env, &mut HashSet::new())
    }

    fn repr_safe(&self, env: &Env, visited: &mut HashSet<usize>) -> String {
        match self {
            Value::String(v) => format!("'{}'", v),
            _ => self.to_string_safe(env, visited),
        }
    }

    pub fn to_string(&self, env: &Env) -> String {
        self.to_string_safe(env, &mut HashSet::new())
    }

    fn to_string_safe(&self, env: &Env, visited: &mut HashSet<usize>) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Int(v) => format!("{}", v),
            Value::Float(v) => format!("{}", v),
            Value::String(v) => format!("{}", v),
            Value::Bool(v) => if *v { "true" } else { "false" }.to_string(),
            Value::Func(f, _) => {
                let s = env.get_segment(*f as usize);
                format!("<function '{}' at {:p}>", s.name(), s)
            }
            Value::Array(v) if visited.contains(v) => "[...]".to_string(),
            Value::Object(v) if visited.contains(v) => "{ ... }".to_string(),
            Value::Array(v) => {
                visited.insert(*v);
                match env.heap.access(*v) {
                    GCObject::Array { mark: _, vec } => format!(
                        "[{}]",
                        vec.iter()
                            .map(|v| v.repr_safe(env, visited))
                            .collect::<Vec<String>>()
                            .join(", ")
                    ),
                    _ => unreachable!("value-pointer heap-object type mismatch"),
                }
            }
            Value::Object(v) => {
                visited.insert(*v);
                match env.heap.access(*v) {
                    GCObject::Object { mark: _, map } => format!(
                        "{{ {} }}",
                        map.iter()
                            .map(|(k, v)| format!(
                                "{}: {}",
                                k.repr_safe(env, visited),
                                v.repr_safe(env, visited)
                            ))
                            .collect::<Vec<String>>()
                            .join(", ")
                    ),
                    _ => unreachable!("value-pointer heap-object type mismatch"),
                }
            }
        }
    }

    pub fn length(&self, env: &Env) -> Result<usize, error::Error> {
        match self {
            Value::String(v) => Ok(v.len()),
            Value::Object(p) | Value::Array(p) => match env.heap.access(*p) {
                GCObject::Array { mark: _, vec } => Ok(vec.len()),
                GCObject::Object { mark: _, map } => Ok(map.len()),
                _ => unreachable!("value-pointer heap-object type mismatch"),
            },
            Value::Null => error::Error::unexpected_null().err(),
            t1 => error::Error::type_error(self, t1).err(),
        }
    }
}

impl ops::Add<&Value> for &Value {
    type Output = Result<Value, error::Error>;
    fn add(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.wrapping_add(*v1))),
            (Value::Float(v0), Value::Float(v1)) => Ok(Value::Float(v0.add(*v1))),
            (Value::Int(v0), Value::Float(v1)) => Ok(Value::Float((*v0 as f64).add(*v1))),
            (Value::Float(v0), Value::Int(v1)) => Ok(Value::Float(v0.add((*v1) as f64))),
            (Value::String(v0), Value::String(v1)) => {
                Ok(Value::String(Rc::new(v0.to_string() + v1)))
            }
            (t0, t1) => error::Error::op_type_mismatch(operator::Op::Add, t0, t1).err(),
        }
    }
}

impl ops::Sub<&Value> for &Value {
    type Output = Result<Value, error::Error>;
    fn sub(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.wrapping_sub(*v1))),
            (Value::Float(v0), Value::Float(v1)) => Ok(Value::Float(v0.sub(*v1))),
            (Value::Int(v0), Value::Float(v1)) => Ok(Value::Float((*v0 as f64).sub(*v1))),
            (Value::Float(v0), Value::Int(v1)) => Ok(Value::Float(v0.sub((*v1) as f64))),
            (t0, t1) => error::Error::op_type_mismatch(operator::Op::Sub, t0, t1).err(),
        }
    }
}

impl ops::Mul<&Value> for &Value {
    type Output = Result<Value, error::Error>;
    fn mul(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.wrapping_mul(*v1))),
            (Value::Float(v0), Value::Float(v1)) => Ok(Value::Float(v0.mul(*v1))),
            (Value::Int(v0), Value::Float(v1)) => Ok(Value::Float((*v0 as f64).mul(*v1))),
            (Value::Float(v0), Value::Int(v1)) => Ok(Value::Float(v0.mul((*v1) as f64))),
            (t0, t1) => error::Error::op_type_mismatch(operator::Op::Mul, t0, t1).err(),
        }
    }
}

impl ops::Rem<&Value> for &Value {
    type Output = Result<Value, error::Error>;
    fn rem(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.wrapping_rem(*v1))),
            (Value::Float(v0), Value::Float(v1)) => Ok(Value::Float(v0.rem(*v1))),
            (Value::Int(v0), Value::Float(v1)) => Ok(Value::Float((*v0 as f64).rem(*v1))),
            (Value::Float(v0), Value::Int(v1)) => Ok(Value::Float(v0.rem((*v1) as f64))),
            (t0, t1) => error::Error::op_type_mismatch(operator::Op::Mod, t0, t1).err(),
        }
    }
}

impl ops::Div<&Value> for &Value {
    type Output = Result<Value, error::Error>;
    fn div(self, rhs: &Value) -> Self::Output {
        match rhs {
            Value::Int(0) | Value::Float(0.0) => error::Error::zero_division().err(),
            _ => match (self, rhs) {
                (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.wrapping_div(*v1))),
                (Value::Float(v0), Value::Float(v1)) => Ok(Value::Float(v0.div(*v1))),
                (Value::Int(v0), Value::Float(v1)) => Ok(Value::Float((*v0 as f64).div(*v1))),
                (Value::Float(v0), Value::Int(v1)) => Ok(Value::Float(v0.div((*v1) as f64))),
                (t0, t1) => error::Error::op_type_mismatch(operator::Op::Div, t0, t1).err(),
            },
        }
    }
}

impl ops::BitAnd<&Value> for &Value {
    type Output = Result<Value, error::Error>;
    fn bitand(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.bitand(*v1))),
            (t0, t1) => error::Error::op_type_mismatch(operator::Op::BitAnd, t0, t1).err(),
        }
    }
}

impl ops::BitOr<&Value> for &Value {
    type Output = Result<Value, error::Error>;
    fn bitor(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.bitor(*v1))),
            (t0, t1) => error::Error::op_type_mismatch(operator::Op::BitOr, t0, t1).err(),
        }
    }
}

impl ops::BitXor<&Value> for &Value {
    type Output = Result<Value, error::Error>;
    fn bitxor(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.bitxor(*v1))),
            (t0, t1) => error::Error::op_type_mismatch(operator::Op::BitXor, t0, t1).err(),
        }
    }
}

impl ops::Shl<&Value> for &Value {
    type Output = Result<Value, error::Error>;
    fn shl(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::Int(v0), Value::Int(v1)) if *v1 >= 0 => {
                Ok(Value::Int(v0.wrapping_shl(*v1 as u32)))
            }
            (Value::Int(_), Value::Int(v1)) => error::Error::negative_shift(*v1).err(),
            (t0, t1) => error::Error::op_type_mismatch(operator::Op::Shl, t0, t1).err(),
        }
    }
}

impl ops::Neg for &Value {
    type Output = Result<Value, error::Error>;
    fn neg(self) -> Self::Output {
        match self {
            Value::Int(i) => Ok(Value::Int(-*i)),
            Value::Float(i) => Ok(Value::Float(-*i)),
            t0 => error::Error::op_type_mismatch_un(operator::Op::Sub, t0).err(),
        }
    }
}

impl PartialOrd for &Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Null, Value::Null) => Some(std::cmp::Ordering::Equal),
            (Value::Int(v0), Value::Int(v1)) => v0.partial_cmp(v1),
            (Value::Float(v0), Value::Float(v1)) => v0.partial_cmp(v1),
            (Value::Bool(v0), Value::Bool(v1)) => v0.partial_cmp(v1),
            (Value::String(v0), Value::String(v1)) => v0.partial_cmp(v1),
            (Value::Func(f0, c0), Value::Func(f1, c1)) => {
                (f0 == f1 && c0 == c1).then_some(std::cmp::Ordering::Equal)
            }
            _ => None,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Null => state.write_u8(0),
            Value::Int(i) => {
                state.write_u8(1);
                i.hash(state);
            }
            Value::Float(f) => {
                state.write_u8(2);
                state.write_u64(f.to_bits());
            }
            Value::Bool(b) => {
                state.write_u8(3);
                b.hash(state);
            }
            Value::String(s) => {
                state.write_u8(4);
                s.hash(state);
            }
            Value::Func(id, addr) => {
                state.write_u8(5);
                id.hash(state);
                addr.hash(state);
            }
            Value::Object(o) => {
                state.write_u8(6);
                o.hash(state);
            }
            Value::Array(v) => {
                state.write_u8(7);
                v.hash(state);
            }
        }
    }
}
