.globl __start
# $a0 - bottom 32 bits
# $a1 - top 32 bits
# print:
#     li $t0, 0b1111000000000000
#     li $t1, 12      # i = 12
# .loop:
#     srav $t0, $a0, $t1
#     andi $t0, $t0, 0xf
# 
#     addi $t1, $t1, -4
#     j .loop
# .end:
#     ja $ra

# t0 = a0
# t3 = sp
# *t3 = '\0'
# t3 -= 1;
# sp -= 21
# while (t3 > sp) {
#     t1 = t0 & 0xf;
#     putchar(hex(t1));
#     t0 >>= 4;
#     t3 += 1;
# }
#
print_hex:
    slti $t0, $t1, 10
    beq $t0, $zero, .alpha
    addi $a0, $t1, '0'
    j .hex_done
    .alpha:
    addi $t1, $t1, -10
    addi $a0, $t1, 'a'
    .hex_done:
    li $v0, 11
    syscall
    jr $ra
print:
    move $t8, $sp

    move $t0, $a0 # t0 = a0

    move $t3, $sp # t3 = sp
    addi $sp, $sp, -21 # allocate 21 bytes on the stack
    sw $zero, 0($t3) # *t3 = 0
    addi $t3, $t3, -1 # t3 -= 1
.a:
    ble $t3, $sp, .done
    andi $t1, $t0, 0xf

    move $s0, $t0
    move $a0, $t1
    bal print_hex
    move $t0, $s0

    srl $t0, $t0, 4
    addi $t3, $t3, 1
    j .a
.done:

    move $sp, $t8
    jr $ra
__start:
    li $v0, 30
    syscall
    bal print
    li $v0, 11
    li $a0, '\n'
    syscall
    j __start
    li $v0, 10
    syscall
