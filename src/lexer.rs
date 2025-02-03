pub mod lexer {
    use core::fmt;
    use std::str::Chars;

    use crate::error;
    use crate::utils::io;
    pub static MAX_OP_PRECEDENCE: u8 = 10;

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

    #[derive(Debug, PartialEq)]
    pub enum Tk {
        Null,
        Int(i32),
        Bool(bool),
        Float(f32),
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

        pub fn loohahead_token(&self) -> &Token {
            &self.tks[(self.tki + 2) % 3]
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
                '"' => self.extract_string(),
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
                Tk::Float(buf.parse::<f32>().unwrap_or(0.0))
            } else {
                Tk::Int(buf.parse::<i32>().unwrap_or(0))
            }
        }

        fn extract_string(&mut self) -> Tk {
            let mut buf = String::new();

            while self.lookahead_char != '"' && self.lookahead_char != '\0' {
                buf.push(self.advance());
            }

            self.advance();
            Tk::String(buf)
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

    impl Op {
        pub fn precedence(&self) -> u8 {
            match self {
                Op::Or => 10,
                Op::And => 9,
                Op::BitOr => 8,
                Op::BitXor => 7,
                Op::BitAnd => 6,
                Op::Eq | Op::Neq => 5,
                Op::Gt | Op::Ge | Op::Lt | Op::Le => 4,
                Op::Shl | Op::Shr => 3,
                Op::Add | Op::Sub => 2,
                Op::Mul | Op::Div | Op::Mod => 1,
                Op::Not | Op::BitNot => 0,
                _ => MAX_OP_PRECEDENCE,
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
}
