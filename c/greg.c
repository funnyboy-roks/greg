#include "greg.h"

int main(void);

typedef unsigned int size_t;

size_t syscall0(size_t call) {
    asm(
        "move $v0, %0\n"
        "syscall"
        :
        : "r"(call)
        : "$v0"
    );
}

size_t syscall1(size_t call, size_t arg1) {
    asm(
        "move $v0, %0\n"
        "move $a0, %1\n"
        "syscall"
        :
        : "r"(call), "r"(arg1)
        : "$a0", "$v0"
    );
}

#define SYS_print_int 1
#define SYS_print_str 4
#define SYS_exit 10
#define SYS_print_char 11

void exit(size_t exit_status) {
    syscall1(SYS_exit, (size_t)exit_status);
    for(;;);
}

void print(char *str) {
    syscall1(SYS_print_str, (size_t)str);
}

void print_int(size_t n) {
    syscall1(SYS_print_int, n);
}

void print_char(char c) {
    syscall1(SYS_print_char, c);
}

void __start() {
    exit(main());
}
