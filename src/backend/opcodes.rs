use crate::frontend::operator::Op;

pub type Reg = u16;

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
    Shr(Reg, Reg, Reg),
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
    ObjIns(Reg, Reg, Reg),
    ObjGet(Reg, Reg, Reg),
    ObjNew(Reg),
    ArrNew(Reg, Reg),
    Import(Reg),
}

impl Op {
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
            Op::Ge => Ins::Le(r0, r2, r1),
            Op::Lt => Ins::Lt(r0, r1, r2),
            Op::Gt => Ins::Lt(r0, r2, r1),
            Op::Shr => Ins::Shr(r0, r1, r2),
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
