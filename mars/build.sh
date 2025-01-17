#!/bin/sh

set -xe

if [[ "$1" = "clean" ]]; then
    rm *.bin mars.jar || true
    exit 0
fi

if [[ ! -f mars.jar ]]; then
	wget -O mars.jar https://github.com/dpetersanderson/MARS/releases/download/v.4.5.1/Mars4_5.jar
fi

file=${1%.*}

data_file="${file}.data.bin"
text_file="${file}.text.bin"

java -jar ./mars.jar a \
    dump .data Binary "$data_file" \
    dump .text Binary "$text_file" \
    "$1"

cat "$data_file" "$text_file" > "${file}.bin"
