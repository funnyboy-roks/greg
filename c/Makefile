CC=./mips32el--musl--stable-2024.05-1/bin/mipsel-buildroot-linux-musl-gcc

all: build/fib build/hello build/fib-asm

build/fib: greg.c fib.c $(CC)
	mkdir -p build
	$(CC) -nostdlib -o build/fib greg.c fib.c

build/hello: greg.c hello.c $(CC)
	mkdir -p build
	$(CC) -nostdlib -o build/hello greg.c hello.c

build/fib-asm: fib-asm.s $(CC)
	mkdir -p build
	$(CC) -nostdlib -o build/fib-asm fib-asm.s

$(CC):
	tar -xf mips32-el.tar.xz

mips32-el.tar.xz:
	wget -O mips32-el.tar.xz 'https://toolchains.bootlin.com/downloads/releases/toolchains/mips32el/tarballs/mips32el--musl--stable-2024.05-1.tar.xz'
