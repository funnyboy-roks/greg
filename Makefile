build/mips32-el.tar.xz: build
	wget -O build/mips32-el.tar.xz 'https://toolchains.bootlin.com/downloads/releases/toolchains/mips32el/tarballs/mips32el--musl--stable-2024.05-1.tar.xz'

build:
	mkdir build
