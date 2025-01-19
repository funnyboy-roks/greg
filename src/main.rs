pub mod inst;
pub mod reg;

use std::{
    ffi::CStr,
    fs::{self},
    ops::{Index, IndexMut},
    process,
};

use elf::endian::LittleEndian;
use inst::{Func, Imm, Inst, InstKind, Opcode, Reg, Syscall};
use reg::*;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
struct Foo<'a> {
    text: &'a [u8],
    pc: usize,
}

impl Iterator for Foo<'_> {
    type Item = Inst;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pc == self.text.len() {
            return None;
        }
        assert!(self.pc < self.text.len());
        let bytes = &self.text[self.pc..][..4];
        self.pc += 4;
        let opcode = Opcode(u32::from_le_bytes(bytes.try_into().unwrap()));
        let Some(inst) = Inst::new(opcode) else {
            todo!(
                "full = 0b{:032b}, op = 0x{:02x} (0b{:06b}), func = 0x{:02x} (0b{:06b})",
                opcode.0,
                opcode.op(),
                opcode.op(),
                opcode.func(),
                opcode.func()
            )
        };
        Some(inst)
    }
}

#[derive(Copy, Clone, Default, Debug)]
struct Greg {
    greg: [u32; 32],
}

macro_rules! index {
    ($ident: ident[$($kind: ident),+]) => {
        $(
        impl Index<$kind> for $ident {
            type Output = u32;

            fn index(&self, index: $kind) -> &Self::Output {
                &self.greg[index as usize]
            }
        }

        impl IndexMut<$kind> for $ident {
            fn index_mut(&mut self, index: $kind) -> &mut Self::Output {
                &mut self.greg[index as usize]
            }
        }
        )+
    };
}

index!(Greg[usize, u64, u32, u16, u8]);

fn syscall(data: &[u8], data_offset: usize, greg: &mut Greg) {
    let syscall = Syscall::from(greg[V0]);
    match syscall {
        Syscall::PrintInteger => {
            print!("{}", greg[A0]);
        }
        Syscall::PrintFloat => todo!(),
        Syscall::PrintDouble => todo!(),
        Syscall::PrintString => {
            eprintln!("[syscall] print string at 0x{:08x}", greg[A0]);
            let cstr =
                CStr::from_bytes_until_nul(&data[greg[A0] as usize - data_offset..]).unwrap();
            print!("{}", cstr.to_str().unwrap());
        }
        Syscall::ReadInteger => todo!(),
        Syscall::ReadFloat => todo!(),
        Syscall::ReadDouble => todo!(),
        Syscall::ReadString => todo!(),
        Syscall::Sbrk => todo!(),
        Syscall::Exit => {
            eprintln!("[syscall] exit with code {:?}", greg[A0]);
            process::exit(greg[A0] as i32);
        }
        Syscall::PrintCharacter => {
            let c = greg[A0] as u8 as char;
            print!("{}", c);
        }
        Syscall::ReadCharacter => todo!(),
        Syscall::OpenFile => todo!(),
        Syscall::ReadFromFile => todo!(),
        Syscall::WriteToFile => todo!(),
        Syscall::CloseFile => todo!(),
        Syscall::Exit2 => todo!(),
        Syscall::Time => todo!(),
        Syscall::MidiOut => todo!(),
        Syscall::Sleep => todo!(),
        Syscall::MidiOutSynchronous => todo!(),
        Syscall::PrintHexInteger => {
            print!("0x{:08x}", greg[A0]);
        }
        Syscall::PrintBinInteger => todo!(),
        Syscall::PrintUnsignedInteger => todo!(),
        Syscall::SetSeed => todo!(),
        Syscall::RandomInt => todo!(),
        Syscall::RandomIntRange => todo!(),
        Syscall::RandomFloat => todo!(),
        Syscall::RandomDouble => todo!(),
        Syscall::ConfirmDialog => todo!(),
        Syscall::InputDialogInt => todo!(),
        Syscall::InputDialogFloat => todo!(),
        Syscall::InputDialogDouble => todo!(),
        Syscall::InputDialogString => todo!(),
        Syscall::MessageDialog => todo!(),
        Syscall::MessageDialogInt => todo!(),
        Syscall::MessageDialogFloat => todo!(),
        Syscall::MessageDialogDouble => todo!(),
        Syscall::MessageDialogString => todo!(),
    }
}

fn _main() {
    let (text, data) = if let Some(text) = std::env::args().nth(1) {
        let text = std::fs::read(text).unwrap();
        let data = if let Some(data) = std::env::args().nth(2) {
            std::fs::read(data).unwrap()
        } else {
            vec![0u8]
        };
        (text, data)
    } else {
        eprintln!(
            "Usage: {} <text-file> [data-file]",
            std::env::args().next().unwrap()
        );
        process::exit(1);
    };
    run(0, &text, &data, 0)
}

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("Usage: {} <file>", std::env::args().next().unwrap());
        process::exit(1);
    };
    let file = fs::read(path).unwrap();
    let elf = elf::ElfBytes::<LittleEndian>::minimal_parse(&file).unwrap();
    dbg!(&elf);

    let text = dbg!(elf.section_header_by_name(".text").unwrap().unwrap());
    let foo = elf.symbol_table().unwrap().unwrap();
    let mut start = 0;
    for x in foo.0.iter() {
        if foo.1.get(x.st_name as usize).unwrap() == "__start" {
            dbg!(&x, text.sh_addr);
            let sec = dbg!(elf.section_headers().unwrap().get(x.st_shndx.into())).unwrap();
            start = (x.st_value - sec.sh_offset) as usize;
            break;
        }
    }
    let (text, _) = elf.section_data(&text).unwrap();

    let empty = vec![];
    let data = if let Some(data) = elf.section_header_by_name(".rodata").unwrap() {
        let (bytes, _) = elf.section_data(&data).unwrap();
        (bytes, data.sh_offset)
    } else {
        (empty.as_slice(), 0)
    };
    dbg!(start);
    // let data = elf.section_data(&text).unwrap();
    run(start, text, data.0, data.1 as usize);
}

fn spec_op(greg: &mut Greg, inst: Inst, foo: &mut Foo, data: &[u8], data_offset: usize) -> bool {
    let Some(func) = inst.func() else {
        return false;
    };

    dbg!(func);
    eprintln!(
        "[{}:{}:{}] inst.reg() = {:?}",
        file!(),
        line!(),
        column!(),
        inst.reg()
    ); // inlined dbg!() (ish)

    let Reg {
        rs, rt, rd, shift, ..
    } = inst.reg();
    match func {
        Func::Sll => {
            greg[rd] = greg[rt] << shift;
        }
        Func::Srl => todo!(),
        Func::Sra => {
            greg[rd] = greg[rt] >> shift;
        }
        Func::Sllv => todo!(),
        Func::Srlv => todo!(),
        Func::Srav => todo!(),
        Func::Jr => {
            foo.pc = greg[rs] as usize;
        }
        Func::Jalr => todo!(),
        Func::Syscall => {
            dbg!(greg[V0], greg[A0], greg[A1]);
            syscall(&data, data_offset, greg);
        }
        Func::Mfhi => todo!(),
        Func::Mthi => todo!(),
        Func::Mflo => todo!(),
        Func::Mtlo => todo!(),
        Func::Mult => todo!(),
        Func::MultU => todo!(),
        Func::Div => todo!(),
        Func::DivU => todo!(),
        Func::Add => {
            greg[rd] = greg[rs].wrapping_add_signed(greg[rt] as i32);
        }
        Func::Addu => {
            greg[rd] = greg[rs].wrapping_add(greg[rt]);
        }
        Func::Sub => todo!(),
        Func::Subu => todo!(),
        Func::And => {
            greg[rd] = rs as u32 & rt as u32;
        }
        Func::Or => {
            greg[rd] = greg[rs] | greg[rt];
        }
        Func::Xor => todo!(),
        Func::Nor => todo!(),
        Func::Slt => {
            greg[rd] = u32::from(bool::from(greg[rs as usize] < greg[rt as usize]));
        }
        Func::Sltu => todo!(),
    }

    true
}

fn run(entry_point: usize, text: &[u8], data: &[u8], data_offset: usize) {
    let mut foo = Foo {
        text: &text,
        pc: entry_point,
    };

    // TODO: Read ELF to determine how much memory is needed or something, idrk
    let mut mem = [0u8; 4 * 1024 * 1024];

    let mut greg: Greg = Default::default();
    // let mut freg = [0f32; 32];

    while let Some(inst) = foo.next() {
        eprintln!();
        eprintln!("[{}:{}:{}] inst = {:?}", file!(), line!(), column!(), inst); // inlined dbg!() (ish)
        match inst.kind {
            InstKind::Special => {
                if !spec_op(&mut greg, inst, &mut foo, data, data_offset) {
                    todo!("Unknown op 0x{0:02x} (0b{0:06b})", inst.opcode.func());
                }
            }
            InstKind::AddI => {
                let Imm { rs, rt, imm } = inst.imm();
                greg[rt] = greg[rs].wrapping_add_signed(imm as i32);
            }
            InstKind::AddIU => {
                let Imm { rs, rt, imm } = inst.imm();
                greg[rt] = greg[rs] as u32 + i32::from(imm) as u32;
            }
            InstKind::Bal => {
                let Imm { imm, .. } = inst.imm();
                let imm = (imm as i16) << 2;
                greg[RA] = foo.pc as u32 + 8;
                foo.pc = foo.pc.wrapping_add_signed(imm as isize);
            }
            InstKind::LW => {
                let Imm { rs, rt, imm } = inst.imm();
                greg[rt] = u32::from_le_bytes(
                    mem[greg[rs] as usize + imm as usize..][..4]
                        .try_into()
                        .unwrap(),
                );
            }
            InstKind::LUI => {
                let Imm { rt, imm, .. } = inst.imm();
                greg[rt] = imm as u32;
            }
            InstKind::OrI => {
                let Imm { rs, rt, imm } = inst.imm();
                greg[rs] = greg[rt] | imm as u32;
            }
            InstKind::SW => {
                let Imm { rs, rt, imm } = inst.imm();
                mem[(greg[rs] as usize + imm as usize)..][..4]
                    .copy_from_slice(&(greg[rt] as u32).to_le_bytes());
            }
            InstKind::SB => {
                // MEM [$s + i]:1 = LB ($t)
                let Imm { rs, rt, imm } = inst.imm();
                mem[rs as usize + imm as usize] = rt as u8;
            }
            InstKind::LL => {
                // $rt = MEM[$base+$offset]
                let Imm { rs, rt, imm } = inst.imm();
                greg[rt] = mem[greg[rs] as usize + imm as usize] as u32;
            }
            InstKind::Lwci => {
                eprintln!("[NYI] lwci");
                // $ft = memory[base+offset]
                let Imm { rs, rt, imm } = inst.imm();
                dbg!(rt, rs, imm);
                // greg[rt] = mem[greg[rs] as usize + imm as usize] as u32;
            }
            InstKind::Bne => {
                let Imm { rs, rt, imm } = inst.imm();
                let imm = (imm as i16 as i32) << 2;
                if greg[rs] != greg[rt] {
                    foo.pc = foo.pc.wrapping_add_signed(imm as isize);
                }
            }
            InstKind::Sc => {
                // if atomic_update then memory[base+offset] ← rt, rt ← 1 else rt ← 0
                let Imm { rs, rt, imm } = inst.imm();
                mem[(greg[rs] as i32 + imm as i16 as i32) as usize] = greg[rt] as u8;
                greg[rt] = 1;
            }
            InstKind::Cache => {
                eprintln!("CACHE OP {}", inst.opcode.rt());
            }
            InstKind::Beq => {
                // if ($s == $t) pc += i << 2
                let Imm { rs, rt, imm } = inst.imm();
                let imm = (imm as i16 as i32) << 2;
                if greg[rs] == greg[rt] {
                    foo.pc = foo.pc.wrapping_add_signed(imm as isize);
                }
            }
            InstKind::SltI => {
                // $t = ($s < SE(i))
                let Imm { rs, rt, imm } = inst.imm();
                let imm = imm as i16 as i32;
                greg[rt] = u32::from((greg[rs] as i32) < imm);
            }
            InstKind::J => todo!(),
            InstKind::Jal => todo!(),
            InstKind::Blez => todo!(),
            InstKind::Bgtz => todo!(),
            InstKind::SltIU => todo!(),
            InstKind::AndI => todo!(),
            InstKind::XorI => todo!(),
            InstKind::Mfc0 => todo!(),
            InstKind::LBU => todo!(),
            InstKind::LHU => todo!(),
            InstKind::SH => todo!(),
        }
    }
}
