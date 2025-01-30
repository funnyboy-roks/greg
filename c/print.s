.globl __start
.data
    s: .asciiz "hello world\n"
.text
# $a1 - pointer to a null-terminated string
print:
    lb $a0, 0($a1)
    beq $a0, $zero, done
    li $v0, 11
    syscall
    addi $a1, $a1, 1
    j print
done:
    jr $ra

__start:
    la $a1, s
    bal print
    li $v0, 10
    syscall
