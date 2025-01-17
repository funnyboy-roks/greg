void _exit_(void) {
    asm(
        "li $v0, 10\n"
        "syscall"
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

void print_int(unsigned int n) {
}

void syscall1(size_t number, int arg1);

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
    syscall1(1, x);
    _exit_();
    // 1 + 2;
    // print("hello");
    // exit();
}
