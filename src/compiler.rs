pub mod compiler {
    use core::fmt;
    use std::{any::Any, collections::HashMap, path::Iter, vec};

    type Reg = u32;

    use colored::Colorize;

    use crate::{
        lexer::lexer::{self, Op},
        parser::parser::{self, Ast, AstNode},
        utils::{error, io},
        vm::vm,
    };

    #[derive(Debug)]
    pub enum Ins {
        Nop,
        Neg(Reg, Reg),
        Not(Reg, Reg),
        Add(Reg, Reg, Reg),
        Sub(Reg, Reg, Reg),
        Mul(Reg, Reg, Reg),
        Div(Reg, Reg, Reg),
        Mod(Reg, Reg, Reg),
        Neq(Reg, Reg, Reg),
        Eq(Reg, Reg, Reg),
        Le(Reg, Reg, Reg),
        Lt(Reg, Reg, Reg),
        Shl(Reg, Reg, Reg),
        BitNot(Reg, Reg),
        BitOr(Reg, Reg, Reg),
        BitXor(Reg, Reg, Reg),
        BitAnd(Reg, Reg, Reg),
        Call(Reg, Reg, Reg),
        Close(Reg, Reg, Reg),
        SetG(Reg, Reg),
        Move(Reg, Reg),
        LoadN(Reg),
        LoadB(Reg, bool),
        LoadF(Reg, usize),
        LoadG(Reg, Reg),
        LoadU(Reg, Reg),
        LoadK(Reg, Reg),
        JumpFalse(Reg, usize),
        JumpTrue(Reg, usize),
        Jump(usize),
        Ret(Reg),
        RetNone,
    }

    pub struct Segment {
        name: String,
        global: bool,
        slots: Reg,
        bytecode: Vec<Ins>,
        constants: Vec<vm::Value>,
        symbols: HashMap<String, Reg>,
        upvals: HashMap<String, Reg>,
        parent: Option<usize>,
    }

    pub struct Compiler {
        segments: Vec<Segment>,
        current_segment: usize,
    }

    impl Segment {
        pub fn new(name: String, global: bool) -> Self {
            Self {
                name,
                global,
                slots: 0,
                bytecode: vec![],
                constants: vec![],
                upvals: HashMap::new(),
                symbols: HashMap::new(),
                parent: None,
            }
        }

        pub fn name(&self) -> &String {
            &self.name
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

        pub fn locals(&self) -> usize {
            self.symbols.len()
        }

        pub fn upvals(&self) -> usize {
            self.upvals.len()
        }

        pub fn consts(&self) -> usize {
            self.constants.len()
        }

        pub fn global(&self) -> bool {
            self.global
        }

        pub fn local(&self) -> bool {
            !self.global
        }

        pub fn spare_reg(&self) -> Reg {
            if self.global() {
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

        pub fn storek(&mut self, v: vm::Value) -> Reg {
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
                "{} {}(slots = {}, locals = {}, upvals = {}, consts = {}) {}\n{}{}\n{}\n",
                "function".green(),
                self.name().cyan(),
                self.slots(),
                self.locals(),
                self.upvals(),
                self.consts(),
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

    impl Compiler {
        pub fn new() -> Self {
            Self {
                current_segment: 0,
                segments: vec![Segment::new("__start".to_string(), true)],
            }
        }

        fn seg(&self) -> &Segment {
            &self.segments[self.current_segment]
        }

        fn seg_mut(&mut self) -> &mut Segment {
            &mut self.segments[self.current_segment]
        }

        pub fn programs(&self) -> &Vec<Segment> {
            &self.segments
        }

        fn global_seg(&self) -> &Segment {
            &self.segments[0]
        }

        fn with(&mut self, ins: Ins) -> &mut Self {
            self.seg_mut().ins_mut().push(ins);
            self
        }

        fn set_ins(&mut self, i: usize, ins: Ins) -> &mut Self {
            self.seg_mut().ins_mut()[i] = ins;
            self
        }

        fn set_ins_with_count(&mut self, i: usize, f: &dyn Fn(usize) -> Ins) -> &mut Self {
            self.seg_mut().ins_mut()[i] = f(self.seg().count());
            self
        }

        pub fn compile(&mut self, tree: &parser::AstNode) -> Result<&mut Self, error::Error> {
            self.compile_block(tree).map(|s| s.with(Ins::RetNone))
        }

        fn compile_block(&mut self, n: &parser::AstNode) -> Result<&mut Self, error::Error> {
            match n.ast() {
                Ast::Block(v) => v
                    .iter()
                    .try_for_each(|n| self.compile_statement(n).map(|_| ()))
                    .map(|_| self),
                _ => Err(error::Error::invalid_ast_node(n.pos())),
            }
        }

        fn compile_statement(&mut self, n: &parser::AstNode) -> Result<&mut Self, error::Error> {
            match n.ast() {
                Ast::If(e0, b0, b1) => self.compile_if(e0, b0, b1),
                Ast::While(e0, b0) => self.compile_while(e0, b0),
                Ast::FuncDef(a, b, c) => self.compile_function(None, a, b, c, n.pos()),
                Ast::Let(id, e0) => self.compile_let(id, e0, n.pos()),
                Ast::Assign(op, reference, e0) => self.compile_assign(*op, reference, e0),
                Ast::Call(f, args) => self.compile_call(self.seg().slots, f, args),
                Ast::Return(e0) if self.seg().local() => self.compile_return(e0),
                Ast::Return(_) => Err(error::Error::invalid_return_position(n.pos())),
                _ => unreachable!(),
            }
        }

        fn compile_function(
            &mut self,
            r: Option<Reg>,
            name: &Option<String>,
            args: &Vec<String>,
            body: &AstNode,
            pos: io::Pos,
        ) -> Result<&mut Self, error::Error> {
            let fid = self.segments.len();

            self.segments.push(Segment {
                parent: Some(self.current_segment),
                global: false,
                slots: Reg::try_from(args.len()).unwrap() + 1,
                bytecode: vec![],
                constants: vec![],
                upvals: HashMap::new(),
                name: name.clone().unwrap_or("<lambda>".to_string()),
                symbols: args
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (v.to_string(), Reg::try_from(i).unwrap()))
                    .collect(),
            });

            let old_segment = self.current_segment;
            self.current_segment = fid;

            self.compile_block(body)?;
            if !matches!(self.seg().ins().last(), Some(Ins::RetNone | Ins::Ret(_))) {
                self.with(Ins::RetNone);
            }

            self.current_segment = old_segment;

            let fr = match name {
                None => Ok(r.unwrap()),
                Some(name) => self
                    .seg_mut()
                    .new_symbol(name.to_string())
                    .ok_or_else(|| error::Error::duplicate_var_name(name.to_string(), pos)),
            }?;

            self.with(Ins::LoadF(fr, fid));

            let func = &self.segments[fid];
            if let uc @ 1.. = func.upvals.len() {
                let r = r.map(|r| r + 1).unwrap_or(self.seg().spare_reg());

                func.upvals
                    .clone()
                    .iter()
                    .try_for_each(|(v0, i)| self.compile_id(r + i, v0, pos).map(|_| ()))?;

                self.with(Ins::Close(fr, r, r + Reg::try_from(uc - 1).unwrap()));
                self.seg_mut().slots =
                    std::cmp::max(self.seg().slots, r + Reg::try_from(uc).unwrap());
            }

            Ok(self)
        }

        fn compile_let(
            &mut self,
            id: &String,
            e0: &AstNode,
            pos: io::Pos,
        ) -> Result<&mut Self, error::Error> {
            match self.seg_mut().new_symbol(id.to_string()) {
                Some(r) if self.seg().local() => self.compile_expr(r, e0),
                Some(r) => Ok(self.compile_expr(0, e0)?.with(Ins::SetG(r, 0))),
                None => Err(error::Error::duplicate_var_name(id.to_string(), pos)),
            }
        }

        fn compile_assign(
            &mut self,
            op: lexer::Op,
            v: &AstNode,
            e0: &AstNode,
        ) -> Result<&mut Self, error::Error> {
            let id = match v.ast() {
                Ast::Reference(id) => Ok(id),
                _ => Err(error::Error::invalid_ast_node(v.pos())),
            }?;

            let r = self.seg().spare_reg();
            self.compile_expr(r, e0)?;

            let global_reg = self.global_seg().symbols.get(id);
            let local_reg = self
                .seg()
                .local()
                .then(|| ())
                .and_then(|_| self.seg().symbols.get(id));

            match (global_reg, local_reg) {
                (Some(&gr), None) if op == Op::Assign => Ok(self.with(Ins::SetG(gr, 0))),
                (_, Some(&lr)) if op == Op::Assign => Ok(self.with(Ins::Move(lr, r))),
                (_, Some(&lr)) => Ok(self.with(op.to_ins(lr, lr, r))),
                (Some(&gr), None) => Ok(self
                    .with(Ins::LoadG(r + 1, gr))
                    .with(op.to_ins(r, r + 1, r))
                    .with(Ins::SetG(gr, r))),
                (None, None) => Err(error::Error::mutate_closure(id.to_string(), v.pos())),
            }
        }

        fn compile_return(&mut self, e0: &Option<Box<AstNode>>) -> Result<&mut Self, error::Error> {
            Ok(match e0 {
                None => self.with(Ins::RetNone),
                Some(e0) => {
                    let r = self.seg().spare_reg();
                    self.compile_expr(r, e0)?.with(Ins::Ret(r))
                }
            })
        }

        fn compile_while(&mut self, e0: &AstNode, b0: &AstNode) -> Result<&mut Self, error::Error> {
            let r = self.seg().spare_reg();
            let jmp0 = self.seg().count();
            let jmp1 = self.compile_expr(r, e0)?.seg().count();
            let jmp2 = self.with(Ins::Nop).compile_block(b0)?.seg().count();

            Ok(self
                .set_ins(jmp1, Ins::JumpFalse(r, jmp2))
                .with(Ins::Jump(jmp0)))
        }

        fn compile_if(
            &mut self,
            e0: &AstNode,
            b0: &AstNode,
            b1: &Option<Box<AstNode>>,
        ) -> Result<&mut Self, error::Error> {
            let r = self.seg().spare_reg();

            let jmp0 = self.compile_expr(r, e0)?.seg().count();
            self.with(Ins::Nop).compile_block(b0)?;

            let jmp1 = match b1 {
                None => 0,
                Some(_) => self.with(Ins::Nop).seg().count() - 1,
            };

            self.set_ins(jmp0, Ins::JumpFalse(r, self.seg().count() - 1));

            let len = self.seg().count() - 1;

            Ok(match b1 {
                None => self,
                Some(b1) => match b1.ast() {
                    Ast::Block(_) => self.compile_block(b1),
                    Ast::If(a, b, c) => self.compile_if(a, b, c),
                    _ => unreachable!(),
                }?
                .set_ins(jmp1, Ins::Jump(len)),
            })
        }

        fn compile_expr(&mut self, r: Reg, e: &AstNode) -> Result<&mut Self, error::Error> {
            self.seg_mut().slots = std::cmp::max(self.seg().slots(), r + 1);

            match e.ast() {
                Ast::Call(f, args) => self.compile_call(r, f, args),
                Ast::Reference(id) => self.compile_id(r, id, e.pos()),
                Ast::Deref(_, _) => todo!(),
                Ast::Subscript(_, _) => todo!(),
                Ast::UnaryExp(op, e0) => self.compile_unary(r, *op, e0),
                Ast::TernaryExp(e0, e1, e2) => self.compile_ternary(r, e0, e1, e2),
                Ast::BinaryExp(op, e0, e1) => match op {
                    lexer::Op::Or | lexer::Op::And => self.compile_bool_expr(r, *op, e0, e1),
                    _ => self.compile_bin_expr(r, *op, e0, e1),
                },
                Ast::FuncDef(name, args, body) => {
                    self.compile_function(Some(r), name, args, body, e.pos())
                }
                Ast::Null | Ast::Int(_) | Ast::Float(_) | Ast::Bool(_) | Ast::String(_) => {
                    self.compile_literal(r, e)
                }
                _ => unreachable!(),
            }
        }

        fn compile_ternary(
            &mut self,
            r: Reg,
            e0: &AstNode,
            e1: &AstNode,
            e2: &AstNode,
        ) -> Result<&mut Self, error::Error> {
            let jmp0 = self.compile_expr(r, e0)?.seg().count();

            let jmp1 = self
                .with(Ins::Nop)
                .compile_expr(r, e1)?
                .set_ins_with_count(jmp0, &|c| Ins::JumpFalse(r, c))
                .seg()
                .count();

            self.with(Ins::Nop)
                .set_ins_with_count(jmp1, &|c| Ins::Jump(c))
                .compile_expr(r, e2)
        }

        fn compile_bin_expr(
            &mut self,
            r: Reg,
            op: lexer::Op,
            e0: &AstNode,
            e1: &AstNode,
        ) -> Result<&mut Self, error::Error> {
            Ok(self
                .compile_expr(r, e0)?
                .compile_expr(r + 1, e1)?
                .with(op.to_ins(r, r, r + 1)))
        }

        fn compile_bool_expr(
            &mut self,
            r: Reg,
            op: lexer::Op,
            e0: &AstNode,
            e1: &AstNode,
        ) -> Result<&mut Self, error::Error> {
            let start = self.seg().count();
            self.compile_expr(r, e0)?;

            let jmp = self.seg().count();
            self.with(Ins::Nop).compile_expr(r, e1)?;

            self.set_ins(
                jmp,
                match op {
                    Op::Or => Ins::JumpFalse(r, self.seg().count() - 1),
                    Op::And => Ins::JumpTrue(r, self.seg().count() - 1),
                    _ => unreachable!(),
                },
            );

            for idx in (start..self.seg().count()).rev() {
                let ins = match self.seg().ins().get(idx).unwrap() {
                    ins @ (Ins::JumpTrue(_, d) | Ins::JumpFalse(_, d)) => {
                        match (ins, self.seg().ins().get(*d).unwrap()) {
                            (Ins::JumpTrue(r, _), Ins::JumpTrue(_, d)) => Ins::JumpTrue(*r, *d),
                            (Ins::JumpFalse(r, _), Ins::JumpFalse(_, d)) => Ins::JumpFalse(*r, *d),
                            _ => continue,
                        }
                    }
                    _ => continue,
                };

                self.set_ins(idx, ins);
            }

            Ok(self)
        }

        fn compile_unary(
            &mut self,
            r: Reg,
            op: lexer::Op,
            e0: &AstNode,
        ) -> Result<&mut Self, error::Error> {
            Ok(self.compile_expr(r, e0)?.with(match op {
                Op::Sub => Ins::Neg(r, r),
                Op::Not => Ins::Not(r, r),
                Op::BitNot => Ins::BitNot(r, r),
                _ => unreachable!(),
            }))
        }

        fn compile_call(
            &mut self,
            r: Reg,
            f: &AstNode,
            args: &Vec<AstNode>,
        ) -> Result<&mut Self, error::Error> {
            let argc = args.len().try_into().unwrap();
            self.seg_mut().slots = std::cmp::max(self.seg().slots(), r + argc);
            self.compile_expr(r, f)?;

            args.iter().enumerate().try_for_each(|(i, e)| {
                self.compile_expr(r + (Reg::try_from(i).unwrap()) + 1, e)
                    .map(|_| ())
            })?;

            Ok(self.with(Ins::Call(r, r, argc)))
        }

        fn compile_literal(&mut self, r: Reg, l: &AstNode) -> Result<&mut Self, error::Error> {
            Ok(match l.ast() {
                Ast::Null => self.with(Ins::LoadN(r)),
                Ast::Bool(b) => self.with(Ins::LoadB(r, *b)),
                Ast::Int(i) => {
                    let k = self.seg_mut().storek(vm::Value::Int(*i));
                    self.with(Ins::LoadK(r, k))
                }
                Ast::Float(f) => {
                    let k = self.seg_mut().storek(vm::Value::Float(*f));
                    self.with(Ins::LoadK(r, k))
                }
                Ast::String(s) => {
                    let k = self.seg_mut().storek(vm::Value::String(s.to_string()));
                    self.with(Ins::LoadK(r, k))
                }
                _ => unreachable!(),
            })
        }

        fn compile_id(
            &mut self,
            r0: Reg,
            id: &String,
            pos: io::Pos,
        ) -> Result<&mut Self, error::Error> {
            match self.find_reference(self.current_segment, id) {
                Some((r1, _, true)) => Ok(self.with(Ins::LoadG(r0, r1))),
                Some((r1, true, _)) => Ok(self.with(Ins::LoadU(r0, r1))),
                Some((r1, false, _)) => Ok(self.with(Ins::Move(r0, r1))),
                None => Err(error::Error::unknown_var_name(id.to_string(), pos)),
            }
        }

        fn find_reference(&mut self, segment: usize, id: &String) -> Option<(Reg, bool, bool)> {
            let seg = &self.segments[segment];

            if let Some(r) = seg.get_symbol(id) {
                return Some((r, false, seg.global()));
            }

            if let Some(r) = seg.get_upval(id) {
                return Some((r, true, seg.global()));
            }

            seg.parent.and_then(|parent| {
                self.find_reference(parent, id).map(|(r, _, global)| {
                    if global {
                        (r, false, true)
                    } else {
                        let upval = self.segments[segment].new_upval(id.to_string()).unwrap();
                        (upval, true, false)
                    }
                })
            })
        }
    }

    impl lexer::Op {
        pub fn to_ins(&self, r0: Reg, r1: Reg, r2: Reg) -> Ins {
            match self {
                Op::Add => Ins::Add(r0, r1, r2),
                Op::Sub => Ins::Sub(r0, r1, r2),
                Op::Mul => Ins::Mul(r0, r1, r2),
                Op::Div => Ins::Div(r0, r1, r2),
                Op::Mod => Ins::Mod(r0, r1, r2),
                Op::Eq => Ins::Eq(r0, r1, r2),
                Op::Neq => Ins::Neq(r0, r1, r2),
                Op::Le => Ins::Le(r0, r1, r2),
                Op::Ge => Ins::Lt(r0, r2, r1),
                Op::Lt => Ins::Lt(r0, r1, r2),
                Op::Gt => Ins::Le(r0, r2, r1),
                Op::Shr => Ins::Shl(r0, r2, r1),
                Op::Shl => Ins::Shl(r0, r1, r2),
                Op::BitOr => Ins::BitOr(r0, r1, r2),
                Op::BitXor => Ins::BitXor(r0, r1, r2),
                Op::BitAnd => Ins::BitAnd(r0, r1, r2),
                Op::AddEq => Ins::Add(r0, r1, r2),
                Op::SubEq => Ins::Sub(r0, r1, r2),
                Op::MulEq => Ins::Mul(r0, r1, r2),
                Op::DivEq => Ins::Div(r0, r1, r2),
                Op::ModEq => Ins::Mod(r0, r1, r2),
                Op::Or | Op::And | Op::Not | Op::BitNot | Op::Assign => unreachable!(),
            }
        }
    }
}
