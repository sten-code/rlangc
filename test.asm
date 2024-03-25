section .text
    global _start
_start:
    push rbp
    mov rbp, rsp
    mov rax, 5
	
    mov [rbp-4], rax
    mov rax, 4
	
    mov [rbp-8], rax
    mov rax, 10
	
    mov [rbp-4], rax
    
    mov rdi, rax
    mov rax, 60
    syscall
    pop rbp
    ret