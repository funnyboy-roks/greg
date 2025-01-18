#include "syscall.h"


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

void syscall0(size_t call) {
    asm(
        "move $v0, %0\n"
        "syscall"
        :
        : "r"(call)
        : "$v0"
    );
}

void syscall1(size_t call, size_t arg1) {
    asm(
        "move $v0, %0\n"
        "move $a0, %1\n"
        "syscall"
        :
        : "r"(call), "r"(arg1)
        : "$a0", "$v0"
    );
}

void _exit_(void) {
    syscall0(10);
}

void print(char *str) {
    syscall1(4, (size_t)str);
}

void print_int(size_t n) {
    syscall1(1, n);
}
