void _exit_(void) {
    asm(
        "li $v0, 10\n"
        "syscall"
    );
}

void print(char *str) {
    asm(
        "add $a0, %0\n"
        "li $v0, 4\n"
        "syscall"
        :
        : "r"(str)
        : "a0", "$v0"
    );
}

// void print(char *str) {
//     asm(
//         "add $a0, %0\n"
//         "addi $v0, $zero, 4\n"
//         "syscall"
//         :
//         : "r"(str)
//         : "$v0"
//     );
// }

typedef unsigned int size_t;

void syscall0(size_t number) {
    asm(
        "move $v0, %0"
        "syscall"
        :
        : "r"(n)
        :
    );
}

void syscall1(size_t number, size_t arg1) {
    asm(
        "move $a0, %0"
        "move $v0, %1"
        "syscall"
        :
        : "r"(n), "r"(arg1)
        :
    );
}

void print_int(unsigned int n) {
    syscall1(1, (size_t)n);
}


void __start(void)
{
    unsigned int x = 35;
    // int y = 34;
    // int z = x + y;
    // print_int(x);
    // asm(
    //     "move $4, %0\n"
    //     "li $2, 1\n"
    //     "syscall"
    //     :
    //     : "r"(x)
    //     : "$2", "$4"
    // );
    // syscall1(1, x);
    // syscall1(11, '\n');
    print_int(x + 34);
    syscall0(10);
    // 1 + 2;
    // print("hello");
    // exit();
}
