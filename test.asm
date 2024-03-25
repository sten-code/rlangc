section .text
    global _start
_start:
    push rbp
    mov rbp, rsp
    mov rax, 10
	
    mov [rbp-4], rax
    mov rax, [rbp-4]
    mov [rbp-8], rax
    mov rax, [rbp-4]
    mov [rbp-12], rax
    mov rax, [rbp-8]
    mov [rbp-16], rax
    
    mov rdi, rax
    mov rax, 60
    syscall
    pop rbp
    ret