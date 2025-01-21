# fibonacci numbers implemented in mips32 assembly

.text
.globl __start
__start:
    li $t0, 0
    li $t1, 1

loop:
    li $v0, 1
    move $a0, $t0
    syscall

    li $v0, 11
    li $a0, '\n'
    syscall

    add $t2, $t0, $t1
    move $t0, $t1
    move $t1, $t2

    # slti $s0, $t0, 1000
    # bne $s0, $zero, loop
    j loop

    li $v0, 10
    syscall
