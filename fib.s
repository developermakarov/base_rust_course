.section .text
.globl main
main:
    # загружаем 12 в регистр rcx
    movq    $12, %rcx
    # загружаем 0 в rax(далее обработаем кейс с 0)
    movq    $0, %rax
    # тоже самое как и с 0, только 1
    movq    $1, %rdx
    # побитовое сравнение, без изменения регистра установит ZF=0 при rcx==0
    testq   %rcx, %rcx
    jz      .done
    # тоже самое как и с 0, только 1
    cmpq    $1, %rcx
    je      .done
    # счётчик цикла, начинаем с 2
    movq    $2, %rsi

.loop:
    # копируем сохраняем rdx(i-1) значение, чтобы не потерять при сложении в rdi
    movq    %rdx, %rdi
    # складываем i-2 c i-1 => rdx => rdx теперь i
    addq    %rax, %rdx
    # записываем значение из rdi в rax => rax i-1 для некст итерации
    movq    %rdi, %rax
    # инкремент счётчика
    incq    %rsi
    # сравнением i с целевым rcx
    cmpq    %rcx, %rsi
    # ну и обратно в цикл пока не равно 12
    jle     .loop

.done:
    # запишем результат в rax, чтобы вернуть значение
    movq    %rdx, %rax
    ret
