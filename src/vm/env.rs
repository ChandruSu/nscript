use std::collections::{BTreeMap, HashMap};

use crate::{
    compiler::compiler::{self, Ins, Reg},
    error,
    lexer::lexer,
    parser::parser,
    stdlib::stdlib,
    utils::io,
};

use super::{
    heap::{Alloc, GCObject, Heap},
    segment::Segment,
    value::Value,
};

struct CallInfo {
    pc: usize,
    sp: usize,
    program: usize,
    closure: usize,
    retloc: usize,
}

pub struct Env {
    segments: Vec<Segment>,
    calls: Vec<CallInfo>,
    registers: Vec<Value>,
    globals: Vec<Value>,
    pub heap: Heap,
    pub sources: io::SourceManager,
    import_cache: HashMap<String, usize>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            calls: vec![],
            registers: vec![Value::Null; 1024], // TODO: make these dynamic Stack allocators
            globals: vec![Value::Null; 128],
            heap: Heap::new(8),
            sources: io::SourceManager::new(),
            import_cache: HashMap::new(),
            segments: vec![
                Segment::empty("__start".to_string(), true),
                Segment::native("__import".to_string(), 1, Self::import),
            ],
        }
    }

    pub fn trace_pos(&self) -> Vec<io::Pos> {
        self.calls
            .iter()
            .rev()
            .filter_map(|call| self.get_segment(call.program).get_pos(call.pc))
            .map(io::Pos::clone)
            .collect()
    }

    fn import(&mut self, arg0: usize, argc: usize) -> Result<Value, error::Error> {
        let args = &self.registers[arg0..arg0 + argc];

        let path = match args.first() {
            Some(Value::String(path)) => path.to_string(),
            _ => return error::Error::argument_error(0, 1).err(),
        };

        if let Some(v) = self.import_cache.get(&path) {
            Ok(Value::Object(*v))
        } else if path == "std" {
            // TODO: make this extendible
            let i = stdlib::load_std_into_env(self);
            self.import_cache.insert(path, i);
            Ok(Value::Object(i))
        } else {
            // TODO: Maybe spawn new env and for each global, pull env value and realloc on current env's heap
            let old_calls = std::mem::take(&mut self.calls);
            let old_globals = std::mem::replace(&mut self.globals, vec![Value::Null; 128]);
            let old_registers = std::mem::replace(&mut self.registers, vec![Value::Null; 1024]);
            let old_main = std::mem::replace(
                &mut self.segments[0],
                Segment::empty("__start".to_string(), true),
            );

            let src = self.sources.load_source_file(&path)?;
            let ast = &parser::Parser::new(&mut lexer::Lexer::new(src)).parse()?;

            compiler::Compiler::new(self).compile(ast)?;
            self.execute(0)?;

            let exports = self.segments[0]
                .symbols()
                .iter()
                .map(|(k, v)| (Value::from_string(k), self.globals[*v as usize].clone()))
                .collect();

            let i = self.heap.alloc(GCObject::object(exports));
            self.import_cache.insert(path, i);

            self.calls = old_calls;
            self.globals = old_globals;
            self.registers = old_registers;
            self.segments[0] = old_main;

            Ok(Value::Object(i))
        }
    }

    pub fn gc(&mut self, _arg0: usize, _argc: usize) -> Result<Value, error::Error> {
        let active_register_range = 0..self
            .calls
            .last()
            .map(|call| call.sp + self.segments[call.program].slots() as usize)
            .unwrap_or(0);

        println!("{:?}", active_register_range);

        for register in self.registers[active_register_range]
            .iter()
            .chain(self.globals.iter())
        {
            match register {
                Value::Object(p) | Value::Func(_, p) => self.heap.mark(*p),
                _ => continue,
            };
        }

        self.heap.sweep();
        Ok(Value::Null)
    }

    pub fn new_seg(
        &mut self,
        name: String,
        global: bool,
        slots: Reg,
        bytecode: Vec<compiler::Ins>,
        constants: Vec<Value>,
        symbols: HashMap<String, Reg>,
        upvals: HashMap<String, Reg>,
        positions: BTreeMap<usize, io::Pos>,
        parent: Option<usize>,
    ) -> usize {
        self.segments.push(Segment::new(
            name, global, slots, bytecode, constants, symbols, upvals, parent, positions, None,
        ));
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

    pub fn reg_global(&self, i: usize) -> &Value {
        &self.globals[i]
    }

    pub fn execute(&mut self, program: usize) -> Result<(), error::Error> {
        self.calls.push(CallInfo {
            pc: 0,
            sp: 0,
            closure: 0,
            retloc: 0,
            program,
        });

        'next_call: while let Some(mut ci) = self.calls.pop() {
            let pg = &self.segments[ci.program];

            if let Some(native_fptr) = pg.native_function_pointer() {
                self.registers[ci.retloc] = native_fptr(self, ci.sp, pg.slots() as usize)?;
                // TODO: self.registers[ci.sp..ci.sp + pg.slots as usize + 1].fill(Value::Null);
                continue 'next_call;
            }

            let reg = &mut self.registers[ci.sp..ci.sp + pg.slots() as usize + 1];
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
                            GCObject::Closure { mark: _, vals } => vals[b as usize].clone(),
                            _ => todo!(),
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
                                self.heap
                                    .alloc(GCObject::closure(reg[b as usize..c as usize].to_vec())),
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
                        reg[a as usize] =
                            Value::Object(self.heap.alloc(GCObject::object(HashMap::new())));
                    }
                    Ins::ObjGet(a, b, c) => {
                        match reg[b as usize] {
                            Value::Object(ptr) => {
                                reg[a as usize] = match self.heap.access(ptr) {
                                    GCObject::Object { mark: _, map } => {
                                        map[&reg[c as usize]].clone()
                                    }
                                    _ => todo!(),
                                }
                            }
                            _ => todo!(),
                        };
                    }
                    Ins::ObjIns(a, b, c) => {
                        let k = reg[b as usize].clone();
                        let v = reg[c as usize].clone();
                        match reg[a as usize] {
                            Value::Object(ptr) => match self.heap.access_mut(ptr) {
                                GCObject::Object { mark: _, map } => {
                                    map.insert(k, v);
                                }
                                _ => todo!(),
                            },
                            _ => todo!(),
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
