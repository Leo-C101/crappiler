use std::{fmt, path::PathBuf};

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

    println!("{tokens:#?}");

    Ok(String::new())
}
