typedef unsigned int size_t;
void exit(size_t exit_status);
void print(char *str);
void print_int(size_t n);
void print_char(char c);
size_t syscall0(size_t call);
size_t syscall1(size_t call, size_t arg1);
