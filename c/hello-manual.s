.globl __start
__start:
    li $v0, 11
    li $a0, 'h'
    syscall
    li $a0, 'e'
    syscall
    li $a0, 'l'
    syscall
    li $a0, 'l'
    syscall
    li $a0, 'o'
    syscall
    li $a0, ' '
    syscall
    li $a0, 'w'
    syscall
    li $a0, 'o'
    syscall
    li $a0, 'r'
    syscall
    li $a0, 'l'
    syscall
    li $a0, 'd'
    syscall
    li $a0, '\n'
    syscall
    li $v0, 10
    syscall
