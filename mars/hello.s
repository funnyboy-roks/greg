.data
s: .asciiz "hello World\n"
.text
li $v0, 4
la $a0, s
syscall
