pub mod parser {
    use core::fmt;

    use colored::Colorize;

    use crate::{
        lexer::lexer::{self, Op, Tk},
        utils::{error, io},
    };

    pub enum Ast {
        Null,
        Int(i32),
        Float(f32),
        Bool(bool),
        String(String),
        Object(Vec<(AstNode, AstNode)>),
        Reference(String),
        Block(Vec<AstNode>),
        TernaryExp(Box<AstNode>, Box<AstNode>, Box<AstNode>),
        BinaryExp(lexer::Op, Box<AstNode>, Box<AstNode>),
        UnaryExp(lexer::Op, Box<AstNode>),
        Subscript(Box<AstNode>, Box<AstNode>),
        Call(Box<AstNode>, Vec<AstNode>),
        Deref(Box<AstNode>, String),
        Let(String, Box<AstNode>),
        Assign(lexer::Op, Box<AstNode>, Box<AstNode>),
        Return(Option<Box<AstNode>>),
        If(Box<AstNode>, Box<AstNode>, Option<Box<AstNode>>),
        While(Box<AstNode>, Box<AstNode>),
        FuncDef(Option<String>, Vec<String>, Box<AstNode>),
    }

    pub struct AstNode {
        ast: Ast,
        pos: io::Pos,
    }

    pub struct Parser<'a> {
        lexer: &'a mut lexer::Lexer<'a>,
    }

    impl AstNode {
        pub fn new(ast: Ast, pos: io::Pos) -> Self {
            Self { ast, pos }
        }

        pub fn ast(&self) -> &Ast {
            &self.ast
        }

        pub fn pos(&self) -> io::Pos {
            self.pos
        }

        pub fn print_tree(
            &self,
            f: &mut std::fmt::Formatter<'_>,
            mut stem: u128,
            level: usize,
            last: bool,
        ) -> std::fmt::Result {
            stem |= 1 << level;

            let mut l0 = stem;

            while l0 > 1 {
                if l0 & 0x1 == 1 {
                    write!(f, "  |")?;
                } else {
                    write!(f, "   ")?;
                }
                l0 >>= 1;
            }

            if last {
                write!(f, "  '--")?;
            } else {
                write!(f, "  |--")?;
            }

            if last {
                stem &= !(1 << level);
            }

            match &self.ast {
                Ast::Null => writeln!(f, "{}", "null".green()),
                Ast::Int(i) => writeln!(f, "{} {}", "int-literal".green(), *i),
                Ast::Float(ff) => writeln!(f, "{} {}", "float-literal".green(), *ff),
                Ast::Bool(b) => writeln!(f, "{} {}", "bool-literal".green(), *b),
                Ast::String(s) => writeln!(f, "{} {}", "string-literal".green(), *s),
                Ast::Reference(s) => writeln!(f, "{} {}", "reference".green(), *s),
                Ast::TernaryExp(a, b, c) => {
                    writeln!(f, "{}", "ternary-expression".green())?;
                    a.print_tree(f, stem, level + 1, false)?;
                    b.print_tree(f, stem, level + 1, false)?;
                    c.print_tree(f, stem, level + 1, true)
                }
                Ast::BinaryExp(op, a, b) => {
                    writeln!(f, "{} {:?}", "binary-expression".green(), op)?;
                    a.print_tree(f, stem, level + 1, false)?;
                    b.print_tree(f, stem, level + 1, true)
                }
                Ast::UnaryExp(op, a) => {
                    writeln!(f, "{} {:?}", "unary-expression".green(), op)?;
                    a.print_tree(f, stem, level + 1, true)
                }
                Ast::Subscript(a, b) => {
                    writeln!(f, "{}", "subscript".green())?;
                    a.print_tree(f, stem, level + 1, false)?;
                    b.print_tree(f, stem, level + 1, true)
                }
                Ast::Deref(a, b) => {
                    writeln!(f, "{} ->{}", "attribute-dereference".green(), b)?;
                    a.print_tree(f, stem, level + 1, true)
                }
                Ast::Let(a, b) => {
                    writeln!(f, "{} {}", "var-declaration".green(), a)?;
                    b.print_tree(f, stem, level + 1, true)
                }
                Ast::Assign(op, a, b) => {
                    writeln!(f, "{} {:?}", "var-assignment".green(), op)?;
                    a.print_tree(f, stem, level + 1, false)?;
                    b.print_tree(f, stem, level + 1, true)
                }
                Ast::Return(a) => {
                    if let Some(a) = a {
                        writeln!(f, "{}", "return-statement".green())?;
                        a.print_tree(f, stem, level + 1, true)
                    } else {
                        writeln!(f, "{}", "return-statement".green())
                    }
                }
                Ast::If(a, b, c) => {
                    writeln!(f, "{}", "if-statement".green())?;
                    a.print_tree(f, stem, level + 1, false)?;

                    if let Some(c) = c {
                        b.print_tree(f, stem, level + 1, false)?;
                        c.print_tree(f, stem, level + 1, true)
                    } else {
                        b.print_tree(f, stem, level + 1, true)
                    }
                }
                Ast::While(a, b) => {
                    writeln!(f, "{}", "while-loop".green())?;
                    a.print_tree(f, stem, level + 1, false)?;
                    b.print_tree(f, stem, level + 1, true)
                }
                Ast::FuncDef(a, args, b) => {
                    let v = a.clone().unwrap_or("<lambda>".to_string());
                    writeln!(f, "{} {}({})", "function".green(), v, args.join(", "))?;
                    b.print_tree(f, stem, level + 1, true)
                }
                Ast::Call(a, v) => {
                    writeln!(f, "{}", "function-call".green())?;
                    a.print_tree(f, stem, level + 1, v.len() == 0)?;

                    for (i, node) in v.iter().enumerate() {
                        node.print_tree(f, stem, level + 1, i == v.len() - 1)?
                    }

                    Ok(())
                }
                Ast::Block(v) => {
                    writeln!(f, "{}", "block".green())?;

                    for (i, node) in v.iter().enumerate() {
                        node.print_tree(f, stem, level + 1, i == v.len() - 1)?
                    }

                    Ok(())
                }
                Ast::Object(vec) => {
                    writeln!(f, "{}", "object-literal".green())?;
                    for (i, (k, v)) in vec.iter().enumerate() {
                        if i == vec.len() - 1 {
                            k.print_tree(f, stem, level + 1, true)?;
                            v.print_tree(f, stem, level + 2, true)?
                        } else {
                            k.print_tree(f, stem, level + 1, false)?;
                            v.print_tree(f, stem | (1 << (level + 1)), level + 2, true)?
                        }
                    }
                    Ok(())
                }
            }
        }
    }

    impl fmt::Display for AstNode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "Parse result - Abstract Syntax Tree:").and(self.print_tree(f, 0, 0, true))
        }
    }

    impl<'a> Parser<'a> {
        pub fn new(lexer: &'a mut lexer::Lexer<'a>) -> Self {
            Self { lexer }
        }

        fn head(&self) -> &lexer::Token {
            self.lexer.head_token()
        }

        // fn lookahead(&self) -> &lexer::Token {
        //     self.lexer.loohahead_token()
        // }

        fn consume(&mut self) -> Result<&lexer::Token, error::Error> {
            self.lexer.next_valid_token()?;
            Ok(self.lexer.prev_token())
        }

        fn consume_if(&mut self, tk: lexer::Tk) -> Result<bool, error::Error> {
            if self.lexer.head_token().tk == tk {
                self.consume().map(|_| true)
            } else {
                Ok(false)
            }
        }

        fn expect(&mut self, tk: lexer::Tk) -> Result<&lexer::Token, error::Error> {
            let head = self.lexer.head_token();

            if head.tk == tk {
                self.consume()
            } else {
                error::Error::unexpected_token(&head.tk, &tk, head.pos).err()
            }
        }

        fn expect_id(&mut self) -> Result<&String, error::Error> {
            let head = self.consume()?;

            if let Tk::Id(id) = &head.tk {
                Ok(id)
            } else {
                error::Error::id_expected(head.pos).err()
            }
        }

        pub fn parse(&mut self) -> Result<AstNode, error::Error> {
            self.lexer.next_valid_token()?;
            self.lexer.next_valid_token()?;
            self.parse_block()
        }

        fn parse_block(&mut self) -> Result<AstNode, error::Error> {
            let mut statements: Vec<AstNode> = Vec::new();

            while !matches!(self.head().tk, Tk::RightBrace | Tk::EOF) {
                statements.push(self.parse_statement()?);
            }

            let pos = statements
                .first()
                .map(|s| s.pos)
                .unwrap_or(self.lexer.cursor());

            Ok(AstNode::new(Ast::Block(statements), pos))
        }

        fn parse_scoped_block(&mut self) -> Result<AstNode, error::Error> {
            self.expect(Tk::LeftBrace)?;
            let block = self.parse_block()?;
            self.expect(Tk::RightBrace)?;
            Ok(block)
        }

        fn parse_statement(&mut self) -> Result<AstNode, error::Error> {
            match &self.head().tk {
                Tk::If => self.parse_if_stmt(),
                Tk::While => self.parse_loop(),
                Tk::Let => self.parse_let(),
                Tk::Return => self.parse_return(),
                Tk::Fun => self.parse_function(false),
                Tk::Id(_) => self.parse_assign_or_call(),
                tk => error::Error::unexpected_token_any(tk, self.head().pos).err(),
            }
        }

        fn parse_let(&mut self) -> Result<AstNode, error::Error> {
            let pos = self.expect(Tk::Let)?.pos;
            let id = self
                .consume()?
                .as_id()
                .map(|s| s.to_string())
                .ok_or(error::Error::id_expected(pos))?;

            self.expect(Tk::Operator(Op::Assign))?;
            let e = Box::new(self.parse_expression()?);
            self.expect(Tk::Semi)?;

            Ok(AstNode::new(Ast::Let(id, e), pos))
        }

        fn parse_assign_or_call(&mut self) -> Result<AstNode, error::Error> {
            let pos = self.head().pos;
            let id = self.parse_reference()?;

            let op = match &self.consume()?.tk {
                Tk::Operator(
                    op @ (Op::Assign | Op::AddEq | Op::SubEq | Op::MulEq | Op::ModEq | Op::DivEq),
                ) => Ok(*op),
                Tk::Operator(op) => error::Error::non_assign_op(*op, self.head().pos).err(),
                Tk::Semi => return Ok(id),
                tk => error::Error::unexpected_token_any(tk, pos).err(),
            }?;

            let e = Box::new(self.parse_expression()?);
            self.expect(Tk::Semi)?;

            Ok(AstNode::new(Ast::Assign(op, Box::new(id), e), pos))
        }

        fn parse_if_stmt(&mut self) -> Result<AstNode, error::Error> {
            let pos = self.expect(Tk::If)?.pos;

            let cond = Box::new(self.parse_expression()?);
            let block1 = Box::new(self.parse_scoped_block()?);
            let block2 = if let Tk::Else = self.head().tk {
                Some(Box::new(self.parse_else_stmts()?))
            } else {
                None
            };

            Ok(AstNode::new(Ast::If(cond, block1, block2), pos))
        }

        fn parse_else_stmts(&mut self) -> Result<AstNode, error::Error> {
            self.expect(Tk::Else)?;

            match &self.head().tk {
                Tk::If => self.parse_if_stmt(),
                Tk::LeftBrace => self.parse_scoped_block(),
                tk => error::Error::unexpected_token_any(tk, self.head().pos).err(),
            }
        }

        fn parse_loop(&mut self) -> Result<AstNode, error::Error> {
            let pos = self.expect(Tk::While)?.pos;
            let cond = Box::new(self.parse_expression()?);
            let block = Box::new(self.parse_scoped_block()?);
            Ok(AstNode::new(Ast::While(cond, block), pos))
        }

        fn parse_return(&mut self) -> Result<AstNode, error::Error> {
            let pos = self.expect(Tk::Return)?.pos;

            let e = if self.consume_if(Tk::Semi)? {
                None
            } else {
                let e1 = self.parse_expression()?;
                self.expect(Tk::Semi)?;
                Some(Box::new(e1))
            };

            Ok(AstNode::new(Ast::Return(e), pos))
        }

        pub fn parse_expression(&mut self) -> Result<AstNode, error::Error> {
            self.parse_ternary()
        }

        fn parse_ternary(&mut self) -> Result<AstNode, error::Error> {
            match &self.head().tk {
                Tk::If => {
                    let pos = self.consume()?.pos;
                    let ex0 = self.parse_expression()?;
                    self.expect(Tk::LeftBrace)?;
                    let ex1 = self.parse_expression()?;
                    self.expect(Tk::RightBrace)?;
                    self.expect(Tk::Else)?;
                    self.expect(Tk::LeftBrace)?;
                    let ex2 = self.parse_expression()?;
                    self.expect(Tk::RightBrace)?;

                    Ok(AstNode::new(
                        Ast::TernaryExp(Box::new(ex0), Box::new(ex1), Box::new(ex2)),
                        pos,
                    ))
                }
                _ => self.parse_binary(lexer::MAX_OP_PRECEDENCE),
            }
        }

        fn parse_binary(&mut self, prec: u8) -> Result<AstNode, error::Error> {
            if prec < 1 {
                return self.parse_unary();
            }

            let mut lhs = self.parse_binary(prec - 1)?;

            while let Tk::Operator(op) = self.head().tk {
                if op.precedence() != prec {
                    break;
                }

                self.consume()?;

                lhs = AstNode {
                    pos: lhs.pos,
                    ast: Ast::BinaryExp(op, Box::new(lhs), Box::new(self.parse_binary(prec - 1)?)),
                }
            }

            Ok(lhs)
        }

        fn parse_unary(&mut self) -> Result<AstNode, error::Error> {
            match self.head().tk {
                Tk::Operator(op @ (Op::Sub | Op::Not | Op::BitNot)) => {
                    let pos = self.consume()?.pos;

                    Ok(AstNode::new(
                        Ast::UnaryExp(op, Box::new(self.parse_unary()?)),
                        pos,
                    ))
                }
                Tk::Operator(Op::Add) => {
                    self.consume()?;
                    self.parse_unary()
                }
                Tk::Operator(op) => error::Error::non_unary_op(op, self.head().pos).err(),
                _ => self.parse_term(),
            }
        }

        fn parse_term(&mut self) -> Result<AstNode, error::Error> {
            match &self.head().tk {
                Tk::Null => Ok(AstNode::new(Ast::Null, self.consume()?.pos)),
                Tk::Int(i) => Ok(AstNode::new(Ast::Int(*i), self.consume()?.pos)),
                Tk::Bool(b) => Ok(AstNode::new(Ast::Bool(*b), self.consume()?.pos)),
                Tk::Float(f) => Ok(AstNode::new(Ast::Float(*f), self.consume()?.pos)),
                Tk::String(s) => Ok(AstNode::new(Ast::String(s.clone()), self.consume()?.pos)),
                Tk::If => self.parse_ternary(),
                Tk::Fun => self.parse_function(true),
                Tk::Id(_) => self.parse_reference(),
                Tk::LeftBrace => self.parse_object(),
                Tk::LeftParen => {
                    self.consume()?;
                    let node = self.parse_expression()?;
                    self.expect(Tk::RightParen)?;
                    Ok(node)
                }
                tk => error::Error::unexpected_token_any(tk, self.head().pos).err(),
            }
        }

        fn parse_reference(&mut self) -> Result<AstNode, error::Error> {
            let pos = self.head().pos;

            let mut lhs = self
                .consume()?
                .as_id()
                .map(|s| AstNode::new(Ast::Reference(s.to_string()), pos))
                .ok_or(error::Error::id_expected(pos))?;

            while let nt @ (Tk::LeftParen | Tk::LeftBracket | Tk::Dot) = &self.head().tk {
                match nt {
                    Tk::LeftParen => {
                        let pos = self.consume()?.pos;
                        lhs = AstNode::new(Ast::Call(Box::new(lhs), self.parse_exprs()?), pos);
                        self.expect(Tk::RightParen)?;
                    }
                    Tk::LeftBracket => {
                        self.consume()?;
                        lhs = AstNode::new(
                            Ast::Subscript(Box::new(lhs), Box::new(self.parse_expression()?)),
                            pos,
                        );
                        self.expect(Tk::RightBracket)?;
                    }
                    Tk::Dot => {
                        self.consume()?;
                        let attr = self
                            .consume()?
                            .as_id()
                            .map(|s| s.to_string())
                            .ok_or(error::Error::id_expected(pos))?;
                        lhs = AstNode::new(Ast::Deref(Box::new(lhs), attr), pos)
                    }
                    _ => unreachable!(),
                }
            }

            Ok(lhs)
        }

        fn parse_exprs(&mut self) -> Result<Vec<AstNode>, error::Error> {
            let mut expressions: Vec<AstNode> = vec![];
            if !matches!(self.head().tk, Tk::RightParen) {
                expressions.push(self.parse_expression()?);

                while self.consume_if(Tk::Comma)? {
                    expressions.push(self.parse_expression()?);
                }
            }

            Ok(expressions)
        }

        fn parse_function(&mut self, lambda: bool) -> Result<AstNode, error::Error> {
            let pos = self.expect(Tk::Fun)?.pos;

            let id = if lambda {
                None
            } else {
                Some(self.expect_id()?.to_string())
            };

            self.expect(Tk::LeftParen)?;

            let mut args: Vec<String> = vec![];
            if !matches!(self.head().tk, Tk::RightParen) {
                args.push(self.expect_id()?.to_string());

                while self.consume_if(Tk::Comma)? {
                    args.push(self.expect_id()?.to_string());
                }
            }

            self.expect(Tk::RightParen)?;
            let block = Box::new(self.parse_scoped_block()?);

            Ok(AstNode::new(Ast::FuncDef(id, args, block), pos))
        }

        fn parse_object(&mut self) -> Result<AstNode, error::Error> {
            let pos = self.expect(Tk::LeftBrace)?.pos;
            let mut values = Vec::<(AstNode, AstNode)>::new();

            if !matches!(self.head().tk, Tk::RightBrace) {
                let key = self.parse_expression()?;
                self.expect(Tk::Colon)?;
                values.push((key, self.parse_expression()?));

                while self.consume_if(Tk::Comma)? {
                    let key = self.parse_expression()?;
                    self.expect(Tk::Colon)?;
                    values.push((key, self.parse_expression()?));
                }
            }

            self.expect(Tk::RightBrace)
                .map(|_| AstNode::new(Ast::Object(values), pos))
        }
    }
}
