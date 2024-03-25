section .text
    global _start
_start:
    push rbp
    mov rbp, rsp
    mov rax, 10
	
    mov [rbp-4], rax
    mov rax, 5
	
    mov [rbp-8], rax
    
    push rax
    mov rax, 60
    pop rdi
    syscall
    pop rbp
    ret