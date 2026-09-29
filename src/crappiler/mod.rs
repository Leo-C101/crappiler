use std::{fmt, path::PathBuf};

use crate::crappiler::{
    lexer::{Token, TokenKind},
    parser::Expr,
};

pub mod codegen;
pub mod lexer;
pub mod parser;

#[derive(Debug)]
pub enum CrapError {
    InvalidInputFile {
        path: PathBuf,
    },
    UnknownCharacter {
        ln: usize,
        col: usize,
        ch: char,
    },
    ImproperNumber {
        ln: usize,
        col: usize,
        literal: String,
    },
    UnterminatedString {
        ln: usize,
        col: usize,
    },
    UnexpectedToken {
        expected: Option<TokenKind>,
        token: Token,
    },
    InvalidExitCode {
        exit_code: String,
    },
    UnexpectedExpr {
        expr: Expr,
    },
}

impl fmt::Display for CrapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = match self {
            Self::InvalidInputFile { path } => {
                format!("invalid input file: {}", path.display())
            }
            Self::UnknownCharacter { ln, col, ch } => {
                format!("unknown character '{ch}' at line {ln}, col {col}")
            }
            Self::ImproperNumber { ln, col, literal } => {
                format!("improper number literal '{literal}' at line {ln}, col {col}")
            }
            Self::UnterminatedString { ln, col } => {
                format!("unterminated string literal at line {ln}, col {col}")
            }
            Self::UnexpectedToken { expected, token } => {
                if let Some(expected) = expected {
                    format!(
                        "expected token of kind {expected:?} but got {:?} at line {}, col {}",
                        token.kind, token.ln, token.col
                    )
                } else {
                    format!("unexpected token: {:?}", token.kind)
                }
            }
            Self::InvalidExitCode { exit_code } => {
                format!("invalid exit code '{}'", exit_code)
            }
            Self::UnexpectedExpr { expr } => {
                format!("unexpected expression: {:?}", expr)
            }
        };

        write!(f, "{output}")
    }
}

impl std::error::Error for CrapError {
    fn description(&self) -> &str {
        format!("{}", self).leak()
    }
}

pub fn compile(src: String) -> Result<String, CrapError> {
    let tokens = lexer::lex(src)?;
    let ast = parser::parse(tokens.clone())?;
    let asm = codegen::generate_code(tokens, ast)?;

    Ok(asm)
}
