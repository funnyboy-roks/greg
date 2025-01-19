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

    pub fn imm(self) -> u16 {
        self.0 as u16 & 0xff_ff
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
    pub imm: u16,
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
}
