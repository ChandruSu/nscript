use core::fmt;
use std::collections::{BTreeMap, HashMap};

use colored::Colorize;

use crate::{
    backend::opcodes::{Ins, Reg},
    error,
    utils::io,
};

use super::{env::Env, value::Value};

pub type NativeFnPtr = fn(&mut Env, usize, usize) -> Result<Value, error::Error>;

pub struct Segment {
    name: String,
    global: bool,
    slots: Reg,
    bytecode: Vec<Ins>,
    constants: Vec<Value>,
    symbols: HashMap<String, Reg>,
    upvals: HashMap<String, Reg>,
    parent: Option<usize>,
    positions: BTreeMap<usize, io::Pos>,
    native: Option<NativeFnPtr>,
}

impl Segment {
    pub fn new(
        name: String,
        global: bool,
        slots: Reg,
        bytecode: Vec<Ins>,
        constants: Vec<Value>,
        symbols: HashMap<String, Reg>,
        upvals: HashMap<String, Reg>,
        parent: Option<usize>,
        positions: BTreeMap<usize, io::Pos>,
        native: Option<NativeFnPtr>,
    ) -> Self {
        Self {
            name,
            global,
            slots,
            bytecode,
            constants,
            upvals,
            symbols,
            positions,
            parent,
            native,
        }
    }

    pub fn empty(name: String, global: bool) -> Self {
        Self {
            name,
            global,
            slots: 0,
            bytecode: vec![],
            constants: vec![],
            upvals: HashMap::new(),
            symbols: HashMap::new(),
            positions: BTreeMap::new(),
            parent: None,
            native: None,
        }
    }

    pub fn native(name: String, args: u16, native: NativeFnPtr) -> Self {
        Self {
            name,
            global: false,
            slots: args,
            bytecode: vec![],
            constants: vec![],
            upvals: HashMap::new(),
            symbols: HashMap::new(),
            positions: BTreeMap::new(),
            parent: None,
            native: Some(native),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn parent(&self) -> Option<usize> {
        self.parent
    }

    pub fn ins(&self) -> &Vec<Ins> {
        &self.bytecode
    }

    pub fn ins_mut(&mut self) -> &mut Vec<Ins> {
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

    pub fn symbols(&self) -> &HashMap<String, Reg> {
        &self.symbols
    }

    pub fn bytecode(&self) -> &Vec<Ins> {
        &self.bytecode
    }

    pub fn native_function_pointer(&self) -> &Option<NativeFnPtr> {
        &self.native
    }

    pub fn constant(&self, i: usize) -> &Value {
        &self.constants[i]
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
                    "{:02} {} {}\n",
                    i,
                    format!("{:?}", op).to_lowercase().green(),
                    self.get_pos(i)
                        .map(|p| format!("{}:{}", p.line + 1, p.column + 1))
                        .unwrap_or_default()
                ))
                .collect::<Vec<String>>()
                .join("")
                .trim_end(),
            "end".green(),
        )
    }
}
