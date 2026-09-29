use crate::crappiler::{
    CrapError,
    lexer::{Token, TokenKind},
};

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, CrapError> {
    Parser::new(tokens).parse()
}

#[derive(Debug)]
pub enum Expr {
    NumberExpr(i32),
    StringExpr(String),
    VariableExpr(String),
    BinaryExpr {
        l: Box<Expr>,
        r: Box<Expr>,
        op: char,
    },
}

impl Expr {
    pub fn is_binary_expr(&self) -> bool {
        matches!(self, Self::BinaryExpr { .. })
    }
}

#[derive(Debug)]
pub enum Stmt {
    PrintStmt { value: Expr, newline: bool },
    ExitStmt { value: Expr },
    ExprStmt(Expr),
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, CrapError> {
        let mut stmts = Vec::new();

        while !self.at_end() {
            stmts.push(self.parse_statement()?);
        }

        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Stmt, CrapError> {
        match self.peek().kind {
            TokenKind::Print | TokenKind::Println => self.parse_print(),

            TokenKind::Exit => self.parse_exit(),

            _ => Ok(Stmt::ExprStmt(self.parse_expression()?)),
        }
    }

    fn parse_print(&mut self) -> Result<Stmt, CrapError> {
        let newline = self.match_kind(TokenKind::Println);
        self.pos += 1;
        self.consume(TokenKind::LParen)?;
        let expr = self.parse_expression()?;
        self.consume(TokenKind::RParen)?;
        self.consume(TokenKind::Semicolon)?;

        Ok(Stmt::PrintStmt {
            value: expr,
            newline,
        })
    }

    fn parse_exit(&mut self) -> Result<Stmt, CrapError> {
        self.pos += 1; // consume exit
        self.consume(TokenKind::LParen)?;
        let expr = self.parse_expression()?;
        self.consume(TokenKind::RParen)?;
        self.consume(TokenKind::Semicolon)?;

        Ok(Stmt::ExitStmt { value: expr })
    }

    fn parse_expression(&mut self) -> Result<Expr, CrapError> {
        self.parse_term()
    }

    fn parse_term(&mut self) -> Result<Expr, CrapError> {
        let mut expr = Box::new(self.parse_factor()?);

        loop {
            if !self.match_kind(TokenKind::Plus) && !self.match_kind(TokenKind::Minus) {
                break;
            }

            let op = match self.peek().kind {
                TokenKind::Plus => '+',
                TokenKind::Minus => '-',
                _ => panic!(),
            };

            self.pos += 1;
            expr = Box::new(Expr::BinaryExpr {
                l: expr,
                r: Box::new(self.parse_factor()?),
                op,
            });
        }

        Ok(*expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, CrapError> {
        let mut expr = Box::new(self.parse_primary()?);

        loop {
            if !self.match_kind(TokenKind::Star) && !self.match_kind(TokenKind::Slash) {
                break;
            }

            let op = match self.peek().kind {
                TokenKind::Star => '*',
                TokenKind::Slash => '/',
                _ => panic!(),
            };

            self.pos += 1;
            expr = Box::new(Expr::BinaryExpr {
                l: expr,
                r: Box::new(self.parse_primary()?),
                op,
            });
        }

        Ok(*expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, CrapError> {
        match self.peek().kind.clone() {
            TokenKind::Identifier(ident) => {
                self.pos += 1;
                Ok(Expr::VariableExpr(ident))
            }
            TokenKind::LParen => {
                self.pos += 1;
                let expr = self.parse_expression()?;
                self.consume(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::String(s) => {
                self.pos += 1;
                Ok(Expr::StringExpr(s))
            }
            TokenKind::Number(i) => {
                self.pos += 1;
                Ok(Expr::NumberExpr(i))
            }
            _ => Err(CrapError::UnexpectedToken {
                expected: None,
                token: self.peek().clone(),
            }),
        }
    }

    fn at_end(&self) -> bool {
        self.match_kind(TokenKind::EOF)
    }

    fn consume(&mut self, kind: TokenKind) -> Result<&Token, CrapError> {
        self.expect_kind(kind)?;
        self.pos += 1;
        Ok(&self.tokens[self.pos - 1])
    }

    fn expect_kind(&self, kind: TokenKind) -> Result<(), CrapError> {
        if !self.match_kind(kind.clone()) {
            Err(CrapError::UnexpectedToken {
                expected: Some(kind),
                token: self.peek().clone(),
            })
        } else {
            Ok(())
        }
    }

    fn match_kind(&self, kind: TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(&kind)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
}
