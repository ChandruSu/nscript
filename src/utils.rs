pub mod io {
    use std::{fs, str::Chars};

    use super::error;

    type SourceId = u32;

    #[derive(Debug)]
    pub struct Source {
        id: SourceId,
        src_origin: String,
        src_content: String,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Pos {
        pub offset: i32,
        pub column: i32,
        pub line: u32,
        pub line_start: u32,
        pub src_id: SourceId,
    }

    pub struct SourceManager {
        sources: Vec<Source>,
    }

    impl Source {
        pub fn id(&self) -> u32 {
            self.id
        }

        pub fn char_stream(&self) -> Chars {
            self.src_content.chars()
        }

        pub fn get_origin(&self) -> &String {
            &self.src_origin
        }
    }

    impl SourceManager {
        pub fn new() -> Self {
            Self { sources: vec![] }
        }

        pub fn get_source(&self, id: u32) -> Option<&Source> {
            self.sources.get(id as usize)
        }

        pub fn load_source_file(&mut self, file_path: &str) -> Result<&Source, error::Error> {
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    self.sources.push(Source {
                        id: self.sources.len() as u32,
                        src_origin: file_path.to_string(),
                        src_content: content,
                    });

                    Ok(self.sources.last().unwrap())
                }
                Err(_) => Err(error::Error::file_read_error(file_path)),
            }
        }
    }
}

pub mod error {
    use crate::{lexer::lexer, vm::vm};

    use super::io;

    pub enum ErrorType {
        IOError,
        NameError(String),
        SyntaxError,
        CompilerError,
        TypeError(&'static str),
        ArithmeticError(vm::Value),
    }

    pub struct Error {
        pub msg: String,
        pub err_type: ErrorType,
        pub pos: Option<io::Pos>,
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

        pub fn non_unary_op(op: lexer::Op, pos: io::Pos) -> Self {
            Self {
                msg: format!(
                    "Incorrect operator found: '{}', expected valid unary operator",
                    op
                ),
                err_type: ErrorType::SyntaxError,
                pos: Some(pos),
            }
        }

        pub fn non_assign_op(op: lexer::Op, pos: io::Pos) -> Self {
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

        pub fn op_type_mismatch_un(op: lexer::Op, t0: &vm::Value) -> Self {
            Self {
                msg: format!("Cannot apply operation '{}' to type {}", op, t0.type_name(),),
                err_type: ErrorType::TypeError(t0.type_name()),
                pos: None,
            }
        }

        pub fn op_type_mismatch(op: lexer::Op, t0: &vm::Value, t1: &vm::Value) -> Self {
            Self {
                msg: format!(
                    "Cannot apply operation '{}' between types {} and {}",
                    op,
                    t0.type_name(),
                    t1.type_name()
                ),
                err_type: ErrorType::TypeError(t0.type_name()),
                pos: None,
            }
        }

        pub fn negative_shift(v: i32) -> Self {
            Self {
                msg: format!("Cannot apply bitwise shift operation using a signed integer",),
                err_type: ErrorType::ArithmeticError(vm::Value::Int(v)),
                pos: None,
            }
        }

        pub fn uncallable_type(t0: &vm::Value) -> Self {
            Self {
                msg: format!("Cannot call non-function value of type {}", t0.type_name()),
                err_type: ErrorType::TypeError(t0.type_name()),
                pos: None,
            }
        }

        pub fn dump_error(&self, sources: &io::SourceManager) {
            match &self.err_type {
                ErrorType::IOError => {
                    eprintln!("IO ERROR: {}", self.msg)
                }
                ErrorType::CompilerError => {
                    if let Some(pos) = self.pos {
                        eprintln!(
                            "COMPILER ERROR: {} at {}:{}:{}",
                            self.msg,
                            sources.get_source(pos.src_id).unwrap().get_origin(),
                            pos.line + 1,
                            pos.column + 1
                        )
                    }
                }
                ErrorType::SyntaxError => {
                    if let Some(pos) = self.pos {
                        eprintln!(
                            "SYNTAX ERROR: {} at {}:{}:{}",
                            self.msg,
                            sources.get_source(pos.src_id).unwrap().get_origin(),
                            pos.line + 1,
                            pos.column + 1
                        )
                    }
                }
                ErrorType::NameError(_) => {
                    if let Some(pos) = self.pos {
                        eprintln!(
                            "NAME ERROR: {} at {}:{}:{}",
                            self.msg,
                            sources.get_source(pos.src_id).unwrap().get_origin(),
                            pos.line + 1,
                            pos.column + 1
                        )
                    }
                }
                ErrorType::TypeError(_) => {
                    if let Some(pos) = self.pos {
                        eprintln!(
                            "TYPE ERROR: {} at {}:{}:{}",
                            self.msg,
                            sources.get_source(pos.src_id).unwrap().get_origin(),
                            pos.line + 1,
                            pos.column + 1
                        )
                    }
                }
                ErrorType::ArithmeticError(v) => {
                    if let Some(pos) = self.pos {
                        eprintln!(
                            "ARITHMETIC ERROR: {}, Value: {:?} at {}:{}:{}",
                            self.msg,
                            v,
                            sources.get_source(pos.src_id).unwrap().get_origin(),
                            pos.line + 1,
                            pos.column + 1
                        )
                    }
                }
            }
        }
    }
}
