use core::fmt;

pub static MAX_BIN_OP_PRECEDENCE: u8 = 11;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Le,
    Ge,
    Lt,
    Gt,
    Or,
    And,
    Not,
    Shr,
    Shl,
    Assign,
    AddEq,
    SubEq,
    MulEq,
    DivEq,
    ModEq,
    BitOr,
    BitXor,
    BitAnd,
    BitNot,
}

impl Op {
    pub fn precedence(&self) -> u8 {
        match self {
            Op::Or => 1,
            Op::And => 2,
            Op::BitOr => 3,
            Op::BitXor => 4,
            Op::BitAnd => 5,
            Op::Eq | Op::Neq => 6,
            Op::Gt | Op::Ge | Op::Lt | Op::Le => 7,
            Op::Shl | Op::Shr => 8,
            Op::Add | Op::Sub => 9,
            Op::Mul | Op::Div | Op::Mod => 10,
            Op::Not | Op::BitNot => 11,
            _ => MAX_BIN_OP_PRECEDENCE,
        }
    }

    pub fn op_str(&self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Mod => "%",
            Op::Eq => "==",
            Op::Neq => "!=",
            Op::Le => "<=",
            Op::Ge => ">=",
            Op::Lt => "<",
            Op::Gt => ">",
            Op::Or => "||",
            Op::And => "&&",
            Op::Not => "!",
            Op::Shr => ">>",
            Op::Shl => "<<",
            Op::Assign => "=",
            Op::AddEq => "+=",
            Op::SubEq => "-=",
            Op::MulEq => "*=",
            Op::DivEq => "/=",
            Op::ModEq => "%=",
            Op::BitOr => "|",
            Op::BitXor => "^",
            Op::BitAnd => "&",
            Op::BitNot => "~",
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.op_str())
    }
}
