use crate::REGS;

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Opcode(pub u32);

impl std::fmt::Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(inst) = InstKind::new(self.op()) {
            f.debug_struct("Opcode").field("op", &inst).finish()
        } else {
            f.debug_struct("Opcode").field("op", &self.op()).finish()
        }
    }
}

impl Opcode {
    pub fn op(self) -> u8 {
        (self.0 >> 26) as u8
    }

    pub fn rs(self) -> u8 {
        (self.0 >> 21) as u8 & 0b1_1111
    }

    pub fn rt(self) -> u8 {
        (self.0 >> 16) as u8 & 0b1_1111
    }

    pub fn rd(self) -> u8 {
        (self.0 >> 11) as u8 & 0b1_1111
    }

    pub fn shift(self) -> u8 {
        (self.0 >> 6) as u8 & 0b1_1111
    }

    pub fn func(self) -> u8 {
        self.0 as u8 & 0b11_1111
    }

    pub fn imm(self) -> i16 {
        (self.0 as u16 & 0xff_ff) as i16
    }

    pub fn address(self) -> u32 {
        self.0 & !(0b11_1111u32 << 26u32)
    }
}

macro_rules! foo {
    ([$($tt: tt)+] $vis: vis enum $name: ident ($type: ident) {$($n: ident = $v: literal),+$(,)?}) => {
        $($tt)+
        #[repr($type)]
        $vis enum $name {
            $($n = $v,)+
        }

        impl $name {
            pub fn new(v: $type) -> Option<Self> {
                match v {
                    $($v => Some(Self::$n),)+
                    _ => None
                }
            }
        }

        impl From<$type> for $name {
            fn from(value: $type) -> Self {
                let Some(s) = Self::new(value) else {
                    panic!("Unknown {} 0x{:02x}", stringify!($name), value);
                };

                s
            }
        }
    };
}

foo! {
    [#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]]
    pub enum InstKind(u8) {
        Special = 0x00,
        Bal = 0x01,
        J = 0x02,
        Jal = 0x03,
        Beq = 0x04,
        Bne = 0x05,
        Blez = 0x06,
        Bgtz = 0x07,
        AddI = 0x08,
        AddIU = 0x09,
        SltI = 0x0a,
        SltIU = 0x0b,
        AndI = 0x0c,
        OrI = 0x0d,
        XorI = 0x0e,
        LUI = 0x0f,
        Mfc0 = 0x10,
        LW = 0x23,
        LBU = 0x24,
        LHU = 0x25,
        SB = 0x28,
        SH = 0x29,
        SW = 0x2b,
        Cache = 0x2f,
        LL = 0x30,
        Lwci = 0x31,
        Sc = 0x38,
    }
}

foo! {
    [#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]]
    pub enum Func(u8) {
        Sll = 0x00,
        Srl = 0x02,
        Sra = 0x03,
        Sllv = 0x04,
        Srlv = 0x06,
        Srav = 0x07,

        Jr = 0x08,
        Jalr = 0x09,

        Syscall = 0x0c,

        Mfhi = 0x10,
        Mthi = 0x11,
        Mflo = 0x12,
        Mtlo = 0x13,

        Mult = 0x18,
        MultU = 0x19,
        Div = 0x1a,
        DivU = 0x1b,

        Add = 0x20,
        Addu = 0x21,
        Sub = 0x22,
        Subu = 0x23,
        And = 0x24,
        Or = 0x25,
        Xor = 0x26,
        Nor = 0x27,

        Slt = 0x2a,
        Sltu = 0x2b,
    }
}

foo! {
    [#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]]
    pub enum Syscall(u32) {
        PrintInteger = 0x01,
        PrintFloat = 0x02,
        PrintDouble = 0x03,
        PrintString = 0x04,
        ReadInteger = 0x05,
        ReadFloat = 0x06,
        ReadDouble = 0x07,
        ReadString = 0x08,
        Sbrk = 0x09,
        Exit = 0x0a,
        PrintCharacter = 0x0b,
        ReadCharacter = 0x0c,
        OpenFile = 0x0d,
        ReadFromFile = 0x0e,
        WriteToFile = 0x0f,
        CloseFile = 0x10,
        Exit2 = 0x11,

        Time = 0x1e,
        MidiOut = 0x1f,
        Sleep = 0x20,
        MidiOutSynchronous = 0x21,
        PrintHexInteger = 0x22,
        PrintBinInteger = 0x23,
        PrintUnsignedInteger = 0x24,
        SetSeed = 0x28,
        RandomInt = 0x29,
        RandomIntRange = 0x2a,
        RandomFloat = 0x2b,
        RandomDouble = 0x2c,
        ConfirmDialog = 0x32,
        InputDialogInt = 0x33,
        InputDialogFloat = 0x34,
        InputDialogDouble = 0x35,
        InputDialogString = 0x36,
        MessageDialog = 0x37,
        MessageDialogInt = 0x38,
        MessageDialogFloat = 0x39,
        MessageDialogDouble = 0x3a,
        MessageDialogString = 0x3b,
    }
}

impl Func {
    pub fn decompile(self, inst: Inst) -> String {
        let Reg {
            rd, rs, rt, shift, ..
        } = inst.reg();

        let rd = rd as usize;
        let rs = rs as usize;
        let rt = rt as usize;
        let shift = shift as usize;
        match self {
            Func::Sll => {
                if inst.opcode.0 == 0 {
                    format!("nop")
                } else {
                    format!("sll {}, {}, {}", REGS[rd], REGS[rt], shift)
                }
            }
            Func::Srl => format!("srl {}, {}, {}", REGS[rd], REGS[rt], shift),
            Func::Sra => format!("sra {}, {}, {}", REGS[rd], REGS[rt], shift),
            Func::Sllv => format!("sllv {}, {}, {}", REGS[rd], REGS[rt], REGS[rs]),
            Func::Srlv => format!("srlv {}, {}, {}", REGS[rd], REGS[rt], REGS[rs]),
            Func::Srav => format!("srav {}, {}, {}", REGS[rd], REGS[rt], REGS[rs]),
            Func::Jr => format!("jr {}", REGS[rs]),
            Func::Jalr => format!("jalr {}", REGS[rs]),
            Func::Syscall => format!("sycall"),
            Func::Mfhi => format!("mfhi {}", REGS[rd]),
            Func::Mthi => format!("mthi {}", REGS[rs]),
            Func::Mflo => format!("mflo {}", REGS[rd]),
            Func::Mtlo => format!("mtlo {}", REGS[rs]),
            Func::Mult => format!("mult {}, {}", REGS[rs], REGS[rt]),
            Func::MultU => format!("multu {}, {}", REGS[rs], REGS[rt]),
            Func::Div => format!("div {}, {}", REGS[rs], REGS[rt]),
            Func::DivU => format!("divu {}, {}", REGS[rs], REGS[rt]),
            Func::Add => format!("add {}, {}, {}", REGS[rd], REGS[rs], REGS[rt]),
            Func::Addu => format!("addu {}, {}, {}", REGS[rd], REGS[rs], REGS[rt]),
            Func::Sub => format!("sub {}, {}, {}", REGS[rd], REGS[rs], REGS[rt]),
            Func::Subu => format!("subu {}, {}, {}", REGS[rd], REGS[rs], REGS[rt]),
            Func::And => format!("and {}, {}, {}", REGS[rd], REGS[rs], REGS[rt]),
            Func::Or => format!("or {}, {}, {}", REGS[rd], REGS[rs], REGS[rt]),
            Func::Xor => format!("xor {}, {}, {}", REGS[rd], REGS[rs], REGS[rt]),
            Func::Nor => format!("nor {}, {}, {}", REGS[rd], REGS[rs], REGS[rt]),
            Func::Slt => format!("slt {}, {}, {}", REGS[rd], REGS[rs], REGS[rt]),
            Func::Sltu => format!("sltu {}, {}, {}", REGS[rd], REGS[rs], REGS[rt]),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Inst {
    pub kind: InstKind,
    pub opcode: Opcode,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Reg {
    pub rs: u8,
    pub rt: u8,
    pub rd: u8,
    pub shift: u8,
    pub func: u8,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Imm {
    pub rs: u8,
    pub rt: u8,
    pub imm: i16,
}

impl Inst {
    pub fn new(opcode: Opcode) -> Option<Self> {
        Some(Self {
            kind: InstKind::new(opcode.op())?,
            opcode,
        })
    }

    pub fn reg(self) -> Reg {
        Reg {
            rs: self.opcode.rs(),
            rt: self.opcode.rt(),
            rd: self.opcode.rd(),
            shift: self.opcode.shift(),
            func: self.opcode.func(),
        }
    }

    pub fn imm(self) -> Imm {
        Imm {
            rs: self.opcode.rs(),
            rt: self.opcode.rt(),
            imm: self.opcode.imm(),
        }
    }

    pub fn func(self) -> Option<Func> {
        Func::new(self.opcode.func())
    }

    pub fn decompile(self) -> String {
        let op = self.opcode;
        match self.kind {
            InstKind::Special => {
                if let Some(f) = self.func() {
                    f.decompile(self)
                } else {
                    "<unknown special opcode>".into()
                }
            }
            InstKind::Bal => format!("bal {}, 0x{:x}", REGS[op.rs() as usize], op.imm()),
            InstKind::J => format!("j {}", op.imm()),
            InstKind::Jal => format!("jal {}", op.imm()),
            InstKind::Beq => format!(
                "beq {}, {}, {}",
                REGS[op.rs() as usize],
                REGS[op.rt() as usize],
                op.imm()
            ),
            InstKind::Bne => format!(
                "bne {}, {}, {}",
                REGS[op.rs() as usize],
                REGS[op.rt() as usize],
                op.imm()
            ),
            InstKind::Blez => format!("blez {}, 0x{:x}", REGS[op.rt() as usize], op.imm()),
            InstKind::Bgtz => format!("bgtz {}, 0x{:x}", REGS[op.rt() as usize], op.imm()),
            InstKind::AddI => format!(
                "addi {}, {}, 0x{:x}",
                REGS[op.rt() as usize],
                REGS[op.rs() as usize],
                op.imm()
            ),
            InstKind::AddIU => format!(
                "addiu {}, {}, 0x{:x}",
                REGS[op.rt() as usize],
                REGS[op.rs() as usize],
                op.imm()
            ),
            InstKind::SltI => format!(
                "slti {}, {}, 0x{:x}",
                REGS[op.rt() as usize],
                REGS[op.rs() as usize],
                op.imm()
            ),
            InstKind::SltIU => format!(
                "sltiu {}, {}, 0x{:x}",
                REGS[op.rt() as usize],
                REGS[op.rs() as usize],
                op.imm()
            ),
            InstKind::AndI => format!(
                "andi {}, {}, 0x{:x}",
                REGS[op.rt() as usize],
                REGS[op.rs() as usize],
                op.imm()
            ),
            InstKind::OrI => format!(
                "ori {}, {}, 0x{:x}",
                REGS[op.rt() as usize],
                REGS[op.rs() as usize],
                op.imm()
            ),
            InstKind::XorI => format!(
                "xori {}, {}, 0x{:x}",
                REGS[op.rt() as usize],
                REGS[op.rs() as usize],
                op.imm()
            ),
            InstKind::LUI => format!("lui {}, 0x{:x}", REGS[op.rt() as usize], op.imm()),
            InstKind::Mfc0 => todo!(),
            InstKind::LW => format!(
                "lw {}, 0x{:x}({})",
                REGS[op.rt() as usize],
                op.imm(),
                REGS[op.rs() as usize],
            ),
            InstKind::LBU => format!(
                "lbu {}, 0x{:x}({})",
                REGS[op.rt() as usize],
                op.imm(),
                REGS[op.rs() as usize],
            ),
            InstKind::LHU => format!(
                "lhu {}, 0x{:x}({})",
                REGS[op.rt() as usize],
                op.imm(),
                REGS[op.rs() as usize],
            ),
            InstKind::SB => format!(
                "sb {}, 0x{:x}({})",
                REGS[op.rt() as usize],
                op.imm(),
                REGS[op.rs() as usize],
            ),
            InstKind::SH => format!(
                "sh {}, 0x{:x}({})",
                REGS[op.rt() as usize],
                op.imm(),
                REGS[op.rs() as usize],
            ),
            InstKind::SW => format!(
                "sw {}, 0x{:x}({})",
                REGS[op.rt() as usize],
                op.imm(),
                REGS[op.rs() as usize],
            ),
            InstKind::Cache => format!("<Cache OP>",),
            InstKind::LL => format!(
                "ll {}, 0x{:x}({})",
                REGS[op.rt() as usize],
                op.imm(),
                REGS[op.rs() as usize],
            ),
            InstKind::Lwci => format!(
                "lwci {}, 0x{:x}({})",
                REGS[op.rt() as usize],
                op.imm(),
                REGS[op.rs() as usize],
            ),
            InstKind::Sc => format!(
                "sc {}, 0x{:x}({})",
                REGS[op.rt() as usize],
                op.imm(),
                REGS[op.rs() as usize],
            ),
        }
    }
}
