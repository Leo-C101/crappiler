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
    InvalidFunction {
        message: String,
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
            Self::InvalidFunction { message } => message.clone(),
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

#[cfg(test)]
mod tests {
    use super::{CrapError, compile};

    #[test]
    fn compiles_i32_function_calls() {
        let asm = compile(
            "fn add(a: i32, b: i32) -> i32 { return a + b; } fn main() -> i32 { return add(19, 23); }"
                .to_string(),
        )
        .unwrap();

        assert!(asm.contains("call fn_main"));
        assert!(asm.contains("call fn_add"));
        assert!(asm.contains("mov eax, dword [rbp + 24]"));
        assert!(asm.contains("mov eax, dword [rbp + 16]"));
        assert!(asm.contains("add eax, ecx"));
        assert!(asm.contains("mov edi, eax"));
    }

    #[test]
    fn requires_main_function() {
        let result = compile("fn helper() -> i32 { return 0; }".to_string());

        assert!(matches!(result, Err(CrapError::InvalidFunction { .. })));
    }

    #[test]
    fn compiles_input_sample() {
        assert!(compile(include_str!("../../input.crap").to_string()).is_ok());
    }
}
