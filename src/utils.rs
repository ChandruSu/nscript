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
    use crate::lexer::lexer;

    use super::io;

    pub enum ErrorType {
        IOError,
        SyntaxError(io::Pos),
    }

    pub struct Error {
        pub msg: String,
        pub err_type: ErrorType,
    }

    impl Error {
        pub fn invalid_token_char(c: char, pos: io::Pos) -> Self {
            Self {
                msg: format!("Invalid token reached starting with {}", c),
                err_type: ErrorType::SyntaxError(pos),
            }
        }

        pub fn unexpected_token(tk0: &lexer::Tk, tk1: &lexer::Tk, pos: io::Pos) -> Self {
            Self {
                msg: format!(
                    "Unexpected token reached: '{:?}', expected '{:?}'",
                    tk0, tk1
                ),
                err_type: ErrorType::SyntaxError(pos),
            }
        }

        pub fn unexpected_token_any(tk0: &lexer::Tk, pos: io::Pos) -> Self {
            Self {
                msg: format!("Unexpected token reached: '{:?}'", tk0),
                err_type: ErrorType::SyntaxError(pos),
            }
        }

        pub fn id_expected(pos: io::Pos) -> Self {
            Self {
                msg: format!("Unexpected token, identifier or symbol expected"),
                err_type: ErrorType::SyntaxError(pos),
            }
        }

        pub fn file_read_error(file_path: &str) -> Self {
            Self {
                msg: format!("Cannot read file: '{}'", file_path),
                err_type: ErrorType::IOError,
            }
        }

        pub fn non_unary_op(op: lexer::Op, pos: io::Pos) -> Self {
            Self {
                msg: format!(
                    "Incorrect operator found: '{:?}', expected valid unary operator",
                    op
                ),
                err_type: ErrorType::SyntaxError(pos),
            }
        }

        pub fn non_assign_op(op: lexer::Op, pos: io::Pos) -> Self {
            Self {
                msg: format!(
                    "Incorrect operator found: '{:?}', expected valid assignment operator",
                    op
                ),
                err_type: ErrorType::SyntaxError(pos),
            }
        }

        pub fn dump_error(&self, sources: &io::SourceManager) {
            match &self.err_type {
                ErrorType::IOError => {
                    eprintln!("IO ERROR: {}", self.msg)
                }
                ErrorType::SyntaxError(pos) => {
                    eprintln!(
                        "SYNTAX ERROR: {} at {}:{}:{}",
                        self.msg,
                        sources.get_source(pos.src_id).unwrap().get_origin(),
                        pos.line + 1,
                        pos.column + 1
                    )
                }
            }
        }
    }
}
