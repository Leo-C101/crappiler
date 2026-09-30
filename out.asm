section .data
    newline db 10
    string_0 db "Hello, World!"
    string_len_0 equ 13
    string_1 db "This is from my compiler!"
    string_len_1 equ 25
    string_2 db "This is a really inefficient way of compiling."
    string_len_2 equ 46
    string_3 db "I'm just going to use LLVM"
    string_len_3 equ 26
section .text
    global _start

_start:
    call fn_main
    mov edi, eax
    mov rax, 60
    syscall

fn_print_messages:
    push rbp
    mov rbp, rsp
    mov rax, 1
    mov rdi, 1
    mov rsi, string_0
    mov rdx, string_len_0
    syscall
    mov rax, 1
    mov rdi, 1
    mov rsi, newline
    mov rdx, 1
    syscall
    mov rax, 1
    mov rdi, 1
    mov rsi, string_1
    mov rdx, string_len_1
    syscall
    mov rax, 1
    mov rdi, 1
    mov rsi, newline
    mov rdx, 1
    syscall
    mov rax, 1
    mov rdi, 1
    mov rsi, string_2
    mov rdx, string_len_2
    syscall
    mov rax, 1
    mov rdi, 1
    mov rsi, newline
    mov rdx, 1
    syscall
    mov rax, 1
    mov rdi, 1
    mov rsi, string_3
    mov rdx, string_len_3
    syscall
    mov rax, 1
    mov rdi, 1
    mov rsi, newline
    mov rdx, 1
    syscall
    mov eax, 0
    mov rsp, rbp
    pop rbp
    ret

fn_main:
    push rbp
    mov rbp, rsp
    call fn_print_messages
    mov eax, 1
    mov rsp, rbp
    pop rbp
    ret
