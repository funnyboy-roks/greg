pub mod inst;
pub mod reg;

use std::{
    borrow::BorrowMut,
    collections::HashMap,
    ffi::CStr,
    fs::{self, File},
    io::Read,
    ops::{Index, IndexMut},
    os::fd::{AsFd, AsRawFd, FromRawFd},
    process, thread,
    time::{Duration, SystemTime},
};

use elf::endian::LittleEndian;
use inst::{Func, Imm, Inst, InstKind, Opcode, Reg, Syscall};
use rand::{rngs::StdRng, Rng, SeedableRng};
use reg::*;

impl Iterator for Greg<'_> {
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

#[derive(Default, Debug)]
struct Greg<'a> {
    reg: [u32; 32],
    file: &'a [u8],
    text: &'a [u8],
    pc: usize,
    open_files: Vec<File>,
    rngs: HashMap<u32, StdRng>,
}

macro_rules! index {
    ($ident: ident[$($kind: ident),+]) => {
        $(
        impl Index<$kind> for $ident<'_> {
            type Output = u32;

            fn index(&self, index: $kind) -> &Self::Output {
                &self.reg[index as usize]
            }
        }

        impl IndexMut<$kind> for $ident<'_> {
            fn index_mut(&mut self, index: $kind) -> &mut Self::Output {
                &mut self.reg[index as usize]
            }
        }
        )+
    };
}

index!(Greg[usize, u64, u32, u16, u8]);

impl Greg<'_> {
    fn get_rng(&mut self, n: u32) -> &mut StdRng {
        self.rngs
            .entry(n)
            .or_insert_with(|| StdRng::from_seed(Default::default()))
            .borrow_mut()
    }

    pub fn syscall(&mut self) {
        let syscall = Syscall::from(self[V0]);
        match syscall {
            Syscall::PrintInteger => {
                print!("{}", self[A0] as i32);
            }
            Syscall::PrintFloat => todo!(),
            Syscall::PrintDouble => todo!(),
            Syscall::PrintString => {
                eprintln!("[syscall] print string at 0x{:08x}", self[A0]);
                let cstr = CStr::from_bytes_until_nul(&self.file[self[A0] as usize..]).unwrap();
                print!("{}", cstr.to_str().unwrap());
            }
            Syscall::ReadInteger => {
                let line = std::io::stdin().lines().next().unwrap().unwrap();
                let n: u32 = line.parse().unwrap();
                self[V0] = n;
            }
            Syscall::ReadFloat => todo!(),
            Syscall::ReadDouble => todo!(),
            Syscall::ReadString => {
                // $a0 = address of input buffer
                // $a1 = maximum number of characters to read
                // TODO: need memory to exist first
                // let addr = greg[A0]
                // let file = unsafe { File::from_raw_fd(line.as_raw_fd()) };
                // let buf = vec![0u8; ];
                // file.read();
                todo!()
            }
            Syscall::Sbrk => todo!(),
            Syscall::Exit => {
                eprintln!("[syscall] exit with code 0");
                process::exit(0);
            }
            Syscall::PrintCharacter => {
                let c = self[A0] as u8 as char;
                print!("{}", c);
            }
            Syscall::ReadCharacter => {
                let stdin = std::io::stdin();
                let stdin = stdin.as_fd();
                let mut file = unsafe { File::from_raw_fd(stdin.as_raw_fd()) };
                let mut buf = [0u8; 1];
                file.read_exact(&mut buf).unwrap();
                self[V0] = buf[0] as u32;
            }
            Syscall::OpenFile => {
                // TODO: max file opens?
                todo!();
            }
            Syscall::ReadFromFile => todo!(),
            Syscall::WriteToFile => todo!(),
            Syscall::CloseFile => todo!(),
            Syscall::Exit2 => {
                eprintln!("[syscall] exit with explicit code {:?}", self[A0]);
                process::exit(self[A0] as i32);
            }
            Syscall::Time => {
                let time = SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap();
                let secs: u64 = time.as_millis().try_into().unwrap();

                self[A0] = (secs >> 32) as u32;
                self[A1] = secs as u32;
            }
            Syscall::MidiOut => todo!(),
            Syscall::Sleep => {
                let dur = self[A0];
                thread::sleep(Duration::from_millis(dur as u64));
            }
            Syscall::MidiOutSynchronous => todo!(),
            Syscall::PrintHexInteger => {
                print!("0x{:08x}", self[A0]);
            }
            Syscall::PrintBinInteger => {
                print!("0b{:032b}", self[A0]);
            }
            Syscall::PrintUnsignedInteger => {
                print!("{}", self[A0]);
            }
            Syscall::SetSeed => {
                self.rngs.insert(
                    self[A0],
                    StdRng::seed_from_u64(
                        //   aaaaaaaabbbbbbbb
                        // ^     cccccccc
                        // because why not
                        ((self[A0] as u64) << 32 | self[A0] as u64) ^ ((self[A0] as u64) << 16),
                    ),
                );
            }
            Syscall::RandomInt => {
                self[V0] = self.get_rng(self[A0]).r#gen();
            }
            Syscall::RandomIntRange => {
                let high = self[A1];
                self[V0] = self.get_rng(self[A0]).gen_range(0..high);
            }
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

    pub fn spec_op(&mut self, inst: Inst) -> bool {
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
                self[rd] = self[rt] << shift;
            }
            Func::Srl => todo!(),
            Func::Sra => {
                self[rd] = self[rt] >> shift;
            }
            Func::Sllv => todo!(),
            Func::Srlv => todo!(),
            Func::Srav => todo!(),
            Func::Jr => {
                self.pc = self[rs] as usize;
            }
            Func::Jalr => todo!(),
            Func::Syscall => {
                dbg!(self[V0], self[A0], self[A1]);
                self.syscall();
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
                self[rd] = self[rs].wrapping_add_signed(self[rt] as i32);
            }
            Func::Addu => {
                self[rd] = self[rs].wrapping_add(self[rt]);
            }
            Func::Sub => todo!(),
            Func::Subu => todo!(),
            Func::And => {
                self[rd] = rs as u32 & rt as u32;
            }
            Func::Or => {
                self[rd] = self[rs] | self[rt];
            }
            Func::Xor => todo!(),
            Func::Nor => todo!(),
            Func::Slt => {
                self[rd] = u32::from(bool::from(self[rs as usize] < self[rt as usize]));
            }
            Func::Sltu => todo!(),
        }

        true
    }

    pub fn run(mut self) {
        // TODO: Read ELF to determine how much memory is needed or something, idrk
        let mut mem = [0u8; 4 * 1024 * 1024];

        // let mut freg = [0f32; 32];

        while let Some(inst) = self.next() {
            eprintln!();
            eprintln!("[{}:{}:{}] inst = {:?}", file!(), line!(), column!(), inst); // inlined dbg!() (ish)
            match inst.kind {
                InstKind::Special => {
                    if !self.spec_op(inst) {
                        todo!("Unknown op 0x{0:02x} (0b{0:06b})", inst.opcode.func());
                    }
                }
                InstKind::AddI => {
                    let Imm { rs, rt, imm } = inst.imm();
                    self[rt] = self[rs].wrapping_add_signed(imm as i32);
                }
                InstKind::AddIU => {
                    let Imm { rs, rt, imm } = inst.imm();
                    self[rt] = self[rs] as u32 + i32::from(imm) as u32;
                }
                InstKind::Bal => {
                    let Imm { imm, .. } = inst.imm();
                    let imm = (imm as i16) << 2;
                    self[RA] = self.pc as u32 + 8;
                    self.pc = self.pc.wrapping_add_signed(imm as isize);
                }
                InstKind::LW => {
                    let Imm { rs, rt, imm } = inst.imm();
                    self[rt] = u32::from_le_bytes(
                        mem[self[rs] as usize + imm as usize..][..4]
                            .try_into()
                            .unwrap(),
                    );
                }
                InstKind::LUI => {
                    let Imm { rt, imm, .. } = inst.imm();
                    self[rt] = imm as u32;
                }
                InstKind::OrI => {
                    let Imm { rs, rt, imm } = inst.imm();
                    self[rs] = self[rt] | imm as u32;
                }
                InstKind::SW => {
                    let Imm { rs, rt, imm } = inst.imm();
                    mem[(self[rs] as usize + imm as usize)..][..4]
                        .copy_from_slice(&(self[rt] as u32).to_le_bytes());
                }
                InstKind::SB => {
                    // MEM [$s + i]:1 = LB ($t)
                    let Imm { rs, rt, imm } = inst.imm();
                    mem[rs as usize + imm as usize] = rt as u8;
                }
                InstKind::LL => {
                    // $rt = MEM[$base+$offset]
                    let Imm { rs, rt, imm } = inst.imm();
                    self[rt] = mem[self[rs] as usize + imm as usize] as u32;
                }
                InstKind::Lwci => {
                    eprintln!("[NYI] lwci");
                    // $ft = memory[base+offset]
                    let Imm { rs, rt, imm } = inst.imm();
                    dbg!(rt, rs, imm);
                    // self[rt] = mem[self[rs] as usize + imm as usize] as u32;
                }
                InstKind::Bne => {
                    let Imm { rs, rt, imm } = inst.imm();
                    let imm = (imm as i16 as i32) << 2;
                    if self[rs] != self[rt] {
                        self.pc = self.pc.wrapping_add_signed(imm as isize);
                    }
                }
                InstKind::Sc => {
                    // if atomic_update then memory[base+offset] ← rt, rt ← 1 else rt ← 0
                    let Imm { rs, rt, imm } = inst.imm();
                    mem[(self[rs] as i32 + imm as i16 as i32) as usize] = self[rt] as u8;
                    self[rt] = 1;
                }
                InstKind::Cache => {
                    eprintln!("CACHE OP {}", inst.opcode.rt());
                }
                InstKind::Beq => {
                    // if ($s == $t) pc += i << 2
                    let Imm { rs, rt, imm } = inst.imm();
                    let imm = (imm as i16 as i32) << 2;
                    if self[rs] == self[rt] {
                        self.pc = self.pc.wrapping_add_signed(imm as isize);
                    }
                }
                InstKind::SltI => {
                    // $t = ($s < SE(i))
                    let Imm { rs, rt, imm } = inst.imm();
                    let imm = imm as i16 as i32;
                    self[rt] = u32::from((self[rs] as i32) < imm);
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

    dbg!(start);
    // let data = elf.section_data(&text).unwrap();
    let greg = Greg {
        reg: Default::default(),
        file: &file,
        text,
        pc: start,
        open_files: Default::default(),
        rngs: Default::default(),
    };

    greg.run();
}
