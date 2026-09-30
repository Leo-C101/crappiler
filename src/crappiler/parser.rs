use crate::crappiler::{
    CrapError,
    lexer::{Token, TokenKind},
};

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, CrapError> {
    Parser::new(tokens).parse()
}

#[derive(Debug, Clone)]
pub enum Expr {
    NumberExpr(i32),
    StringExpr(String),
    VariableExpr(String),
    BinaryExpr {
        l: Box<Expr>,
        r: Box<Expr>,
        op: char,
    },
    CallExpr {
        name: String,
        args: Vec<Expr>,
    },
    UnaryExpr {
        op: char,
        expr: Box<Expr>,
    },
}

impl Expr {
    pub fn is_binary_expr(&self) -> bool {
        matches!(self, Self::BinaryExpr { .. })
    }
}

#[derive(Debug)]
pub enum Stmt {
    PrintStmt {
        value: Expr,
        newline: bool,
    },
    ExitStmt {
        value: Expr,
    },
    ReturnStmt {
        value: Expr,
    },
    ExprStmt(Expr),
    FunctionDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
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
            if !self.match_kind(TokenKind::Fn) {
                return Err(CrapError::InvalidFunction {
                    message: "only function declarations are allowed at the top level".to_string(),
                });
            }
            stmts.push(self.parse_function()?);
        }

        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Stmt, CrapError> {
        match self.peek().kind {
            TokenKind::Print | TokenKind::Println => self.parse_print(),

            TokenKind::Exit => self.parse_exit(),

            TokenKind::Return => self.parse_return(),

            _ => {
                let expr = self.parse_expression()?;
                self.consume(TokenKind::Semicolon)?;
                Ok(Stmt::ExprStmt(expr))
            }
        }
    }

    fn parse_function(&mut self) -> Result<Stmt, CrapError> {
        self.pos += 1;
        let name = match self.consume_identifier()? {
            TokenKind::Identifier(name) => name,
            _ => unreachable!(),
        };

        self.consume(TokenKind::LParen)?;
        let mut params = Vec::new();
        if !self.match_kind(TokenKind::RParen) {
            loop {
                let param = match self.consume_identifier()? {
                    TokenKind::Identifier(name) => name,
                    _ => unreachable!(),
                };
                self.consume(TokenKind::Colon)?;
                self.consume(TokenKind::I32)?;
                params.push(param);

                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
                self.pos += 1;
            }
        }
        self.consume(TokenKind::RParen)?;
        self.consume(TokenKind::Arrow)?;
        self.consume(TokenKind::I32)?;
        self.consume(TokenKind::LBrace)?;

        let mut body = Vec::new();
        while !self.match_kind(TokenKind::RBrace) && !self.at_end() {
            let stmt = self.parse_statement()?;
            let is_return = matches!(stmt, Stmt::ReturnStmt { .. });
            body.push(stmt);
            if is_return && !self.match_kind(TokenKind::RBrace) {
                return Err(CrapError::InvalidFunction {
                    message: format!("return must be the final statement in function '{name}'"),
                });
            }
        }
        self.consume(TokenKind::RBrace)?;

        if !matches!(body.last(), Some(Stmt::ReturnStmt { .. })) {
            return Err(CrapError::InvalidFunction {
                message: format!("function '{name}' must end with a return statement"),
            });
        }

        Ok(Stmt::FunctionDecl { name, params, body })
    }

    fn parse_return(&mut self) -> Result<Stmt, CrapError> {
        self.pos += 1;
        let value = self.parse_expression()?;
        self.consume(TokenKind::Semicolon)?;
        Ok(Stmt::ReturnStmt { value })
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
                if self.match_kind(TokenKind::LParen) {
                    self.pos += 1;
                    let mut args = Vec::new();
                    if !self.match_kind(TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.match_kind(TokenKind::Comma) {
                                break;
                            }
                            self.pos += 1;
                        }
                    }
                    self.consume(TokenKind::RParen)?;
                    Ok(Expr::CallExpr { name: ident, args })
                } else {
                    Ok(Expr::VariableExpr(ident))
                }
            }
            TokenKind::Minus => {
                self.pos += 1;
                Ok(Expr::UnaryExpr {
                    op: '-',
                    expr: Box::new(self.parse_primary()?),
                })
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

    fn consume_identifier(&mut self) -> Result<TokenKind, CrapError> {
        match self.peek().kind.clone() {
            kind @ TokenKind::Identifier(_) => {
                self.pos += 1;
                Ok(kind)
            }
            _ => Err(CrapError::UnexpectedToken {
                expected: None,
                token: self.peek().clone(),
            }),
        }
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
