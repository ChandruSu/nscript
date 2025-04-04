use crate::{
    frontend::lexer,
    frontend::operator::Op,
    utils::io,
    vm::{Env, Value},
};

#[derive(Debug, PartialEq)]
pub enum ErrorType {
    IOError,
    NameError(String),
    SyntaxError,
    CompilerError,
    TypeError(&'static str),
    ArithmeticError(Value),
    ArgumentError(u32, u32),
    IndexError(u32),
    ValueError,
    CustomError,
}

#[derive(Debug)]
pub struct Error {
    pub msg: String,
    pub err_type: ErrorType,
    pub pos: Option<io::Pos>,
}

impl ErrorType {
    fn to_string(&self) -> &'static str {
        match self {
            ErrorType::IOError => "IO ERROR",
            ErrorType::NameError(_) => "NAME ERROR",
            ErrorType::SyntaxError => "SYNTAX ERROR",
            ErrorType::CompilerError => "COMPILER ERROR",
            ErrorType::TypeError(_) => "TYPE ERROR",
            ErrorType::ArithmeticError(_) => "ARITHMETIC ERROR",
            ErrorType::ArgumentError(_, _) => "ARGUMENT ERROR",
            ErrorType::IndexError(_) => "INDEX ERROR",
            ErrorType::ValueError => "VALUE ERROR",
            ErrorType::CustomError => "ERROR",
        }
    }
}

impl Error {
    pub fn err<O>(self) -> Result<O, Self> {
        Err(self)
    }

    pub fn with_pos(self, pos: Option<&io::Pos>) -> Self {
        Self {
            err_type: self.err_type,
            msg: self.msg,
            pos: pos.cloned(),
        }
    }

    pub fn invalid_token_char(c: char, pos: io::Pos) -> Self {
        Self {
            msg: format!("Invalid token reached starting with {}", c),
            err_type: ErrorType::SyntaxError,
            pos: Some(pos),
        }
    }

    pub fn invalid_escape_char(c: char, pos: io::Pos) -> Self {
        Self {
            msg: format!("Invalid escape character in string: '\\{}'", c),
            err_type: ErrorType::SyntaxError,
            pos: Some(pos),
        }
    }

    pub fn unexpected_token(tk0: &lexer::Tk, tk1: &lexer::Tk, pos: io::Pos) -> Self {
        Self {
            msg: format!(
                "Unexpected token reached: '{:?}', expected '{:?}'",
                tk0, tk1
            ),
            err_type: ErrorType::SyntaxError,
            pos: Some(pos),
        }
    }

    pub fn unexpected_token_any(tk0: &lexer::Tk, pos: io::Pos) -> Self {
        Self {
            msg: format!("Unexpected token reached: '{:?}'", tk0),
            err_type: ErrorType::SyntaxError,
            pos: Some(pos),
        }
    }

    pub fn id_expected(pos: io::Pos) -> Self {
        Self {
            msg: format!("Unexpected token, identifier or symbol expected"),
            err_type: ErrorType::SyntaxError,
            pos: Some(pos),
        }
    }

    pub fn file_read_error(file_path: &str) -> Self {
        Self {
            msg: format!("Cannot read file: '{}'", file_path),
            err_type: ErrorType::IOError,
            pos: None,
        }
    }

    pub fn invalid_string_parse_input(s: &str) -> Self {
        Self {
            msg: format!("Cannot parse string: '{}'", s),
            err_type: ErrorType::ValueError,
            pos: None,
        }
    }

    pub fn non_unary_op(op: Op, pos: io::Pos) -> Self {
        Self {
            msg: format!(
                "Incorrect operator found: '{}', expected valid unary operator",
                op
            ),
            err_type: ErrorType::SyntaxError,
            pos: Some(pos),
        }
    }

    pub fn non_assign_op(op: Op, pos: io::Pos) -> Self {
        Self {
            msg: format!(
                "Incorrect operator found: '{}', expected valid assignment operator",
                op
            ),
            err_type: ErrorType::SyntaxError,
            pos: Some(pos),
        }
    }

    pub fn invalid_ast_node(pos: io::Pos) -> Self {
        Self {
            msg: format!("Unexpected AST node at this position - cannot be compiled"),
            err_type: ErrorType::CompilerError,
            pos: Some(pos),
        }
    }

    pub fn invalid_return_position(pos: io::Pos) -> Self {
        Self {
            msg: format!("Return statement from invalid position"),
            err_type: ErrorType::SyntaxError,
            pos: Some(pos),
        }
    }

    pub fn invalid_continue_pos(pos: io::Pos) -> Self {
        Self {
            msg: format!("Continue statement outside of loop"),
            err_type: ErrorType::SyntaxError,
            pos: Some(pos),
        }
    }

    pub fn unknown_var_name(name: String, pos: io::Pos) -> Self {
        Self {
            msg: format!("Unknown variable referenced: '{}'", name),
            err_type: ErrorType::NameError(name),
            pos: Some(pos),
        }
    }

    pub fn duplicate_var_name(name: String, pos: io::Pos) -> Self {
        Self {
            msg: format!("Symbol name has already been used in scope: '{}'", name),
            err_type: ErrorType::NameError(name),
            pos: Some(pos),
        }
    }

    pub fn mutate_closure(name: String, pos: io::Pos) -> Self {
        Self {
            msg: format!(
                "Variable is not in accessible scope and cannot be mutated: '{}'",
                name
            ),
            err_type: ErrorType::NameError(name),
            pos: Some(pos),
        }
    }

    pub fn module_not_found(name: String) -> Self {
        Self {
            msg: format!("Module not found: '{}'", name),
            err_type: ErrorType::NameError(name),
            pos: None,
        }
    }

    pub fn unexpected_null() -> Self {
        Self {
            msg: format!("Recieved unexpected 'null' value"),
            err_type: ErrorType::TypeError("Null"),
            pos: None,
        }
    }

    pub fn type_error(t0: &Value, t1: &Value) -> Self {
        Self {
            msg: format!(
                "Unexpected type recieved: Expected {} Recieved {}",
                t0.type_name(),
                t1.type_name()
            ),
            err_type: ErrorType::TypeError(t1.type_name()),
            pos: None,
        }
    }

    pub fn type_error_any(t0: &Value) -> Self {
        Self {
            msg: format!("Unexpected type recieved: Recieved {}", t0.type_name()),
            err_type: ErrorType::TypeError(t0.type_name()),
            pos: None,
        }
    }

    pub fn unhashable_type(t0: &Value) -> Self {
        Self {
            msg: format!(
                "Unhashable type detected, cannot be used as map key: Recieved {}",
                t0.type_name(),
            ),
            err_type: ErrorType::TypeError(t0.type_name()),
            pos: None,
        }
    }

    pub fn op_type_mismatch_un(op: Op, t0: &Value) -> Self {
        Self {
            msg: format!("Cannot apply operation '{}' to type {}", op, t0.type_name(),),
            err_type: ErrorType::TypeError(t0.type_name()),
            pos: None,
        }
    }

    pub fn op_type_mismatch(op: Op, t0: &Value, t1: &Value) -> Self {
        Self {
            msg: format!(
                "Cannot apply operation '{}' between types {} and {}",
                op,
                t0.type_name(),
                t1.type_name()
            ),
            err_type: ErrorType::TypeError(t1.type_name()),
            pos: None,
        }
    }

    pub fn negative_shift(v: i64) -> Self {
        Self {
            msg: format!("Cannot apply bitwise shift operation using a signed integer",),
            err_type: ErrorType::ArithmeticError(Value::Int(v)),
            pos: None,
        }
    }

    pub fn zero_division() -> Self {
        Self {
            msg: format!("Zero division error"),
            err_type: ErrorType::ArithmeticError(Value::Int(0)),
            pos: None,
        }
    }

    pub fn uncallable_type(t0: &Value) -> Self {
        Self {
            msg: format!("Cannot call non-function value of type {}", t0.type_name()),
            err_type: ErrorType::TypeError(t0.type_name()),
            pos: None,
        }
    }

    pub fn argument_error(rec: u32, exp: u32) -> Self {
        Self {
            msg: format!(
                "Invalid number of arguments provided, recieved: {}, expected: {}",
                rec, exp
            ),
            err_type: ErrorType::ArgumentError(rec, exp),
            pos: None,
        }
    }

    pub fn array_length_error(len: u32) -> Self {
        Self {
            msg: format!("Invalid array length: {}", len,),
            err_type: ErrorType::IndexError(len),
            pos: None,
        }
    }

    pub fn array_index_error(idx: u32) -> Self {
        Self {
            msg: format!("Invalid index: {}", idx,),
            err_type: ErrorType::IndexError(idx),
            pos: None,
        }
    }

    pub fn custom_error(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
            err_type: ErrorType::CustomError,
            pos: None,
        }
    }

    pub fn dump_stack_trace(&self, env: &Env, pos0: io::Pos) {
        let mut trace = env.trace_pos();
        match trace.first() {
            Some(p) if *p == pos0 => {}
            _ => trace.insert(0, pos0),
        };

        trace.iter().for_each(|pos| {
            eprintln!(
                "In file, at {} on line {}, column {}\n    {: >4} | {}\n         {}'",
                env.sources.get_source(pos.src_id).unwrap().get_origin(),
                pos.line + 1,
                pos.column + 1,
                pos.line + 1,
                env.sources.get_line(pos).unwrap_or_default(),
                "-".repeat(pos.column as usize + 2)
            )
        });
    }

    pub fn dump_error(&self, env: &Env) {
        if let Some(pos) = self.pos {
            self.dump_stack_trace(env, pos);
        }

        eprint!("{}: {}", self.err_type.to_string(), self.msg);
        if let Some(pos) = self.pos {
            eprint!(
                " at {}:{}:{}",
                env.sources.get_source(pos.src_id).unwrap().get_origin(),
                pos.line + 1,
                pos.column + 1
            )
        }

        eprintln!("");
    }
}
