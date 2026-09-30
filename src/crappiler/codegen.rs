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

    let mut functions = HashMap::new();
    for stmt in &ast {
        if let Stmt::FunctionDecl { name, params, .. } = stmt {
            if functions.insert(name.clone(), params.len()).is_some() {
                return Err(CrapError::InvalidFunction {
                    message: format!("function '{name}' is declared more than once"),
                });
            }
        }
    }

    let main_param_count = functions
        .get("main")
        .ok_or_else(|| CrapError::InvalidFunction {
            message: "program must define a main function".to_string(),
        })?;
    if *main_param_count != 0 {
        return Err(CrapError::InvalidFunction {
            message: "main function must not take parameters".to_string(),
        });
    }

    for stmt in &ast {
        if !matches!(stmt, Stmt::FunctionDecl { .. }) {
            return Err(CrapError::InvalidFunction {
                message: "only function declarations are allowed at the top level".to_string(),
            });
        }
    }

    asm.push_str("    call fn_main\n");
    asm.push_str("    mov edi, eax\n");
    asm.push_str("    mov rax, 60\n");
    asm.push_str("    syscall\n");

    for stmt in &ast {
        if let Stmt::FunctionDecl { name, params, body } = stmt {
            let param_indexes = params
                .iter()
                .enumerate()
                .map(|(index, param)| (param.clone(), index))
                .collect::<HashMap<_, _>>();

            asm.push_str(format!("\nfn_{name}:\n").as_str());
            asm.push_str("    push rbp\n");
            asm.push_str("    mov rbp, rsp\n");
            for stmt in body {
                match stmt {
                    Stmt::PrintStmt { value, newline } => {
                        compile_print(value, *newline, &mut asm, &string_map, &string_len_map)?
                    }
                    Stmt::ReturnStmt { value } => {
                        compile_expr(value, &mut asm, &param_indexes, &functions)?;
                        asm.push_str("    mov rsp, rbp\n");
                        asm.push_str("    pop rbp\n");
                        asm.push_str("    ret\n");
                    }
                    Stmt::ExprStmt(expr) => {
                        compile_expr(expr, &mut asm, &param_indexes, &functions)?;
                    }
                    Stmt::ExitStmt { value } => {
                        if let Expr::StringExpr(exit_code) = value {
                            return Err(CrapError::InvalidExitCode {
                                exit_code: exit_code.clone(),
                            });
                        }
                        compile_expr(value, &mut asm, &param_indexes, &functions)?;
                        asm.push_str("    mov edi, eax\n");
                        asm.push_str("    mov rax, 60\n");
                        asm.push_str("    syscall\n");
                    }
                    Stmt::FunctionDecl { .. } => unreachable!(),
                }
            }
        }
    }

    Ok(asm)
}

fn compile_print(
    value: &Expr,
    newline: bool,
    asm: &mut String,
    string_map: &HashMap<&String, String>,
    string_len_map: &HashMap<&String, String>,
) -> Result<(), CrapError> {
    let print_value = match value {
        Expr::StringExpr(s) => s,
        _ => {
            return Err(CrapError::UnexpectedExpr {
                expr: value.clone(),
            });
        }
    };

    let string_label = string_map
        .get(print_value)
        .ok_or_else(|| CrapError::InvalidFunction {
            message: "print string was not collected during code generation".to_string(),
        })?;
    let length_label =
        string_len_map
            .get(print_value)
            .ok_or_else(|| CrapError::InvalidFunction {
                message: "print string length was not collected during code generation".to_string(),
            })?;

    asm.push_str("    mov rax, 1\n");
    asm.push_str("    mov rdi, 1\n");
    asm.push_str(format!("    mov rsi, {string_label}\n").as_str());
    asm.push_str(format!("    mov rdx, {length_label}\n").as_str());
    asm.push_str("    syscall\n");

    if newline {
        asm.push_str("    mov rax, 1\n");
        asm.push_str("    mov rdi, 1\n");
        asm.push_str("    mov rsi, newline\n");
        asm.push_str("    mov rdx, 1\n");
        asm.push_str("    syscall\n");
    }

    Ok(())
}

fn compile_expr(
    expr: &Expr,
    asm: &mut String,
    params: &HashMap<String, usize>,
    functions: &HashMap<String, usize>,
) -> Result<(), CrapError> {
    match expr {
        Expr::NumberExpr(number) => {
            asm.push_str(format!("    mov eax, {number}\n").as_str());
        }
        Expr::VariableExpr(name) => {
            let index = params.get(name).ok_or_else(|| CrapError::UnexpectedExpr {
                expr: Expr::VariableExpr(name.clone()),
            })?;
            let function_param_count = params.len();
            let offset = 16 + 8 * (function_param_count - index - 1);
            asm.push_str(format!("    mov eax, dword [rbp + {offset}]\n").as_str());
        }
        Expr::UnaryExpr { op, expr } => {
            compile_expr(expr, asm, params, functions)?;
            match op {
                '-' => asm.push_str("    neg eax\n"),
                _ => unreachable!(),
            }
        }
        Expr::BinaryExpr { l, r, op } => {
            compile_expr(l, asm, params, functions)?;
            asm.push_str("    push rax\n");
            compile_expr(r, asm, params, functions)?;
            asm.push_str("    mov ecx, eax\n");
            asm.push_str("    pop rax\n");
            match op {
                '+' => asm.push_str("    add eax, ecx\n"),
                '-' => asm.push_str("    sub eax, ecx\n"),
                '*' => asm.push_str("    imul eax, ecx\n"),
                '/' => {
                    asm.push_str("    cdq\n");
                    asm.push_str("    idiv ecx\n");
                }
                _ => unreachable!(),
            }
        }
        Expr::CallExpr { name, args } => {
            let expected = functions
                .get(name)
                .ok_or_else(|| CrapError::InvalidFunction {
                    message: format!("function '{name}' is not declared"),
                })?;
            if *expected != args.len() {
                return Err(CrapError::InvalidFunction {
                    message: format!(
                        "function '{name}' expects {expected} arguments but got {}",
                        args.len()
                    ),
                });
            }
            for arg in args {
                compile_expr(arg, asm, params, functions)?;
                asm.push_str("    push rax\n");
            }
            asm.push_str(format!("    call fn_{name}\n").as_str());
            if !args.is_empty() {
                asm.push_str(format!("    add rsp, {}\n", args.len() * 8).as_str());
            }
        }
        Expr::StringExpr(value) => {
            return Err(CrapError::UnexpectedExpr {
                expr: Expr::StringExpr(value.clone()),
            });
        }
    }

    Ok(())
}
