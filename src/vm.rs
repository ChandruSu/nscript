pub mod vm {
    use core::fmt;
    use std::collections::HashMap;

    use colored::Colorize;

    use crate::compiler::compiler::{self, Reg};

    #[derive(PartialEq, Debug)]
    pub enum Value {
        Null,
        Int(i32),
        Float(f32),
        Bool(bool),
        Func(u32, u16),
        String(String),
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
    }

    pub struct Env {
        segments: Vec<Segment>,
        call_stack: Vec<CallInfo>,
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
                call_stack: vec![],
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
    }
}
