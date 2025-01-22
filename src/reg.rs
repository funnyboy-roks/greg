use std::fmt::Display;

use ratatui::{
    style::{Color, Stylize},
    text::Span,
};

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

pub const REGS: [&str; 32] = [
    "$zero", "$at", "$v0", "$v1", "$a0", "$a1", "$a2", "$a3", "$t0", "$t1", "$t2", "$t3", "$t4",
    "$t5", "$t6", "$t7", "$s0", "$s1", "$s2", "$s3", "$s4", "$s5", "$s6", "$s7", "$t8", "$t9",
    "$k0", "$k1", "$gp", "$sp", "$fp", "$ra",
];

repr_impl! {
    [#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]]
    pub enum Reg(u32) {
        Zero = 0,
        AT = 1,
        // First and second return values
        V0 = 2,
        V1 = 3,
        // function arguments
        A0 = 4,
        A1 = 5,
        A2 = 6,
        A3 = 7,
        // Temporary registers
        T0 = 8,
        T1 = 9,
        T2 = 10,
        T3 = 11,
        T4 = 12,
        T5 = 13,
        T6 = 14,
        T7 = 15,
        T8 = 24,
        T9 = 25,

        // Kernel regesters
        K0 = 26,
        K1 = 27,

        // Saved Registers
        S0 = 16,
        S1 = 17,
        S2 = 18,
        S3 = 19,
        S4 = 20,
        S5 = 21,
        S6 = 22,
        S7 = 23,

        // Global Pointer
        GP = 28,
        // Stack pointer
        SP = 29,
        // Frame pointer
        FP = 30,
        // Return address
        RA = 31,
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum RegKind {
    Zero,
    Return,
    Arg,
    Temp,
    Stack,
    Kernel,
    Save,

    Other,
}

impl RegKind {
    pub fn color(self) -> Option<Color> {
        match self {
            RegKind::Zero => Some(Color::Blue),
            RegKind::Return => Some(Color::Green),
            RegKind::Arg => Some(Color::LightGreen),
            RegKind::Temp => Some(Color::Cyan),
            RegKind::Stack => Some(Color::LightCyan),
            RegKind::Kernel => Some(Color::Magenta),
            RegKind::Save => Some(Color::Magenta),
            RegKind::Other => Some(Color::Blue),
        }
    }
}

impl From<u8> for Reg {
    fn from(value: u8) -> Self {
        Self::from(value as u32)
    }
}

impl Reg {
    pub fn as_str(self) -> &'static str {
        REGS[self as usize]
    }

    pub fn kind(self) -> RegKind {
        match self {
            Reg::Zero => RegKind::Zero,
            Reg::AT => RegKind::Other,
            Reg::V0 | Reg::V1 => RegKind::Return,
            Reg::A0 | Reg::A1 | Reg::A2 | Reg::A3 => RegKind::Arg,
            Reg::T0
            | Reg::T1
            | Reg::T2
            | Reg::T3
            | Reg::T4
            | Reg::T5
            | Reg::T6
            | Reg::T7
            | Reg::T8
            | Reg::T9 => RegKind::Temp,
            Reg::K0 | Reg::K1 => RegKind::Kernel,
            Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3 | Reg::S4 | Reg::S5 | Reg::S6 | Reg::S7 => {
                RegKind::Save
            }
            Reg::GP => RegKind::Other,
            Reg::SP => RegKind::Other,
            Reg::FP => RegKind::Other,
            Reg::RA => RegKind::Other,
        }
    }

    pub fn into_span(self) -> Span<'static> {
        if let Some(col) = self.kind().color() {
            self.as_str().fg(col)
        } else {
            self.as_str().into()
        }
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Into<Span<'static>> for &Reg {
    fn into(self) -> Span<'static> {
        self.into_span()
    }
}
