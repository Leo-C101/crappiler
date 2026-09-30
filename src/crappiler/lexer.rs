use crate::crappiler::CrapError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Number(i32),
    String(String),
    Identifier(String),

    Exit,
    Print,
    Println,
    Fn,
    Return,
    I32,

    Plus,
    Minus,
    Star,
    Slash,

    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Arrow,
    Semicolon,

    EOF,
}

const KEYWORDS: [(&str, TokenKind); 6] = [
    ("exit", TokenKind::Exit),
    ("print", TokenKind::Print),
    ("println", TokenKind::Println),
    ("fn", TokenKind::Fn),
    ("return", TokenKind::Return),
    ("i32", TokenKind::I32),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub ln: usize,
    pub col: usize,
}

impl Token {
    pub fn new(kind: TokenKind, ln: usize, col: usize) -> Self {
        Self { kind, ln, col }
    }
}

struct Lexer {
    src: String,
    tokens: Vec<Token>,
    pos: usize,
    ln: usize,
    col: usize,
}

impl Lexer {
    pub fn new(src: String) -> Self {
        Self {
            src,
            tokens: Vec::new(),
            pos: 0,
            ln: 1,
            col: 1,
        }
    }

    pub fn lex(mut self) -> Result<Vec<Token>, CrapError> {
        while !self.at_end() {
            if self.current().is_ascii_whitespace() {
                self.advance();
                continue;
            }

            if self.current().is_ascii_digit() {
                self.lex_number()?;
                continue;
            }

            if self.current() == '"' {
                self.lex_string()?;
                continue;
            }

            if self.current().is_ascii_alphabetic() || self.current() == '_' {
                self.lex_identifier();
                continue;
            }

            match self.current() {
                '+' => {
                    self.push_token(TokenKind::Plus);
                }
                '-' if self.src[self.pos..].starts_with("->") => {
                    self.push_token(TokenKind::Arrow);
                    self.advance();
                }
                '-' => {
                    self.push_token(TokenKind::Minus);
                }
                '*' => {
                    self.push_token(TokenKind::Star);
                }
                '/' => {
                    self.push_token(TokenKind::Slash);
                }
                '(' => {
                    self.push_token(TokenKind::LParen);
                }
                ')' => {
                    self.push_token(TokenKind::RParen);
                }
                '{' => {
                    self.push_token(TokenKind::LBrace);
                }
                '}' => {
                    self.push_token(TokenKind::RBrace);
                }
                ',' => {
                    self.push_token(TokenKind::Comma);
                }
                ':' => {
                    self.push_token(TokenKind::Colon);
                }
                ';' => {
                    self.push_token(TokenKind::Semicolon);
                }
                _ => {
                    return Err(CrapError::UnknownCharacter {
                        ln: self.ln,
                        col: self.col,
                        ch: self.current(),
                    });
                }
            }
        }

        self.tokens
            .push(Token::new(TokenKind::EOF, self.ln, self.col));
        Ok(self.tokens)
    }

    fn push_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token::new(kind, self.ln, self.col));
        self.advance();
    }

    fn lex_number(&mut self) -> Result<(), CrapError> {
        let (start_ln, start_col) = (self.ln, self.col);
        let mut num_str = String::new();

        while !self.at_end() && self.current().is_ascii_digit() {
            num_str.push(self.current());
            self.advance();
        }

        let num = match num_str.parse() {
            Ok(n) => n,
            Err(_) => {
                return Err(CrapError::ImproperNumber {
                    ln: start_ln,
                    col: start_col,
                    literal: num_str,
                });
            }
        };

        self.tokens
            .push(Token::new(TokenKind::Number(num), start_ln, start_col));
        Ok(())
    }

    fn lex_string(&mut self) -> Result<(), CrapError> {
        let (start_ln, start_col) = (self.ln, self.col);
        let mut str_lit = String::new();

        self.advance();
        while !self.at_end() && self.current() != '"' {
            str_lit.push(self.current());
            self.advance();
        }
        self.advance();

        if self.at_end() {
            return Err(CrapError::UnterminatedString {
                ln: start_ln,
                col: start_col,
            });
        }

        self.tokens
            .push(Token::new(TokenKind::String(str_lit), start_ln, start_col));
        Ok(())
    }

    fn lex_identifier(&mut self) {
        let (start_ln, start_col) = (self.ln, self.col);
        let mut ident = String::new();

        while !self.at_end() && (self.current().is_ascii_alphanumeric() || self.current() == '_') {
            ident.push(self.current());
            self.advance();
        }

        if let Some((_, kw)) = KEYWORDS.iter().find(|(name, _)| *name == ident) {
            self.tokens
                .push(Token::new(kw.clone(), start_ln, start_col));
        } else {
            self.tokens.push(Token::new(
                TokenKind::Identifier(ident),
                start_ln,
                start_col,
            ));
        }
    }

    fn advance(&mut self) {
        if !self.at_end() {
            let last = self.current();
            self.pos += 1;
            if last == '\n' {
                self.ln += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
    }

    fn current(&self) -> char {
        self.src.as_bytes()[self.pos] as char
    }

    fn at_end(&self) -> bool {
        self.pos >= self.src.len()
    }
}

pub fn lex(src: String) -> Result<Vec<Token>, CrapError> {
    Lexer::new(src).lex()
}
