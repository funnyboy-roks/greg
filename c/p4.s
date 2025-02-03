.rodata:
dash: .asciiz " - "
.text
# $a0 - in/out
pow2:
    addi $t0, $a0, -1
    and $a0, $a0, $t0
    jr $ra

.macro do_pow2 n
    li $a0, \n

    li $v0, 1
    syscall

    bal pow2

    move $t0, $a0

    li $v0, 11
    li $a0, ' '
    syscall

    move $a0, $t0

    li $v0, 1
    syscall

    li $v0, 11
    li $a0, '\n'

    syscall
.endm

.globl __start
__start:
    do_pow2 0
    do_pow2 1
    do_pow2 2
    do_pow2 3
    do_pow2 4
    do_pow2 5
    do_pow2 6
    do_pow2 7
    do_pow2 8
    do_pow2 9
    do_pow2 10
    do_pow2 11
    do_pow2 12
    do_pow2 13
    do_pow2 14
    do_pow2 15
    do_pow2 16
    do_pow2 17
    do_pow2 18
    do_pow2 19
    do_pow2 20

    li $v0, 10
    syscall
