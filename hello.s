.section .data
msg:
    .ascii "Hello, World!\0"

.section .text
.globl main
main:
    # загружаем адрес строки в rdi, т.к. он первый аргумент для puts
    leaq    msg(%rip), %rdi
    call    puts
    # проверка на 0, puts вернёт 0, если всё прошло хорошо
    xorq    %rax, %rax
    ret
