.text
    /* exports syscall5 to other compilation units (files) */
    .globl syscall1

    syscall1:
        move $v0,$a0
        move $a0,$a1
        syscall
        jr $ra
