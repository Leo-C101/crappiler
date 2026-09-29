use std::collections::HashMap;

use crate::crappiler::{
    CrapError,
    lexer::{Token, TokenKind},
    parser::{Expr, Stmt},
};

pub fn generate_code(tokens: Vec<Token>, ast: Vec<Stmt>) -> Result<String, CrapError> {
    let mut strings = Vec::new();

    for token in &tokens {
        match token.kind.clone() {
            TokenKind::String(s) => {
                strings.push(s);
            }
            _ => {}
        }
    }

    let mut string_map = HashMap::new();
    let mut string_len_map = HashMap::new();

    let mut asm = String::new();
    asm.push_str("section .data\n");
    asm.push_str("    newline db 10\n");

    for (index, string) in strings.iter().enumerate() {
        let str_name = format!("string_{index}");
        let str_len_name = format!("string_len_{index}");
        string_map.insert(string, str_name.clone());
        string_len_map.insert(string, str_len_name.clone());

        asm.push_str(format!("    {} db \"{}\"\n", str_name, string).as_str());
        asm.push_str(format!("    {} equ {}\n", str_len_name, string.len()).as_str());
    }

    asm.push_str("section .text\n");
    asm.push_str("    global _start\n\n");
    asm.push_str("_start:\n");

    for stmt in &ast {
        match stmt {
            Stmt::ExitStmt { value } => {
                asm.push_str("    mov rax, 60\n");

                match value {
                    Expr::NumberExpr(n) => {
                        asm.push_str(format!("    mov rdi, {}\n", n).as_str());
                    }
                    Expr::BinaryExpr { l, r, op } => {
                        // compile_bin(Expr::BinaryExpr { l, r, op })
                        todo!()
                    }
                    Expr::StringExpr(s) => {
                        return Err(CrapError::InvalidExitCode {
                            exit_code: s.clone(),
                        });
                    }
                    Expr::VariableExpr(name) => {
                        todo!()
                    }
                }
                asm.push_str("    syscall\n");
            }

            Stmt::PrintStmt { value, newline } => {
                let print_value = match value {
                    Expr::StringExpr(s) => s.clone(),
                    _ => todo!(),
                };

                asm.push_str("    mov rax, 1\n");
                asm.push_str("    mov rdi, 1\n");
                asm.push_str(
                    format!("    mov rsi, {}\n", string_map.get(&print_value).unwrap()).as_str(),
                );
                asm.push_str(
                    format!(
                        "    mov rdx, {}\n",
                        string_len_map.get(&print_value).unwrap()
                    )
                    .as_str(),
                );
                asm.push_str("    syscall\n");

                if *newline {
                    asm.push_str("    mov rax, 1\n");
                    asm.push_str("    mov rdi, 1\n");
                    asm.push_str(format!("    mov rsi, newline\n").as_str());
                    asm.push_str(format!("    mov rdx, 1\n").as_str());
                    asm.push_str("    syscall\n");
                }
            }

            Stmt::ExprStmt(_) => {}
        }
    }

    Ok(asm)
}

fn compile_bin(bin: Expr) -> Result<String, CrapError> {
    todo!()
}
