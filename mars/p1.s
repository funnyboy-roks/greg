# read two ints from stdin, add them, then print
main:
    # Get first number from user, put into $t0.
    li $v0, 5           # Load 5 into $v0 for syscall to read int
    syscall             # execute that syscall
    add $t0, $v0, $zero # move $v0 to $t0

    # Get second number from user, put into $t1.
    li $v0, 5           # Load 5 into $v0 for syscall to read int
    syscall             # execute that syscall
    add $t1, $v0, $zero # move $v0 to $t1

    # Compute the sum.
    add $t2, $t1, $t0   # compute $t2 = $t0 + $t1

    # Print out $t2.
    li $v0, 1           # Load 1 into $v0 for syscall to print int
    add $a0, $t2, $zero # move $t2 to $a0
    syscall             # execute that syscall

    # exit
    li $v0, 10          # Load 10 into $v0 for syscall exit
    syscall             # execute that syscall
