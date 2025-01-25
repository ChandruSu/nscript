pub mod vm {
    use core::fmt;
    use std::{
        collections::{BTreeMap, HashMap},
        hash::{Hash, Hasher},
        ops, usize,
    };

    use colored::Colorize;

    use crate::{
        compiler::compiler::{self, Ins, Reg},
        heap::heap::{GCObject, Heap},
        lexer::lexer,
        utils::{error, io},
    };

    #[derive(PartialEq, Debug, Clone)]
    pub enum Value {
        Null,
        Int(i32),
        Float(f32),
        Bool(bool),
        String(Box<String>),
        Func(u32, usize),
        Object(usize),
    }

    pub struct Segment {
        name: String,
        global: bool,
        slots: Reg,
        bytecode: Vec<compiler::Ins>,
        constants: Vec<Value>,
        symbols: HashMap<String, Reg>,
        upvals: HashMap<String, Reg>,
        parent: Option<usize>,
        positions: BTreeMap<usize, io::Pos>,
    }

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
    }

    impl Segment {
        pub fn name(&self) -> &String {
            &self.name
        }

        pub fn parent(&self) -> Option<usize> {
            self.parent
        }

        pub fn ins(&self) -> &Vec<compiler::Ins> {
            &self.bytecode
        }

        pub fn ins_mut(&mut self) -> &mut Vec<compiler::Ins> {
            &mut self.bytecode
        }

        pub fn count(&self) -> usize {
            self.bytecode.len()
        }

        pub fn slots(&self) -> Reg {
            self.slots
        }

        pub fn inc_slots(&mut self, n: Reg) {
            self.slots = std::cmp::max(self.slots, n);
        }

        pub fn locals(&self) -> &HashMap<String, Reg> {
            &self.symbols
        }

        pub fn upvals(&self) -> &HashMap<String, Reg> {
            &self.upvals
        }

        pub fn upvals_mut(&mut self) -> &HashMap<String, Reg> {
            &mut self.upvals
        }

        pub fn consts(&self) -> &Vec<Value> {
            &self.constants
        }

        pub fn is_global(&self) -> bool {
            self.global
        }

        pub fn is_local(&self) -> bool {
            !self.global
        }

        pub fn spare_reg(&self) -> Reg {
            if self.is_global() {
                0
            } else {
                self.symbols.len() as Reg
            }
        }

        pub fn new_symbol(&mut self, id: String) -> Option<Reg> {
            if self.symbols.contains_key(&id) {
                None
            } else {
                let location = Reg::try_from(self.symbols.len()).unwrap();
                self.symbols.insert(id, location);
                self.slots += 1;
                Some(location)
            }
        }

        pub fn get_symbol(&self, id: &String) -> Option<Reg> {
            self.symbols.get(id).map(|r| *r)
        }

        pub fn new_upval(&mut self, id: String) -> Option<Reg> {
            if self.upvals.contains_key(&id) {
                None
            } else {
                let location = Reg::try_from(self.upvals.len()).unwrap();
                self.upvals.insert(id, location);
                Some(location)
            }
        }

        pub fn get_upval(&self, id: &String) -> Option<Reg> {
            self.upvals.get(id).map(|r| *r)
        }

        pub fn storek(&mut self, v: Value) -> Reg {
            Reg::try_from(
                self.constants
                    .iter()
                    .position(|v0| *v0 == v)
                    .unwrap_or_else(|| {
                        self.constants.push(v);
                        self.constants.len() - 1
                    }),
            )
            .unwrap()
        }

        pub fn push_pos(&mut self, pos: io::Pos) {
            self.positions.insert(self.count(), pos);
        }

        pub fn get_pos(&self, instruction_addr: usize) -> Option<&io::Pos> {
            self.positions
                .range(..instruction_addr + 1)
                .next_back()
                .map(|(_, v)| v)
        }
    }

    impl fmt::Debug for Segment {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(
                f,
                "{} {}(slots: {}, locals: {}, upvals: {}, consts: {}) {}\n{}{}\n{}",
                "function".green(),
                self.name().cyan(),
                self.slots(),
                self.locals().len(),
                self.upvals().len(),
                self.consts().len(),
                "do".green(),
                self.constants
                    .iter()
                    .enumerate()
                    .map(|(i, k)| format!("{:02} {} {:?}\n", i, ".const".red(), k))
                    .collect::<Vec<String>>()
                    .join("")
                    .trim_start(),
                self.ins()
                    .iter()
                    .enumerate()
                    .map(|(i, op)| format!(
                        "{:02} {}\n",
                        i,
                        format!("{:?}", op).to_lowercase().green()
                    ))
                    .collect::<Vec<String>>()
                    .join("")
                    .trim_end(),
                "end".green(),
            )
        }
    }

    impl Env {
        pub fn new() -> Self {
            Self {
                calls: vec![],
                registers: vec![Value::Null; 1024],
                globals: vec![Value::Null; 128],
                heap: Heap::new(8),
                segments: vec![Segment {
                    name: "__start".to_string(),
                    global: true,
                    slots: 0,
                    bytecode: vec![],
                    constants: vec![],
                    upvals: HashMap::new(),
                    symbols: HashMap::new(),
                    positions: BTreeMap::new(),
                    parent: None,
                }],
            }
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
            self.segments.push(Segment {
                name,
                global,
                slots,
                bytecode,
                constants,
                upvals,
                symbols,
                positions,
                parent,
            });
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
                let reg = &mut self.registers[ci.sp..ci.sp + pg.slots as usize + 1];

                while ci.pc < pg.bytecode.len() {
                    match pg.bytecode[ci.pc] {
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
                            reg[a as usize] = match self.heap.get(ci.closure) {
                                GCObject::Closure(vec) => vec[b as usize].clone(),
                                _ => todo!(),
                            }
                        }
                        Ins::LoadK(a, b) => {
                            reg[a as usize] = pg.constants[b as usize].clone();
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
                                    self.heap.alloc(GCObject::Closure(
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
                            reg[a as usize] =
                                Value::Object(self.heap.alloc(GCObject::Object(HashMap::new())));
                        }
                        Ins::ObjGet(a, b, c) => {
                            let ptr = match reg[b as usize] {
                                Value::Object(ptr) => ptr,
                                _ => todo!(),
                            };

                            reg[a as usize] = match self.heap.get(ptr) {
                                GCObject::Object(m) => m[&reg[c as usize]].clone(),
                                _ => todo!(),
                            }
                        }
                        Ins::ObjIns(a, b, c) => {
                            let k = reg[b as usize].clone();
                            let v = reg[c as usize].clone();
                            match reg[a as usize] {
                                Value::Object(ptr) => match self.heap.get(ptr) {
                                    GCObject::Object(m) => {
                                        m.insert(k, v);
                                    }
                                    _ => todo!(),
                                },
                                _ => todo!(),
                            }
                        }
                    };
                    ci.pc += 1;
                }
            }
            Ok(())
        }
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
                Value::Object(_) => todo!(),
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
            }
        }

        pub fn bit_flip(&self) -> Result<Self, error::Error> {
            match self {
                Value::Int(v) => Ok(Value::Int(!v)),
                t0 => error::Error::op_type_mismatch_un(lexer::Op::BitNot, t0).err(),
            }
        }
    }

    impl ops::Add<&Value> for &Value {
        type Output = Result<Value, error::Error>;
        fn add(self, rhs: &Value) -> Self::Output {
            match (self, rhs) {
                (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.wrapping_add(*v1))),
                (Value::Float(v0), Value::Float(v1)) => Ok(Value::Float(v1.add(*v0))),
                (Value::Int(v0), Value::Float(v1)) => Ok(Value::Float((*v0 as f32).add(*v1))),
                (Value::Float(v0), Value::Int(v1)) => Ok(Value::Float(v0.add((*v1) as f32))),
                (Value::String(v0), Value::String(v1)) => {
                    Ok(Value::String(Box::new(v0.to_string() + v1)))
                }
                (t0, t1) => error::Error::op_type_mismatch(lexer::Op::Add, t0, t1).err(),
            }
        }
    }

    impl ops::Sub<&Value> for &Value {
        type Output = Result<Value, error::Error>;
        fn sub(self, rhs: &Value) -> Self::Output {
            match (self, rhs) {
                (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.wrapping_sub(*v1))),
                (Value::Float(v0), Value::Float(v1)) => Ok(Value::Float(v1.sub(*v0))),
                (Value::Int(v0), Value::Float(v1)) => Ok(Value::Float((*v0 as f32).sub(*v1))),
                (Value::Float(v0), Value::Int(v1)) => Ok(Value::Float(v0.sub((*v1) as f32))),
                (t0, t1) => error::Error::op_type_mismatch(lexer::Op::Sub, t0, t1).err(),
            }
        }
    }

    impl ops::Mul<&Value> for &Value {
        type Output = Result<Value, error::Error>;
        fn mul(self, rhs: &Value) -> Self::Output {
            match (self, rhs) {
                (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.wrapping_mul(*v1))),
                (Value::Float(v0), Value::Float(v1)) => Ok(Value::Float(v1.mul(*v0))),
                (Value::Int(v0), Value::Float(v1)) => Ok(Value::Float((*v0 as f32).mul(*v1))),
                (Value::Float(v0), Value::Int(v1)) => Ok(Value::Float(v0.mul((*v1) as f32))),
                (t0, t1) => error::Error::op_type_mismatch(lexer::Op::Mul, t0, t1).err(),
            }
        }
    }

    impl ops::Rem<&Value> for &Value {
        type Output = Result<Value, error::Error>;
        fn rem(self, rhs: &Value) -> Self::Output {
            match (self, rhs) {
                (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.wrapping_rem(*v1))),
                (Value::Float(v0), Value::Float(v1)) => Ok(Value::Float(v1.rem(*v0))),
                (Value::Int(v0), Value::Float(v1)) => Ok(Value::Float((*v0 as f32).rem(*v1))),
                (Value::Float(v0), Value::Int(v1)) => Ok(Value::Float(v0.rem((*v1) as f32))),
                (t0, t1) => error::Error::op_type_mismatch(lexer::Op::Mod, t0, t1).err(),
            }
        }
    }

    impl ops::Div<&Value> for &Value {
        type Output = Result<Value, error::Error>;
        fn div(self, rhs: &Value) -> Self::Output {
            match (self, rhs) {
                (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.wrapping_div(*v1))),
                (Value::Float(v0), Value::Float(v1)) => Ok(Value::Float(v1.div(*v0))),
                (Value::Int(v0), Value::Float(v1)) => Ok(Value::Float((*v0 as f32).div(*v1))),
                (Value::Float(v0), Value::Int(v1)) => Ok(Value::Float(v0.div((*v1) as f32))),
                (t0, t1) => error::Error::op_type_mismatch(lexer::Op::Div, t0, t1).err(),
            }
        }
    }

    impl ops::BitAnd<&Value> for &Value {
        type Output = Result<Value, error::Error>;
        fn bitand(self, rhs: &Value) -> Self::Output {
            match (self, rhs) {
                (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.bitand(*v1))),
                (t0, t1) => error::Error::op_type_mismatch(lexer::Op::BitAnd, t0, t1).err(),
            }
        }
    }

    impl ops::BitOr<&Value> for &Value {
        type Output = Result<Value, error::Error>;
        fn bitor(self, rhs: &Value) -> Self::Output {
            match (self, rhs) {
                (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.bitor(*v1))),
                (t0, t1) => error::Error::op_type_mismatch(lexer::Op::BitOr, t0, t1).err(),
            }
        }
    }

    impl ops::BitXor<&Value> for &Value {
        type Output = Result<Value, error::Error>;
        fn bitxor(self, rhs: &Value) -> Self::Output {
            match (self, rhs) {
                (Value::Int(v0), Value::Int(v1)) => Ok(Value::Int(v0.bitxor(*v1))),
                (t0, t1) => error::Error::op_type_mismatch(lexer::Op::BitXor, t0, t1).err(),
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
                (t0, t1) => error::Error::op_type_mismatch(lexer::Op::Shl, t0, t1).err(),
            }
        }
    }

    impl ops::Neg for &Value {
        type Output = Result<Value, error::Error>;
        fn neg(self) -> Self::Output {
            match self {
                Value::Int(i) => Ok(Value::Int(-*i)),
                Value::Float(i) => Ok(Value::Float(-*i)),
                t0 => error::Error::op_type_mismatch_un(lexer::Op::Sub, t0).err(),
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
                    state.write_u32(f.to_bits());
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
            }
        }
    }
}
