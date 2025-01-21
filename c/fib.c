#include "./greg.h"

int main(void)
{
    int x = 0;
    int y = 1;
    while (x < 1000) {
        print_int(x);
        int px = x;
        x = y;
        y = px + y;
        print("\n");
    }
}
