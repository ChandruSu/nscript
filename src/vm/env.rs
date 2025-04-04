use std::{collections::HashMap, rc::Rc};

use crate::{
    backend::{
        opcodes::{Ins, Reg},
        stdlib,
    },
    error,
    utils::io,
};

use super::{
    heap::{Alloc, Heap, HeapNode},
    segment::Segment,
    value::Value,
    NativeFnPtr,
};

struct CallInfo {
    pc: usize,
    sp: usize,
    program: usize,
    closure: usize,
    retloc: usize,
}

pub struct ModuleFnRecord {
    name: String,
    function_pointer: NativeFnPtr,
    arg_count: Reg,
}

pub struct Env {
    segments: Vec<Segment>,
    calls: Vec<CallInfo>,
    registers: Vec<Value>,
    globals: Vec<Value>,
    pub heap: Heap,
    pub sources: io::SourceManager,
    modules: HashMap<String, usize>,
}

impl Env {
    pub fn new(args: Vec<String>) -> Self {
        let mut env = Self {
            calls: vec![],
            registers: vec![Value::Null; 1024],
            globals: vec![],
            heap: Heap::new(8),
            sources: io::SourceManager::new(),
            modules: HashMap::new(),
            segments: vec![
                Segment::empty("__start".to_string(), true),
                Segment::native("__import".to_string(), 1, Self::import),
            ],
        };

        stdlib::register_standard_library(&mut env);

        let args_array = env.heap.allocate(HeapNode::array(
            args.into_iter()
                .map(|a| Value::String(Rc::new(a)))
                .collect(),
        ));

        env.set_global("args".to_string(), Value::Array(args_array));
        env
    }

    pub fn trace_pos(&self) -> Vec<io::Pos> {
        self.calls
            .iter()
            .rev()
            .filter_map(|call| self.get_segment(call.program).get_pos(call.pc - 1))
            .map(io::Pos::clone)
            .collect()
    }

    fn import(&mut self, arg0: usize, argc: usize) -> Result<Value, error::Error> {
        let args = &self.registers[arg0..arg0 + argc];

        match args.first() {
            Some(Value::String(name)) => {
                let module = name.to_string();
                match self.modules.get(&module) {
                    Some(v) => Ok(Value::Object(*v)),
                    None => error::Error::module_not_found(module)
                        .with_pos(self.last_call_pos())
                        .err(),
                }
            }
            _ => error::Error::argument_error(0, 1)
                .with_pos(self.last_call_pos())
                .err(),
        }
    }

    pub fn register_module(&mut self, name: String, exports: Vec<ModuleFnRecord>) {
        let mut module = HashMap::new();

        for method in exports {
            module.insert(
                Value::from_string(&method.name),
                Value::Func(self.segments().len() as u32, 0),
            );

            self.segments_mut().push(Segment::native(
                method.name,
                method.arg_count,
                method.function_pointer,
            ));
        }

        let ptr = self.heap.allocate(HeapNode::object(module));
        self.modules.insert(name, ptr);
    }

    pub fn gc(&mut self, _arg0: usize, _argc: usize) -> Result<Value, error::Error> {
        let active_register_range = 0..self
            .calls
            .last()
            .map(|call| call.sp + self.segments[call.program].slots() as usize)
            .unwrap_or(0);

        let global_register_range = 0..self.get_segment(0).symbols().len();

        for register in self.registers[active_register_range]
            .iter()
            .chain(self.globals[global_register_range].iter())
        {
            if let Value::Object(p) | Value::Array(p) | Value::Func(_, p) = register {
                self.heap.mark(*p)
            }
        }

        for module in self.modules.values() {
            self.heap.mark(*module);
        }

        self.heap.sweep();
        Ok(Value::Null)
    }

    pub fn new_seg(&mut self, segment: Segment) -> usize {
        self.segments.push(segment);
        self.segments.len() - 1
    }

    pub fn segments(&self) -> &Vec<Segment> {
        &self.segments
    }

    pub fn get_segment(&self, fid: usize) -> &Segment {
        &self.segments[fid]
    }

    pub fn get_segment_mut(&mut self, fid: usize) -> &mut Segment {
        &mut self.segments[fid]
    }

    pub fn segments_mut(&mut self) -> &mut Vec<Segment> {
        &mut self.segments
    }

    pub fn reg(&self, i: usize) -> &Value {
        &self.registers[i]
    }

    pub fn set_reg(&mut self, i: usize, value: Value) {
        self.registers[i] = value;
    }

    pub fn get_global(&self, symbol: &String) -> Option<&Value> {
        self.get_segment(0)
            .get_symbol(symbol)
            .map(|id| &self.globals[id as usize])
    }

    pub fn set_global(&mut self, symbol: String, value: Value) {
        let register = self.get_segment_mut(0).get_or_create_symbol(symbol) as usize;
        if register >= self.globals.len() {
            self.globals.resize(register + 1, Value::Null);
        }
        self.globals[register] = value;
    }

    pub fn last_call_pos(&self) -> Option<&io::Pos> {
        self.calls
            .last()
            .and_then(|call| self.segments[call.program].get_pos(call.pc))
    }

    pub fn execute(&mut self, program: usize, closure: usize) -> Result<(), error::Error> {
        self.globals
            .resize(self.get_segment(program).symbols().len() * 2, Value::Null);

        self.calls.push(CallInfo {
            pc: 0,
            sp: 0,
            retloc: 0,
            closure,
            program,
        });

        'next_call: while let Some(mut ci) = self.calls.pop() {
            let pg = &self.segments[ci.program];

            if let Some(function) = pg.native_function_pointer() {
                let slots = pg.slots();
                self.registers[ci.retloc] = function(self, ci.sp, slots as usize)
                    .map_err(|e| e.with_pos(self.last_call_pos()))?;

                continue 'next_call;
            }

            let bp = ci.sp + pg.slots() as usize + 1;
            if bp >= self.registers.len() {
                self.registers.resize(bp, Value::Null);
            }

            let reg = &mut self.registers[ci.sp..bp];
            while ci.pc < pg.bytecode().len() {
                match pg.bytecode()[ci.pc] {
                    Ins::Nop => {}
                    Ins::Not(a, b) => {
                        reg[a as usize] = Value::Bool(!reg[b as usize].truthy());
                    }
                    Ins::Neg(a, b) => {
                        reg[a as usize] =
                            (-&reg[b as usize]).map_err(|e| e.with_pos(pg.get_pos(ci.pc)))?
                    }
                    Ins::BitNot(a, b) => {
                        reg[a as usize] = reg[b as usize]
                            .bit_flip()
                            .map_err(|e| e.with_pos(pg.get_pos(ci.pc)))?
                    }
                    Ins::Eq(a, b, c) => {
                        reg[a as usize] = Value::Bool(reg[b as usize] == reg[c as usize])
                    }
                    Ins::Neq(a, b, c) => {
                        reg[a as usize] = Value::Bool(reg[b as usize] != reg[c as usize])
                    }
                    Ins::Le(a, b, c) => {
                        reg[a as usize] = Value::Bool(&reg[b as usize] <= &reg[c as usize])
                    }
                    Ins::Lt(a, b, c) => {
                        reg[a as usize] = Value::Bool(&reg[b as usize] < &reg[c as usize])
                    }
                    Ins::Add(a, b, c) => {
                        reg[a as usize] = (&reg[b as usize] + &reg[c as usize])
                            .map_err(|e| e.with_pos(pg.get_pos(ci.pc)))?;
                    }
                    Ins::Sub(a, b, c) => {
                        reg[a as usize] = (&reg[b as usize] - &reg[c as usize])
                            .map_err(|e| e.with_pos(pg.get_pos(ci.pc)))?;
                    }
                    Ins::Mul(a, b, c) => {
                        reg[a as usize] = (&reg[b as usize] * &reg[c as usize])
                            .map_err(|e| e.with_pos(pg.get_pos(ci.pc)))?;
                    }
                    Ins::Div(a, b, c) => {
                        reg[a as usize] = (&reg[b as usize] / &reg[c as usize])
                            .map_err(|e| e.with_pos(pg.get_pos(ci.pc)))?;
                    }
                    Ins::Mod(a, b, c) => {
                        reg[a as usize] = (&reg[b as usize] % &reg[c as usize])
                            .map_err(|e| e.with_pos(pg.get_pos(ci.pc)))?;
                    }
                    Ins::Shl(a, b, c) => {
                        reg[a as usize] = (&reg[b as usize] << &reg[c as usize])
                            .map_err(|e| e.with_pos(pg.get_pos(ci.pc)))?;
                    }
                    Ins::Shr(a, b, c) => {
                        reg[a as usize] = (&reg[b as usize] >> &reg[c as usize])
                            .map_err(|e| e.with_pos(pg.get_pos(ci.pc)))?;
                    }
                    Ins::BitAnd(a, b, c) => {
                        reg[a as usize] = (&reg[b as usize] & &reg[c as usize])
                            .map_err(|e| e.with_pos(pg.get_pos(ci.pc)))?;
                    }
                    Ins::BitOr(a, b, c) => {
                        reg[a as usize] = (&reg[b as usize] | &reg[c as usize])
                            .map_err(|e| e.with_pos(pg.get_pos(ci.pc)))?;
                    }
                    Ins::BitXor(a, b, c) => {
                        reg[a as usize] = (&reg[b as usize] ^ &reg[c as usize])
                            .map_err(|e| e.with_pos(pg.get_pos(ci.pc)))?;
                    }
                    Ins::SetG(a, b) => {
                        self.globals[a as usize] = reg[b as usize].clone();
                    }
                    Ins::Move(a, b) => {
                        reg[a as usize] = reg[b as usize].clone();
                    }
                    Ins::LoadN(a) => {
                        reg[a as usize] = Value::Null;
                    }
                    Ins::LoadB(a, b) => {
                        reg[a as usize] = Value::Bool(b);
                    }
                    Ins::LoadF(a, b) => {
                        reg[a as usize] = Value::Func(b as u32, 0);
                    }
                    Ins::LoadG(a, b) => {
                        reg[a as usize] = self.globals[b as usize].clone();
                    }
                    Ins::LoadU(a, b) => {
                        reg[a as usize] = match self.heap.access(ci.closure) {
                            HeapNode::Closure { mark: _, vals } => vals[b as usize].clone(),
                            _ => unreachable!("value-pointer heap-object type mismatch"),
                        }
                    }
                    Ins::LoadK(a, b) => {
                        reg[a as usize] = pg.constant(b as usize).clone();
                    }
                    Ins::JumpFalse(a, b) => {
                        if !reg[a as usize].truthy() {
                            ci.pc = b;
                            continue;
                        }
                    }
                    Ins::JumpTrue(a, b) => {
                        if reg[a as usize].truthy() {
                            ci.pc = b;
                            continue;
                        }
                    }
                    Ins::Jump(a) => {
                        ci.pc = a;
                        continue;
                    }
                    Ins::Close(a, b, c) => match &reg[a as usize] {
                        Value::Func(program, _) => {
                            reg[a as usize] = Value::Func(
                                *program,
                                self.heap.allocate(HeapNode::closure(
                                    reg[b as usize..c as usize].to_vec(),
                                )),
                            );
                        }
                        t0 => error::Error::uncallable_type(t0)
                            .with_pos(pg.get_pos(ci.pc))
                            .err()?,
                    },
                    Ins::Call(a, b, c) => match &reg[b as usize] {
                        Value::Func(program, closure) => {
                            let sp = ci.sp + c as usize;
                            let retloc = ci.sp + a as usize;
                            ci.pc += 1;

                            self.calls.push(ci);
                            self.calls.push(CallInfo {
                                pc: 0,
                                sp,
                                retloc,
                                program: *program as usize,
                                closure: *closure as usize,
                            });
                            continue 'next_call;
                        }
                        t0 => error::Error::uncallable_type(t0)
                            .with_pos(pg.get_pos(ci.pc))
                            .err()?,
                    },
                    Ins::Ret(a) => {
                        let v = reg[a as usize].clone();
                        reg.fill(Value::Null);
                        self.registers[ci.retloc] = v;
                        continue 'next_call;
                    }
                    Ins::RetNone => {
                        reg.fill(Value::Null);
                        self.registers[ci.retloc] = Value::Null;
                        continue 'next_call;
                    }
                    Ins::ObjNew(a) => {
                        if self.heap.should_collect() {
                            self.gc(0, 0)?;
                            self.registers[ci.sp + a as usize] =
                                Value::Object(self.heap.allocate(HeapNode::object(HashMap::new())));

                            ci.pc += 1;
                            self.calls.push(ci);
                            continue 'next_call;
                        }

                        reg[a as usize] =
                            Value::Object(self.heap.allocate(HeapNode::object(HashMap::new())));
                    }
                    Ins::ArrNew(a, n) => {
                        if self.heap.should_collect() {
                            self.gc(0, 0)?;

                            self.registers[ci.sp + a as usize] = Value::Array(
                                self.heap
                                    .allocate(HeapNode::array(vec![Value::Null; n as usize])),
                            );

                            ci.pc += 1;
                            self.calls.push(ci);
                            continue 'next_call;
                        }

                        reg[a as usize] = Value::Array(
                            self.heap
                                .allocate(HeapNode::array(vec![Value::Null; n as usize])),
                        );
                    }
                    Ins::ObjGet(a, b, c) => {
                        match &reg[b as usize] {
                            Value::Object(ptr) => {
                                reg[a as usize] = match self.heap.access(*ptr) {
                                    HeapNode::Object { mark: _, map } => {
                                        map.get(&reg[c as usize]).cloned().unwrap_or(Value::Null)
                                    }
                                    _ => unreachable!("value-pointer heap-object type mismatch"),
                                }
                            }
                            Value::Array(ptr) => {
                                reg[a as usize] = match self.heap.access(*ptr) {
                                    HeapNode::Array { mark: _, vec } => match &reg[c as usize] {
                                        Value::Int(i) if 0 <= *i && (*i as usize) < vec.len() => {
                                            vec[*i as usize].clone()
                                        }
                                        Value::Int(i) => error::Error::array_index_error(*i as u32)
                                            .with_pos(pg.get_pos(ci.pc))
                                            .err()?,
                                        v => error::Error::type_error(&Value::Int(0), &v)
                                            .with_pos(pg.get_pos(ci.pc))
                                            .err()?,
                                    },
                                    _ => unreachable!("value-pointer heap-object type mismatch"),
                                }
                            }
                            Value::String(s) => {
                                reg[a as usize] = match &reg[c as usize] {
                                    Value::Int(i) if 0 <= *i && (*i as usize) < s.len() => s
                                        .chars()
                                        .nth(*i as usize)
                                        .map(|c| Value::String(Rc::new(c.to_string())))
                                        .unwrap_or(Value::Null),
                                    Value::Int(i) => error::Error::array_index_error(*i as u32)
                                        .with_pos(pg.get_pos(ci.pc))
                                        .err()?,
                                    v => error::Error::type_error(&Value::Int(0), &v)
                                        .with_pos(pg.get_pos(ci.pc))
                                        .err()?,
                                }
                            }
                            v => error::Error::type_error_any(&v)
                                .with_pos(pg.get_pos(ci.pc))
                                .err()?,
                        };
                    }
                    Ins::ObjIns(a, b, c) => {
                        let k = reg[b as usize].clone();
                        let v = reg[c as usize].clone();
                        match &reg[a as usize] {
                            Value::Object(ptr) => match self.heap.access_mut(*ptr) {
                                HeapNode::Object { mark: _, map } => {
                                    map.insert(k, v);
                                }
                                _ => unreachable!("value-pointer heap-object type mismatch"),
                            },
                            Value::Array(ptr) => match self.heap.access_mut(*ptr) {
                                HeapNode::Array { mark: _, vec } => match k {
                                    Value::Int(i) if 0 <= i && (i as usize) < vec.len() => {
                                        vec[i as usize] = v
                                    }
                                    Value::Int(i) => error::Error::array_index_error(i as u32)
                                        .with_pos(pg.get_pos(ci.pc))
                                        .err()?,
                                    v => error::Error::type_error(&Value::Int(0), &v)
                                        .with_pos(pg.get_pos(ci.pc))
                                        .err()?,
                                },
                                _ => unreachable!("value-pointer heap-object type mismatch"),
                            },
                            v => error::Error::type_error_any(&v)
                                .with_pos(pg.get_pos(ci.pc))
                                .err()?,
                        }
                    }
                    Ins::Import(a) => {
                        let sp = ci.sp + a as usize;
                        let retloc = ci.sp + a as usize;
                        ci.pc += 1;

                        self.calls.push(ci);
                        self.calls.push(CallInfo {
                            pc: 0,
                            sp,
                            retloc,
                            program: 1,
                            closure: 0,
                        });
                        continue 'next_call;
                    }
                };
                ci.pc += 1;
            }
        }
        Ok(())
    }
}

impl ModuleFnRecord {
    pub fn new(name: String, arg_count: u16, function_pointer: NativeFnPtr) -> Self {
        Self {
            name,
            arg_count,
            function_pointer,
        }
    }
}
