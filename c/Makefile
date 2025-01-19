CC=./mips32el--musl--stable-2024.05-1/bin/mipsel-buildroot-linux-musl-gcc

hello: greg.c hello.c $(CC)
	$(CC) -nostdlib -o hello greg.c hello.c

$(CC):
	tar -xf mips32-el.tar.xz

mips32-el.tar.xz:
	wget -O mips32-el.tar.xz 'https://toolchains.bootlin.com/downloads/releases/toolchains/mips32el/tarballs/mips32el--musl--stable-2024.05-1.tar.xz'
