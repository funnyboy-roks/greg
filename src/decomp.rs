use crate::{
    inst::{Func, Inst, InstKind},
    reg::Reg,
    DebugInfo,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Addr {
    Label(String),
    Relative(i32),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DecompKind {
    Syscall,
    Nop,
    Label(String),
    /// ArithLog - f $d, $s, $t
    ArithLog {
        f: Inst,
        d: Reg,
        s: Reg,
        t: Reg,
    },
    /// DivMult - f $s, $t
    DivMult {
        f: Inst,
        s: Reg,
        t: Reg,
    },
    /// Shift - f $d, $t, a
    Shift {
        f: Inst,
        d: Reg,
        t: Reg,
        a: u32,
    },
    /// ShiftV - f $d, $t, $s
    ShiftV {
        f: Inst,
        d: Reg,
        t: Reg,
        s: Reg,
    },
    /// JumpR - f $s
    JumpR {
        f: Inst,
        s: Reg,
    },
    /// MoveFrom - f $d
    MoveFrom {
        f: Inst,
        d: Reg,
    },
    /// MoveTo - f $s
    MoveTo {
        f: Inst,
        s: Reg,
    },
    /// ArithLogI - o $t, $s, i
    ArithLogI {
        o: Inst,
        t: Reg,
        s: Reg,
        i: i32,
    },
    /// LoadI - o $t, immed32
    LoadI {
        o: Inst,
        t: Reg,
        imm: u32,
    },
    /// Branch - o $s, $t, label
    Branch {
        o: Inst,
        s: Reg,
        t: Reg,
        pos: Addr,
    },
    /// BranchZ - o $s, label
    BranchZ {
        o: Inst,
        s: Reg,
        pos: Addr,
    },
    /// LoadStore - o $t, i ($s)
    LoadStore {
        o: Inst,
        s: Reg,
        t: Reg,
        i: i32,
    },
    /// Jump - o label
    Jump {
        o: Inst,
        pos: Addr,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Decomp {
    pub kind: DecompKind,
    pub addr: usize,
}
impl Decomp {
    pub fn active_label(&self) -> Option<&str> {
        match &self.kind {
            DecompKind::Syscall => None,
            DecompKind::Nop => None,
            DecompKind::Label(_) => None,
            DecompKind::ArithLog { .. } => None,
            DecompKind::DivMult { .. } => None,
            DecompKind::Shift { .. } => None,
            DecompKind::ShiftV { .. } => None,
            DecompKind::JumpR { .. } => None,
            DecompKind::MoveFrom { .. } => None,
            DecompKind::MoveTo { .. } => None,
            DecompKind::ArithLogI { .. } => None,
            DecompKind::LoadI { .. } => None,
            DecompKind::Branch {
                pos: Addr::Label(pos),
                ..
            } => Some(pos),
            DecompKind::Branch { .. } => None,
            DecompKind::BranchZ {
                pos: Addr::Label(pos),
                ..
            } => Some(pos),
            DecompKind::BranchZ { .. } => todo!(),
            DecompKind::LoadStore { .. } => None,
            DecompKind::Jump {
                pos: Addr::Label(pos),
                ..
            } => Some(pos),
            DecompKind::Jump { .. } => None,
        }
    }
}

impl DecompKind {
    fn resolve_label(ip: usize, relative: i32, debug: Option<&DebugInfo>) -> Addr {
        let Some(debug) = debug else {
            return Addr::Relative(relative);
        };
        let Some(ip) = ip.checked_add_signed((relative as isize + 1) * 4) else {
            return Addr::Relative(relative);
        };

        let Some((label, _)) = debug.labels.iter().find(|(_, v)| **v == ip) else {
            return Addr::Relative(relative);
        };

        Addr::Label(label.to_string())
    }
    pub fn from(inst: Inst, ip: usize, debug: Option<&DebugInfo>) -> Self {
        macro_rules! make {
            (ArithLogI) => {{
                let imm = inst.imm();
                DecompKind::ArithLogI {
                    o: inst,
                    t: Reg::from(imm.rt),
                    s: Reg::from(imm.rs),
                    i: imm.imm as i32,
                }
            }};
            (LoadStore) => {{
                let imm = inst.imm();
                DecompKind::LoadStore {
                    o: inst,
                    t: Reg::from(imm.rt),
                    i: imm.imm as i32,
                    s: Reg::from(imm.rs),
                }
            }};
            (Branch) => {{
                let imm = inst.imm();
                if imm.rs == 0 && imm.rt == 0 {
                    let inst = Inst {
                        kind: InstKind::J,
                        ..inst
                    };
                    DecompKind::Jump {
                        o: inst,
                        pos: Self::resolve_label(ip, imm.imm.into(), debug),
                    }
                } else {
                    DecompKind::Branch {
                        o: inst,
                        s: Reg::from(imm.rs),
                        t: Reg::from(imm.rt),
                        pos: Self::resolve_label(ip, imm.imm.into(), debug),
                    }
                }
            }};
            (BranchZ) => {{
                let imm = inst.imm();
                DecompKind::BranchZ {
                    o: inst,
                    s: Reg::from(imm.rs),
                    pos: Self::resolve_label(ip, imm.imm.into(), debug),
                }
            }};
            (Jump) => {{
                let jmp = inst.jmp();
                DecompKind::Jump {
                    o: inst,
                    pos: Self::resolve_label(ip, jmp, debug),
                }
            }};
            (ArithLog) => {{
                let reg = inst.reg();
                DecompKind::ArithLog {
                    f: inst,
                    d: Reg::from(reg.rd),
                    s: Reg::from(reg.rs),
                    t: Reg::from(reg.rt),
                }
            }};
            (DivMult) => {{
                let reg = inst.reg();
                DecompKind::DivMult {
                    f: inst,
                    s: Reg::from(reg.rs),
                    t: Reg::from(reg.rt),
                }
            }};
            (MoveFrom) => {{
                let reg = inst.reg();
                DecompKind::MoveFrom {
                    f: inst,
                    d: Reg::from(reg.rd),
                }
            }};
            (MoveTo) => {{
                let reg = inst.reg();
                DecompKind::MoveTo {
                    f: inst,
                    s: Reg::from(reg.rs),
                }
            }};
            (Shift) => {{
                let reg = inst.reg();
                DecompKind::Shift {
                    f: inst,
                    d: Reg::from(reg.rd),
                    t: Reg::from(reg.rt),
                    a: reg.shift as u32,
                }
            }};
            (ShiftV) => {{
                let reg = inst.reg();
                DecompKind::ShiftV {
                    f: inst,
                    d: Reg::from(reg.rd),
                    t: Reg::from(reg.rt),
                    s: Reg::from(reg.rs),
                }
            }};
            (JumpR) => {{
                let reg = inst.reg();
                DecompKind::JumpR {
                    f: inst,
                    s: Reg::from(reg.rs),
                }
            }};
        }
        match inst.kind {
            InstKind::Special => match inst.func().unwrap() {
                Func::Sll if inst.opcode.0 == 0 => DecompKind::Nop,
                Func::Sll => make!(Shift),
                Func::Srl => make!(Shift),
                Func::Sra => make!(Shift),
                Func::Sllv => make!(ShiftV),
                Func::Srlv => make!(ShiftV),
                Func::Srav => make!(ShiftV),
                Func::Jr => make!(JumpR),
                Func::Jalr => make!(JumpR),
                Func::Syscall => DecompKind::Syscall,
                Func::Mfhi => make!(MoveFrom),
                Func::Mthi => make!(MoveTo),
                Func::Mflo => make!(MoveFrom),
                Func::Mtlo => make!(MoveTo),
                Func::Mult => make!(DivMult),
                Func::MultU => make!(DivMult),
                Func::Div => make!(DivMult),
                Func::DivU => make!(DivMult),
                Func::Add => make!(ArithLog),
                Func::Addu => make!(ArithLog),
                Func::Sub => make!(ArithLog),
                Func::Subu => make!(ArithLog),
                Func::And => make!(ArithLog),
                Func::Or => make!(ArithLog),
                Func::Xor => make!(ArithLog),
                Func::Nor => make!(ArithLog),
                Func::Slt => make!(ArithLog),
                Func::Sltu => make!(ArithLog),
            },
            InstKind::Bal => make!(Branch),
            InstKind::J => make!(Jump),
            InstKind::Jal => make!(Jump),
            InstKind::Beq => make!(Branch),
            InstKind::Bne => make!(Branch),
            InstKind::Blez => make!(BranchZ),
            InstKind::Bgtz => make!(BranchZ),
            InstKind::AddI => make!(ArithLogI),
            InstKind::AddIU => make!(ArithLogI),
            InstKind::SltI => make!(ArithLogI),
            InstKind::SltIU => make!(ArithLogI),
            InstKind::AndI => make!(ArithLogI),
            InstKind::OrI => make!(ArithLogI),
            InstKind::XorI => make!(ArithLogI),
            InstKind::LUI => make!(ArithLogI),
            InstKind::Mfc0 => todo!(),
            InstKind::LB => make!(LoadStore),
            InstKind::LW => make!(LoadStore),
            InstKind::LBU => make!(LoadStore),
            InstKind::LHU => make!(LoadStore),
            InstKind::SB => make!(LoadStore),
            InstKind::SH => make!(LoadStore),
            InstKind::SW => make!(LoadStore),
            InstKind::Cache => todo!(),
            InstKind::LL => todo!(),
            InstKind::Lwci => todo!(),
            InstKind::Sc => todo!(),
        }
    }
}
