# print 0..=15 in decimal to stdout
main:
    # int x = 0;
    li $t0, 0
    # do {
number_loop:
    # print(x);
    li $v0, 1                 # Load 1 into $v0 for print int syscall
    add $a0, $t0, $zero       # move $t0 into $a0 so we can print it
    syscall                   # execute that syscall

    # putchar('\n')
    li $v0, 11                # Load 11 into $v0 for print char syscall
    li $a0, '\n'              # Load '\n' into $a0 for newline
    syscall                   # execute that syscall

    # x += 1;
    addi $t0, $t0, 1          # add 1 to $t0

    # } while(x < 16);
    li $t1, 16                # Load 16 into $t1 for comparison
    blt $t0, $t1, number_loop # Branch if $t0 < 16 to `number_loop`

    # exit
    li $v0, 10                # Load 10 into $v0 for syscall exit
    syscall                   # execute that syscall
