#include "./syscall.h"

void __start(void)
{
    for (int i = 0; i < 16; ++i) {
        print_int(i);
        syscall1(11, '\n');
    }
    syscall0(10);
}
