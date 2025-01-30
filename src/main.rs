#[macro_use]
pub mod inst;
pub mod decomp;
pub mod reg;
pub mod tui;

use std::{
    borrow::BorrowMut,
    collections::HashMap,
    ffi::CStr,
    fmt::Write as _,
    fs::{self, File},
    io::{Read, Write as _},
    ops::{Deref, DerefMut, Index, IndexMut},
    os::fd::{AsFd, AsRawFd, FromRawFd},
    path::PathBuf,
    thread,
    time::{Duration, SystemTime},
};

use clap::Parser;
use decomp::{Decomp, DecompKind};
use elf::{endian::LittleEndian, section::SectionHeader, ElfBytes};
use inst::{Func, Imm, Inst, InstKind, Opcode, Reg, Syscall};
use rand::{rngs::StdRng, Rng, SeedableRng};
use reg::*;

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum InstructionResult {
    None,
    Done,
    Exit(u32),
}

repr_impl! {
    [#[derive(Copy, Clone, Debug)]]
    pub enum FileFlags(u32) {
        ReadOnly = 0,
        WriteOnlyCreate = 1,
        WriteOnlyAppend = 9,
    }
}

macro_rules! index {
    ($ident: ident.$field: ident[$($kind: ident),+]) => {
        $(
        impl Index<$kind> for $ident {
            type Output = u32;

            fn index(&self, index: $kind) -> &Self::Output {
                &self.$field[index as usize]
            }
        }

        impl IndexMut<$kind> for $ident {
            fn index_mut(&mut self, index: $kind) -> &mut Self::Output {
                &mut self.$field[index as usize]
            }
        }
        )+
    };
}

impl FileFlags {
    pub fn open_file(self, file: &str) -> Option<File> {
        match self {
            FileFlags::ReadOnly => File::open(file).ok(),
            FileFlags::WriteOnlyCreate => File::create_new(file).ok(),
            FileFlags::WriteOnlyAppend => File::create(file).ok(),
        }
    }
}

impl Iterator for Greg {
    type Item = Inst;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ip == self.memory.text.1 {
            return None;
        }
        let inst = self.curr_inst();
        self.ip += 4;
        Some(inst)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugInfo {
    // string: addr
    labels: HashMap<String, usize>,
}

impl DebugInfo {
    pub fn from(elf: &ElfBytes<'_, LittleEndian>, text: &SectionHeader) -> Self {
        let (symtab, strtab) = elf.symbol_table().unwrap().unwrap();
        let mut labels = HashMap::new();
        // dbg!(text.sh_addr, text.sh_addr + text.sh_size);
        for (sym, name) in symtab.iter().map(|sym| {
            let name = sym.st_name;
            (sym, strtab.get(name as usize).unwrap())
        }) {
            if sym.st_value >= text.sh_addr
                && sym.st_value <= text.sh_addr + text.sh_size
                && name.len() != 0
                && (!name.starts_with('_') || name == "__start")
            {
                labels.insert(name.to_string(), sym.st_value as usize);
            }
        }
        Self { labels }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Memory {
    // (start, end)
    text: (usize, usize),
    file: (usize, usize),
    stack: (usize, usize),

    memory: Vec<u8>,
}

impl Memory {
    pub fn text(&self) -> &[u8] {
        &self.memory[self.text.0..self.text.1]
    }

    pub fn file(&self) -> &[u8] {
        &self.memory[self.file.0..self.file.1]
    }

    pub fn stack(&self) -> &[u8] {
        &self.memory[self.stack.0..self.stack.1]
    }

    pub fn alloc(&mut self, _count: usize) -> usize {
        todo!("Allocator");
    }

    pub fn get_u32<I>(&self, index: I) -> u32
    where
        I: Into<usize>,
    {
        u32::from_le_bytes(
            self.memory[index.into()..][..std::mem::size_of::<u32>()]
                .try_into()
                .unwrap(),
        )
    }

    pub fn set_u32<I>(&mut self, index: I, value: u32)
    where
        I: Into<usize>,
    {
        self.memory[index.into()..][..std::mem::size_of::<u32>()]
            .copy_from_slice(&value.to_le_bytes());
    }
}

impl Deref for Memory {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.memory
    }
}

impl DerefMut for Memory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.memory
    }
}

#[derive(Default, Debug)]
pub struct Greg {
    // TODO: This should probably be i32 and cast to u32 when needing to do unsigned ops
    pub reg: [u32; 32],
    // TODO: Floating point
    // freg: [f32; 32],
    // TODO: Dynamic memory
    pub memory: Memory,
    pub ip: usize,
    // Using a hashmap since we can close files - could also do Vec<Option<File>> but that is more
    // work than I want to do currently
    // fd: File
    pub open_files: HashMap<u32, File>,
    pub rngs: HashMap<u32, StdRng>,

    pub hi: u32,
    pub lo: u32,

    // Only included if the binary was compiled with debug info (`-ggdb` on gcc)
    pub debug: Option<DebugInfo>,

    // If Some(_), then write stdout here,
    // otherwise, print it to stdout
    pub stdout: Option<String>,
}

index!(Greg.reg[usize, u64, u32, u16, u8]);

impl Greg {
    fn inst_at(&self, ip: usize) -> Option<(usize, Inst)> {
        if !(self.memory.text.0..=self.memory.text.1 - 4).contains(&ip) {
            return None;
        }
        let opcode = Opcode(self.memory.get_u32(ip));
        let Some(inst) = Inst::new(opcode) else {
            panic!(
                "Unknown instruction: 0b{:032b}, op = 0x{:02x} (0b{:06b}), func = 0x{:02x} (0b{:06b})",
                opcode.0,
                opcode.op(),
                opcode.op(),
                opcode.func(),
                opcode.func()
            )
        };
        Some((ip, inst))
    }

    fn curr_inst(&self) -> Inst {
        assert!(self.ip <= self.memory.text.1 - 4);
        self.inst_at(self.ip).unwrap().1
    }

    fn get_rng(&mut self, n: u32) -> &mut StdRng {
        self.rngs
            .entry(n)
            .or_insert_with(|| StdRng::from_seed(Default::default()))
            .borrow_mut()
    }

    pub fn syscall(&mut self) -> InstructionResult {
        let syscall = Syscall::from(self[V0]);
        macro_rules! print_write {
            ($($arg:tt)*) => {{
                if let Some(ref mut s) = self.stdout {
                    write!(s, $($arg)*).expect("Write to string will never fail");
                } else {
                    print!($($arg)*);
                }
            }};
        }
        match syscall {
            Syscall::PrintInteger => {
                let n = self[A0] as i32;
                print_write!("{}", n);
            }
            Syscall::PrintFloat => todo!(),
            Syscall::PrintDouble => todo!(),
            Syscall::PrintString => {
                let cstr = CStr::from_bytes_until_nul(&self.memory[self[A0] as usize..])
                    .unwrap()
                    .to_str()
                    .unwrap();
                print_write!("{}", cstr);
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
                print_write!("[syscall] exit with code 0");
                return InstructionResult::Exit(0);
            }
            Syscall::PrintCharacter => {
                let c = self[A0] as u8 as char;
                print_write!("{}", c);
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
                // TODO: max open files?
                let file = CStr::from_bytes_until_nul(&self.memory[self[A0] as usize..])
                    .unwrap()
                    .to_str()
                    .unwrap();
                let flags = FileFlags::from(self[A1]);
                // ignored in MARS
                let _mode = self[A2];

                let file = flags.open_file(file);

                self[V0] = if let Some(file) = file {
                    // 0   - stdin
                    // 1   - stdout
                    // 2   - stderr
                    // 3.. - open file
                    let fd = self.open_files.len() as u32 + 3;
                    self.open_files.insert(fd, file);
                    fd
                } else {
                    (-1i32) as u32
                };
            }
            Syscall::ReadFromFile => {
                // $a0 = file descriptor
                // $a1 = address of input buffer
                // $a2 = maximum number of characters to read
                // TODO: need memory to exist first
                let fd = self[A0];
                let addr = self[A1] as usize;
                let bytes = self[A2] as usize;
                let file = self.open_files.get_mut(&fd).unwrap();
                let buf = &mut self.memory[addr..][..bytes];
                match file.read(buf) {
                    Ok(n) => self[V0] = n as u32,
                    Err(e) => {
                        dbg!(e);
                        self[V0] = (-1i32) as u32;
                    }
                }
            }
            Syscall::WriteToFile => {
                let fd = self[A0];
                let buf = self[A1] as usize;
                let len = self[A2] as usize;

                if let Some(mut file) = self.open_files.get(&fd) {
                    if buf >= self.memory.file.1 {
                        todo!("memory nyi");
                    }
                    match file.write(&self.memory[buf..buf + len]) {
                        Ok(n) => self[V0] = n as u32,
                        Err(_) => {
                            // dbg!(e);
                            self[V0] = (-1i32) as u32;
                        }
                    }
                } else {
                    self[V0] = (-1i32) as u32;
                }
            }
            Syscall::CloseFile => {
                let fd = self[A0];

                if let Some(_) = self.open_files.get(&fd) {
                    self.open_files.remove(&fd);
                }
            }
            Syscall::Exit2 => {
                let code = self[A0];
                print_write!("[syscall] exit with explicit code {:?}", code);
                return InstructionResult::Exit(code);
            }
            Syscall::Time => {
                let time = SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap();
                let secs: u64 = time.as_millis().try_into().unwrap();

                self[A0] = secs as u32;
                self[A1] = (secs >> 32) as u32;
            }
            Syscall::MidiOut => todo!(),
            Syscall::Sleep => {
                let dur = self[A0];
                thread::sleep(Duration::from_millis(dur as u64));
            }
            Syscall::MidiOutSynchronous => todo!(),
            Syscall::PrintHexInteger => {
                let n = self[A0];
                if let Some(ref mut s) = self.stdout {
                    write!(s, "0x{:08x}", n).expect("Write to string will never fail");
                } else {
                    print!("0x{:08x}", n);
                }
            }
            Syscall::PrintBinInteger => {
                let n = self[A0];
                if let Some(ref mut s) = self.stdout {
                    write!(s, "0b{:032b}", n).expect("Write to string will never fail");
                } else {
                    print!("0b{:032b}", n);
                }
            }
            Syscall::PrintUnsignedInteger => {
                let n = self[A0];
                if let Some(ref mut s) = self.stdout {
                    write!(s, "{}", n).expect("Write to string will never fail");
                } else {
                    print!("{}", n);
                }
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
        InstructionResult::None
    }

    pub fn spec_op(&mut self, inst: Inst) -> InstructionResult {
        let Some(func) = inst.func() else {
            todo!("Unknown op 0x{0:02x} (0b{0:06b})", inst.opcode.func());
        };

        // dbg!(func);
        // eprintln!(
        //     "[{}:{}:{}] inst.reg() = {:?}",
        //     file!(),
        //     line!(),
        //     column!(),
        //     inst.reg()
        // ); // inlined dbg!() (ish)

        let Reg {
            rs, rt, rd, shift, ..
        } = inst.reg();
        match func {
            Func::Sll => {
                self[rd] = self[rt] << shift;
            }
            Func::Srl => {
                self[rd] = self[rt] as u32 >> shift as u32;
            }
            Func::Sra => {
                self[rd] = (self[rt] as i32 >> shift as i32) as u32;
            }
            Func::Sllv => {
                self[rd] = self[rt] << self[rs];
            }
            Func::Srlv => {
                self[rd] = self[rt] >> self[rs];
            }
            Func::Srav => {
                self[rd] = (self[rt] as i32 >> self[rs] as i32) as u32;
            }
            Func::Jr => {
                self.ip = self[rs] as usize;
            }
            Func::Jalr => {
                self[RA] = self.ip as u32;
                self.ip = self[rs] as usize;
            }
            Func::Syscall => {
                // dbg!(self[V0], self[A0], self[A1]);
                return self.syscall();
            }
            Func::Mfhi => self[rd] = self.hi,
            Func::Mthi => self.hi = self[rs],
            Func::Mflo => self[rd] = self.lo,
            Func::Mtlo => self.lo = self[rs],
            Func::Mult => {
                let s = self[rs] as i32;
                let t = self[rt] as i32;

                let prod = (s as i64 * t as i64) as u64;

                self.hi = (prod >> 32) as u32;
                self.lo = (prod & 0xffff_ffff) as u32;
            }
            Func::MultU => {
                let s = self[rs] as u32;
                let t = self[rt] as u32;

                let prod = s as u64 * t as u64;

                self.hi = (prod >> 32) as u32;
                self.lo = (prod & 0xffff_ffff) as u32;
            }
            Func::Div => {
                let s = self[rs] as i32;
                let t = self[rt] as i32;

                self.hi = (s % t) as u32;
                self.lo = (s / t) as u32;
            }
            Func::DivU => {
                let s = self[rs] as u32;
                let t = self[rt] as u32;

                self.hi = s % t;
                self.lo = s / t;
            }
            Func::Add => {
                self[rd] = self[rs].wrapping_add_signed(self[rt] as i32);
            }
            Func::Addu => {
                self[rd] = self[rs].wrapping_add(self[rt]);
            }
            Func::Sub => {
                self[rd] = (self[rs] as i32 - self[rt] as i32) as u32;
            }
            Func::Subu => {
                self[rd] = self[rs] - self[rt];
            }
            Func::And => {
                self[rd] = self[rs] & self[rt];
            }
            Func::Or => {
                self[rd] = self[rs] | self[rt];
            }
            Func::Xor => {
                self[rd] = self[rs] ^ self[rt];
            }
            Func::Nor => {
                self[rd] = !(self[rs] | self[rt]);
            }
            Func::Slt => {
                self[rd] = u32::from((self[rs as usize] as i32) < (self[rt as usize] as i32));
            }
            Func::Sltu => {
                self[rd] = u32::from(self[rs as usize] < self[rt as usize]);
            }
        }

        InstructionResult::None
    }

    pub fn step(&mut self) -> InstructionResult {
        let Some(inst) = self.next() else {
            return InstructionResult::Done;
        };
        // eprintln!();
        // eprintln!("[{}:{}:{}] inst = {:?}", file!(), line!(), column!(), inst); // inlined dbg!() (ish)
        match inst.kind {
            InstKind::Special => {
                // TODO: have exit syscall return true here
                return self.spec_op(inst);
            }
            InstKind::AddI => {
                let Imm { rs, rt, imm } = inst.imm();
                self[rt] = self[rs].wrapping_add_signed(imm as i32);
            }
            InstKind::AddIU => {
                let Imm { rs, rt, imm } = inst.imm();
                self[rt] = (self[rs] as u32).wrapping_add(imm as u32);
            }
            InstKind::Bal => {
                let Imm { imm, .. } = inst.imm();
                let imm = (imm as i16) << 2;
                self[RA] = self.ip as u32 + 8;
                self.ip = self.ip.wrapping_add_signed(imm as isize);
            }
            InstKind::LB => {
                let Imm { rs, rt, imm } = inst.imm();
                self[rt] =
                    self.memory.get_u32(self[rs] as usize + imm as usize) as i32 as i8 as u32;
            }
            InstKind::LW => {
                let Imm { rs, rt, imm } = inst.imm();
                let base = self[rs];
                let offset = imm as i32;
                let addr = base.wrapping_add_signed(offset);
                assert_eq!(addr & 0b11, 0);
                self[rt] = self.memory.get_u32(addr as usize);
            }
            InstKind::LUI => {
                let Imm { rt, imm, .. } = inst.imm();
                self[rt] = (imm as u32) << 16;
            }
            InstKind::OrI => {
                let Imm { rs, rt, imm } = inst.imm();
                self[rt] = self[rs] | imm as u32;
            }
            InstKind::SW => {
                let Imm { rs, rt, imm } = inst.imm();
                let rt = self[rt] as u32;
                let rs = self[rs];
                self.memory
                    .set_u32(rs.wrapping_add_signed(imm.into()) as usize, rt);
            }
            InstKind::SB => {
                // MEM [$s + i]:1 = LB ($t)
                let Imm { rs, rt, imm } = inst.imm();
                self.memory[rs as usize + imm as usize] = rt as u8;
            }
            InstKind::LL => {
                // $rt = MEM[$base+$offset]
                let Imm { rs, rt, imm } = inst.imm();
                // TODO: what size should this be?
                self[rt] = self.memory[self[rs] as usize + imm as usize] as i32 as u32;
            }
            InstKind::Lwci => {
                // eprintln!("[NYI] lwci");
                // $ft = memory[base+offset]
                // let Imm { rs, rt, imm } = inst.imm();
                // dbg!(rt, rs, imm);
                todo!("lwci")
                // self[rt] = mem[self[rs] as usize + imm as usize] as u32;
            }
            InstKind::Bne => {
                let Imm { rs, rt, imm } = inst.imm();
                let imm = (imm as i16 as i32) << 2;
                if self[rs] != self[rt] {
                    self.ip = self.ip.wrapping_add_signed(imm as isize);
                }
            }
            InstKind::Sc => {
                // if atomic_update then memory[base+offset] ← rt, rt ← 1 else rt ← 0
                let Imm { rs, rt, imm } = inst.imm();
                let s = self[rs] as i32;
                let t = self[rt] as u8;
                self.memory[(s + imm as i16 as i32) as usize] = t;
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
                    self.ip = self.ip.wrapping_add_signed(imm as isize);
                }
            }
            InstKind::SltI => {
                // $t = ($s < SE(i))
                let Imm { rs, rt, imm } = inst.imm();
                let imm = imm as i16 as i32;
                self[rt] = u32::from((self[rs] as i32) < imm);
            }
            InstKind::J => {
                let addr = inst.jmp();
                let imm = addr << 2;
                self.ip = self.ip.wrapping_add_signed(imm as isize);
            }
            InstKind::Jal => {
                let addr = inst.jmp();
                let imm = addr << 2;
                self[RA] = self.ip as u32;
                self.ip = self.ip.wrapping_add_signed(imm as isize);
            }
            InstKind::Blez => {
                let Imm { rs, imm, .. } = inst.imm();
                let imm = (imm as i32) << 2;
                if self[rs] <= 0 {
                    self.ip = self.ip.wrapping_add_signed(imm as isize);
                }
            }
            InstKind::Bgtz => {
                let Imm { rs, imm, .. } = inst.imm();
                let imm = (imm as i32) << 2;
                if self[rs] > 0 {
                    self.ip = self.ip.wrapping_add_signed(imm as isize);
                }
            }
            InstKind::SltIU => {
                // $t = ($s < SE(i))
                let Imm { rs, rt, imm } = inst.imm();
                let imm = imm as u32;
                self[rt] = u32::from(self[rs] < imm);
            }
            InstKind::AndI => {
                let Imm { rs, rt, imm } = inst.imm();
                self[rt] = self[rs] & imm as u32;
            }
            InstKind::XorI => {
                let Imm { rs, rt, imm } = inst.imm();
                self[rt] = self[rs] ^ imm as u32;
            }
            InstKind::Mfc0 => todo!(),
            InstKind::LBU => {
                // $t = MEM[$s+i]
                let Imm { rs, rt, imm } = inst.imm();
                self[rt] = self.memory[self[rs] as usize + imm as usize] as u32;
            }
            InstKind::LHU => {
                // $t = MEM[$s+i]
                let Imm { rs, rt, imm } = inst.imm();
                self[rt] = u16::from_le_bytes(
                    self.memory[self[rs] as usize + imm as usize..][..2]
                        .try_into()
                        .unwrap(),
                ) as u32;
            }
            InstKind::SH => {
                // $t = MEM[$s+i]
                let Imm { rs, rt, imm } = inst.imm();
                let t = (self[rt] & 0xff_ff) as u16;
                let s = self[rs] as usize;
                self.memory[s + imm as usize..][..2].copy_from_slice(&t.to_le_bytes());
            }
        }

        InstructionResult::None
    }

    fn decompile(&self) -> Vec<Decomp> {
        let mut lines = Vec::with_capacity(
            (self.memory.text.1 - self.memory.text.0) / 4
                + self.debug.as_ref().map(|d| d.labels.len()).unwrap_or(0),
        );
        for ip in (self.memory.text.0..self.memory.text.1).step_by(4) {
            let Some((ip, inst)) = self.inst_at(ip) else {
                continue;
            };
            if let Some(debug) = &self.debug {
                for (label, _) in debug.labels.iter().filter(|(_, v)| **v == ip) {
                    lines.push(Decomp {
                        kind: DecompKind::Label(label.to_string()),
                        addr: ip,
                    })
                }
            }
            let kind = DecompKind::from(inst, ip, self.debug.as_ref());
            let decomp = Decomp { kind, addr: ip };
            lines.push(decomp);
        }
        lines
    }
}

#[derive(Parser, Debug, Clone)]
struct Cli {
    #[clap(long, short)]
    tui: bool,
    #[clap()]
    file: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    // TODO: better elf parsing
    let file = fs::read(cli.file).unwrap();
    let elf = elf::ElfBytes::<LittleEndian>::minimal_parse(&file).unwrap();
    // dbg!(&elf);

    let text = elf.section_header_by_name(".text").unwrap().unwrap();
    let foo = elf.symbol_table().unwrap().unwrap();
    let mut start = 0;
    for x in foo.0.iter() {
        if foo.1.get(x.st_name as usize).unwrap() == "__start" {
            // dbg!(&x, text.sh_addr);
            start = x.st_value as usize;
            break;
        }
    }
    // let (text_data, _) = elf.section_data(&text).unwrap();

    let debug = DebugInfo::from(&elf, &text);

    let file_len = file.len();
    let file_len = file_len + file_len % 4;
    let mut mem = file;
    mem.resize(file_len + 2 * 1024 * 1024, 0);

    let mut greg = Greg {
        reg: Default::default(),
        memory: Memory {
            text: (
                text.sh_addr as usize,
                text.sh_addr as usize + text.sh_size as usize,
            ),
            file: (0, file_len),
            stack: (file_len, file_len + 1024 * 1024),
            memory: mem,
        },
        ip: start,
        open_files: Default::default(),
        rngs: Default::default(),
        stdout: cli.tui.then(String::new),
        hi: 0,
        lo: 0,
        debug: Some(debug),
    };

    greg[SP] = greg.memory.stack.1 as u32;

    // dbg!(&greg);

    // println!("decompiled:");
    // for inst in greg.decompile() {
    //     println!("   {:?}", inst);
    // }

    if cli.tui {
        tui::run_tui(greg).unwrap();
    } else {
        while greg.step() == InstructionResult::None {}
    }
}
