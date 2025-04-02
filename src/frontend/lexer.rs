use std::str::Chars;

use crate::error;
use crate::utils::io;

use super::operator::Op;

#[derive(Debug, PartialEq)]
pub enum Tk {
    Null,
    Int(i64),
    Bool(bool),
    Float(f64),
    String(String),
    Id(String),
    Operator(Op),
    Comment,
    Let,
    Fun,
    If,
    Else,
    While,
    Return,
    Break,
    Continue,
    Import,
    EOF,
    Whitespace,
    Newline,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Semi,
    Comma,
    Dot,
    Colon,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub tk: Tk,
    pub pos: io::Pos,
}

pub struct Lexer<'a> {
    stream: Chars<'a>,
    current_char: char,
    lookahead_char: char,
    cursor: io::Pos,
    tki: usize,
    tks: [Token; 3],
}

impl Token {
    pub fn new(tk: Tk, pos: io::Pos) -> Self {
        Self { tk, pos }
    }

    pub fn as_id(&self) -> Option<&String> {
        match &self.tk {
            Tk::Id(id) => Some(&id),
            _ => None,
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a io::Source) -> Self {
        let mut stream = src.char_stream();
        let cursor = io::Pos {
            offset: -1,
            column: -1,
            line: 0,
            line_start: 0,
            src_id: src.id(),
        };

        Self {
            current_char: '\0',
            lookahead_char: stream.next().unwrap_or('\0'),
            stream,
            cursor,
            tki: 0,
            tks: [
                Token::new(Tk::EOF, cursor),
                Token::new(Tk::EOF, cursor),
                Token::new(Tk::EOF, cursor),
            ],
        }
    }

    pub fn cursor(&self) -> io::Pos {
        self.cursor
    }

    pub fn head_token(&self) -> &Token {
        &self.tks[(self.tki + 1) % 3]
    }

    pub fn prev_token(&self) -> &Token {
        &self.tks[(self.tki) % 3]
    }

    fn advance(&mut self) -> char {
        if self.current_char == '\n' {
            self.cursor.column = -1;
            self.cursor.line += 1;
            self.cursor.line_start = (self.cursor.offset as u32) + 1;
        }

        self.current_char = self.lookahead_char;
        self.lookahead_char = self.stream.next().unwrap_or('\0');

        self.cursor.column += 1;
        self.cursor.offset += 1;
        self.current_char
    }

    fn next_token(&mut self) -> Result<Token, error::Error> {
        let c = self.advance();
        let pos = self.cursor;

        let tk = match c {
            c if c.is_ascii_alphabetic() || c == '_' => self.extract_identifier(),
            c if c.is_digit(10) => self.extract_number(),
            '"' => self.extract_string()?,
            '#' => self.extract_comment(),
            '{' => Tk::LeftBrace,
            '}' => Tk::RightBrace,
            '(' => Tk::LeftParen,
            ')' => Tk::RightParen,
            '[' => Tk::LeftBracket,
            ']' => Tk::RightBracket,
            ';' => Tk::Semi,
            ':' => Tk::Colon,
            ',' => Tk::Comma,
            '.' => Tk::Dot,
            '\n' => Tk::Newline,
            '\0' => Tk::EOF,
            '\t' | '\r' | ' ' => {
                while let '\t' | '\r' | ' ' = self.lookahead_char {
                    self.advance();
                }
                Tk::Whitespace
            }
            c => match (c, self.lookahead_char) {
                ('+', '=') => {
                    self.advance();
                    Tk::Operator(Op::AddEq)
                }
                ('-', '=') => {
                    self.advance();
                    Tk::Operator(Op::SubEq)
                }
                ('*', '=') => {
                    self.advance();
                    Tk::Operator(Op::MulEq)
                }
                ('/', '=') => {
                    self.advance();
                    Tk::Operator(Op::DivEq)
                }
                ('%', '=') => {
                    self.advance();
                    Tk::Operator(Op::ModEq)
                }
                ('=', '=') => {
                    self.advance();
                    Tk::Operator(Op::Eq)
                }
                ('!', '=') => {
                    self.advance();
                    Tk::Operator(Op::Neq)
                }
                ('<', '=') => {
                    self.advance();
                    Tk::Operator(Op::Le)
                }
                ('>', '=') => {
                    self.advance();
                    Tk::Operator(Op::Ge)
                }
                ('>', '>') => {
                    self.advance();
                    Tk::Operator(Op::Shr)
                }
                ('<', '<') => {
                    self.advance();
                    Tk::Operator(Op::Shl)
                }
                ('|', '|') => {
                    self.advance();
                    Tk::Operator(Op::Or)
                }
                ('&', '&') => {
                    self.advance();
                    Tk::Operator(Op::And)
                }
                ('+', _) => Tk::Operator(Op::Add),
                ('-', _) => Tk::Operator(Op::Sub),
                ('*', _) => Tk::Operator(Op::Mul),
                ('/', _) => Tk::Operator(Op::Div),
                ('%', _) => Tk::Operator(Op::Mod),
                ('<', _) => Tk::Operator(Op::Lt),
                ('>', _) => Tk::Operator(Op::Gt),
                ('!', _) => Tk::Operator(Op::Not),
                ('|', _) => Tk::Operator(Op::BitOr),
                ('^', _) => Tk::Operator(Op::BitXor),
                ('&', _) => Tk::Operator(Op::BitAnd),
                ('~', _) => Tk::Operator(Op::BitNot),
                ('=', _) => Tk::Operator(Op::Assign),
                _ => return error::Error::invalid_token_char(c, pos).err(),
            },
        };

        Ok(Token::new(tk, pos))
    }

    fn extract_identifier(&mut self) -> Tk {
        let mut buf = self.current_char.to_string();

        while self.lookahead_char.is_alphanumeric() || self.lookahead_char == '_' {
            buf.push(self.advance());
        }

        match buf.as_str() {
            "let" => Tk::Let,
            "fun" => Tk::Fun,
            "if" => Tk::If,
            "else" => Tk::Else,
            "while" => Tk::While,
            "return" => Tk::Return,
            "true" => Tk::Bool(true),
            "false" => Tk::Bool(false),
            "null" => Tk::Null,
            "break" => Tk::Break,
            "continue" => Tk::Continue,
            "import" => Tk::Import,
            _ => Tk::Id(buf),
        }
    }

    fn extract_number(&mut self) -> Tk {
        let mut buf = self.current_char.to_string();
        let mut is_float = false;

        while self.lookahead_char.is_digit(10) || (self.lookahead_char == '.' && !is_float) {
            is_float = is_float || self.lookahead_char == '.';
            buf.push(self.advance());
        }

        if is_float {
            Tk::Float(buf.parse::<f64>().unwrap_or(0.0))
        } else {
            Tk::Int(buf.parse::<i64>().unwrap_or(0))
        }
    }

    fn extract_string(&mut self) -> Result<Tk, error::Error> {
        let mut buf = String::new();

        while self.lookahead_char != '"' && self.lookahead_char != '\0' {
            if self.lookahead_char == '\\' {
                self.advance();
                buf.push(match self.advance() {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '"' => '"',
                    '\\' => '\\',
                    c => return error::Error::invalid_escape_char(c, self.cursor).err(),
                });
            } else {
                buf.push(self.advance());
            }
        }

        self.advance();
        Ok(Tk::String(buf))
    }

    fn extract_comment(&mut self) -> Tk {
        while self.lookahead_char != '\n' && self.lookahead_char != '\0' {
            self.advance();
        }
        Tk::Comment
    }

    pub fn next_valid_token(&mut self) -> Result<&Token, error::Error> {
        let mut token = self.next_token();
        while let Ok(ref tk) = token {
            match tk.tk {
                Tk::Comment | Tk::Whitespace | Tk::Newline => token = self.next_token(),
                _ => break,
            }
        }

        token.map(|token| {
            self.tks[self.tki % 3] = token;
            self.tki += 1;
            &self.tks[(self.tki + 2) % 3]
        })
    }
}
