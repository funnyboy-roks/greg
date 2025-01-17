// $0 	$zero, $r0 	Always zero
// $1 	$at 	Reserved for assembler
// $2, $3 	$v0, $v1 	First and second return values, respectively
// $4, ..., $7 	$a0, ..., $a3 	First four arguments to functions
// $8, ..., $15 	$t0, ..., $t7 	Temporary registers
// $16, ..., $23 	$s0, ..., $s7 	Saved registers
// $24, $25 	$t8, $t9 	More temporary registers
// $26, $27 	$k0, $k1 	Reserved for kernel (operating system)
// $28 	$gp 	Global pointer
// $29 	$sp 	Stack pointer
// $30 	$fp 	Frame pointer
// $31 	$ra 	Return address
macro_rules! reg {
    ($($ident: ident => $n: literal),+$(,)?) => {
        $(pub const $ident: usize = $n;)+
    }
}
reg! {
    ZERO => 0,
    AT => 1,
    // First and second return values
    V0 => 2,
    V1 => 3,
    // function arguments
    A0 => 4,
    A1 => 5,
    A2 => 6,
    A3 => 7,
    // Temporary registers
    T0 => 8,
    T1 => 9,
    T2 => 10,
    T3 => 11,
    T4 => 12,
    T5 => 13,
    T6 => 14,
    T7 => 15,
    T8 => 24,
    T9 => 25,

    // Kernel regesters
    K0 => 26,
    K1 => 27,

    // Saved Registers
    S0 => 16,
    S1 => 17,
    S2 => 18,
    S3 => 19,
    S4 => 20,
    S5 => 21,
    S6 => 22,
    S7 => 23,

    // Global Pointer
    GP => 28,
    // Stack pointer
    SP => 29,
    // Frame pointer
    FP => 30,
    // Return address
    RA => 31,
}
