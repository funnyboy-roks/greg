CC=./mips32el--musl--stable-2024.05-1/bin/mipsel-buildroot-linux-musl-gcc

foo: sys.s foo.c
	$(CC) -nostdlib -o foo sys.s foo.c

$(CC): mips32-el.tar.xz
	tar xf mips32-el.tar.xz

mips32-el.tar.xz:
	wget -O mips32-el.tar.xz 'https://toolchains.bootlin.com/downloads/releases/toolchains/mips32el/tarballs/mips32el--musl--stable-2024.05-1.tar.xz'
