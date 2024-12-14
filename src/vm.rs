pub mod vm {
    use core::fmt;
    use std::collections::HashMap;

    use colored::Colorize;

    use crate::{
        compiler::compiler::{self, Ins, Reg},
        utils::error,
    };

    #[derive(PartialEq, Debug, Clone)]
    pub enum Value {
        Null,
        Int(i32),
        Float(f32),
        Bool(bool),
        Func(u32, u16),
        String(Box<String>),
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
    }

    struct CallInfo {
        pc: usize,
        sp: usize,
        program: usize,
        closure: usize,
    }

    pub struct Env {
        segments: Vec<Segment>,
        calls: Vec<CallInfo>,
        registers: Vec<Value>,
        globals: Vec<Value>,
        closures: Vec<Vec<Value>>,
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
                self.slots - 1
            }
        }

        pub fn new_symbol(&mut self, id: String) -> Option<Reg> {
            if self.symbols.contains_key(&id) {
                None
            } else {
                let location = Reg::try_from(self.symbols.len()).unwrap();
                self.symbols.insert(id, location);
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
    }

    impl fmt::Debug for Segment {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(
                f,
                "{} {}(slots: {}, locals: {}, upvals: {}, consts: {}) {}\n{}{}\n{}\n",
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
                closures: vec![vec![]],
                segments: vec![Segment {
                    name: "__start".to_string(),
                    global: true,
                    slots: 0,
                    bytecode: vec![],
                    constants: vec![],
                    upvals: HashMap::new(),
                    symbols: HashMap::new(),
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

        pub fn execute(&mut self, program: usize) -> Result<(), error::Error> {
            self.calls.push(CallInfo {
                pc: 0,
                sp: 0,
                closure: 0,
                program,
            });

            'next_call: while let Some(ci) = self.calls.pop() {
                let pg = &self.segments[ci.program];
                let registers = &mut self.registers[ci.sp..ci.sp + pg.slots as usize + 1];

                while ci.pc < pg.bytecode.len() {
                    match pg.bytecode[ci.pc] {
                        Ins::Nop => continue,
                        Ins::Neg(_, _) => todo!(),
                        Ins::Not(_, _) => todo!(),
                        Ins::Add(_, _, _) => todo!(),
                        Ins::Sub(_, _, _) => todo!(),
                        Ins::Mul(_, _, _) => todo!(),
                        Ins::Div(_, _, _) => todo!(),
                        Ins::Mod(_, _, _) => todo!(),
                        Ins::Neq(_, _, _) => todo!(),
                        Ins::Eq(_, _, _) => todo!(),
                        Ins::Le(_, _, _) => todo!(),
                        Ins::Lt(_, _, _) => todo!(),
                        Ins::Shl(_, _, _) => todo!(),
                        Ins::BitNot(_, _) => todo!(),
                        Ins::BitOr(_, _, _) => todo!(),
                        Ins::BitXor(_, _, _) => todo!(),
                        Ins::BitAnd(_, _, _) => todo!(),
                        Ins::SetG(a, b) => {
                            self.globals[a as usize] = registers[b as usize].clone();
                        }
                        Ins::Move(a, b) => {
                            registers[a as usize] = registers[b as usize].clone();
                        }
                        Ins::LoadN(a) => {
                            registers[a as usize] = Value::Null;
                        }
                        Ins::LoadB(a, b) => {
                            registers[a as usize] = Value::Bool(b);
                        }
                        Ins::LoadF(a, b) => {
                            registers[a as usize] = Value::Func(b as u32, 0);
                        }
                        Ins::LoadG(a, b) => {
                            registers[a as usize] = self.globals[b as usize].clone();
                        }
                        Ins::LoadU(a, b) => {
                            registers[a as usize] = self.closures[ci.closure][b as usize].clone();
                        }
                        Ins::LoadK(a, b) => {
                            registers[a as usize] = pg.constants[b as usize].clone();
                        }
                        Ins::JumpFalse(_, _) => todo!(),
                        Ins::JumpTrue(_, _) => todo!(),
                        Ins::Jump(_) => todo!(),
                        Ins::Close(_, _, _) => todo!(),
                        Ins::Call(a, b, c) => match registers[a as usize] {
                            Value::Func(program, closure) => {
                                let sp = ci.sp + pg.slots as usize;
                                self.calls.push(ci);
                                self.calls.push(CallInfo {
                                    sp,
                                    pc: 0,
                                    program: program as usize,
                                    closure: closure as usize,
                                });
                                continue 'next_call;
                            }
                            _ => todo!(),
                        },
                        Ins::Ret(a) => {
                            self.registers[ci.sp - 1] = registers[a as usize].clone();
                            continue 'next_call;
                        }
                        Ins::RetNone => {
                            self.registers[ci.sp - 1] = Value::Null;
                            continue 'next_call;
                        }
                    };
                }
            }

            Ok(())
        }
    }
}
