.section .text
.globl main
main:
    # т.к. exit code существует в диапозоне 0-255, то программа будет валидна для суммы от 0 до 255
    # загружаем два числа в регистры rax и rbx
    movq    $20, %rax
    movq    $15, %rbx
    # сохраняем результат сложения в rax
    addq    %rbx, %rax

    # exit
    ret
