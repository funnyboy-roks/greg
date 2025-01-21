# https://en.wikipedia.org/wiki/Collatz_conjecture

.globl __start
__start:
    li $t0, 840

loop:
    andi $t1, $t0, 1
    bne $t1, $zero, odd
# even
    sra $t0, $t0, 1 # $t0 >>= 1
    j done
odd:
    li $s0, 3
    mult $t0, $s0
    mflo $t0
    addi $t0, $t0, 1
done:
    li $v0, 1
    move $a0, $t0
    syscall

    li $v0, 11
    li $a0, '\n'
    syscall

    li $s0, 1
    bne $t0, $s0, loop

    li $v0, 10
    syscall
