CC=./mips32el--musl--stable-2024.05-1/bin/mipsel-buildroot-linux-musl-gcc
CFLAGS=-nostdlib -ggdb

# TODO: I don't want to specify each target

all: build/fib build/hello build/fib-asm build/collatz

build/collatz: collatz.s $(CC)
	mkdir -p build
	$(CC) $(CFLAGS) -o build/collatz collatz.s

build/fib: greg.c fib.c $(CC)
	mkdir -p build
	$(CC) $(CFLAGS) -o build/fib greg.c fib.c

build/hello: greg.c hello.c $(CC)
	mkdir -p build
	$(CC) $(CFLAGS) -o build/hello greg.c hello.c

build/fib-asm: fib-asm.s $(CC)
	mkdir -p build
	$(CC) $(CFLAGS) -o build/fib-asm fib-asm.s

$(CC):
	tar -xf mips32-el.tar.xz

mips32-el.tar.xz:
	wget -O mips32-el.tar.xz 'https://toolchains.bootlin.com/downloads/releases/toolchains/mips32el/tarballs/mips32el--musl--stable-2024.05-1.tar.xz'
