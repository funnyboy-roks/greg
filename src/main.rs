pub mod reg;

use std::{
    ffi::CStr,
    fs::{self, File},
    io::{BufRead, BufReader},
    ops::{Index, IndexMut},
    process,
};

use elf::endian::LittleEndian;
use reg::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Inst(u32);

impl Inst {
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
        Some(Inst(u32::from_le_bytes(bytes.try_into().unwrap())))
    }
}

fn pause() {
    if option_env!("DEBUG").is_some() {
        std::io::stdin().lock().lines().next();
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

fn syscall(data: &[u8], greg: &mut Greg) {
    match greg[V0] {
        0x01 => {
            eprintln!("{:02x?}", &greg.greg[..20]);
            eprintln!("[syscall] print int {}", greg[A0]);
            print!("{}", greg[A0]);
        }
        0x04 => {
            eprintln!("[syscall] print string at 0x{:08x}", greg[A0]);
            let cstr = CStr::from_bytes_until_nul(&data[greg[A0] as usize..]).unwrap();
            print!("{}", cstr.to_str().unwrap());
        }
        0x0a => {
            eprintln!("[syscall] exit");
            process::exit(0);
        }
        0x0b => {
            let c = greg[A0] as u8 as char;
            eprintln!("[syscall] print char {:?}", c);
            print!("{}", c);
        }
        0x22 => {
            eprintln!("[syscall] print int hex");
            print!("0x{:08x}", greg[A0]);
        }
        call => todo!("Unknown syscall 0x{:02x} ({})", call, call),
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
    run(0, &text, &data)
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
        let (data, _) = elf.section_data(&data).unwrap();
        data
    } else {
        &empty
    };
    dbg!(start);
    // let data = elf.section_data(&text).unwrap();
    run(start, text, data);
}

fn run(entry_point: usize, text: &[u8], data: &[u8]) {
    dbg!(text.len());
    let mut foo = Foo {
        text: &text,
        pc: entry_point,
    };

    let mut mem = [0u8; 1 * 1024 * 1024];

    let mut greg: Greg = Default::default();
    // let mut freg = [0f32; 32];

    while let Some(bar) = foo.next() {
        eprintln!();
        // dbg!(&greg, &freg, off);
        // dbg!(foo.pc);
        dbg!(greg[A0], greg[V0]);
        match bar.op() {
            0x08 => {
                eprintln!("addi");
                let rs = bar.rs();
                let rt = bar.rt();
                let imm = bar.imm();
                greg[rt] = greg[rs].wrapping_add_signed(imm as i32);
                dbg!(rs, rt, imm);
            }
            0x09 => {
                eprintln!("addiu");
                let rs = bar.rs();
                let rt = bar.rt();
                let imm = bar.imm();
                dbg!(rs, rt, imm);
                greg[rt] = greg[rs] as u32 + i32::from(imm) as u32;
            }
            0x00 if bar.func() == 0x21 => {
                eprintln!("addu");
                let rs = bar.rs();
                let rt = bar.rt();
                let rd = bar.rd();
                let shift = bar.shift();
                greg[rd] = greg[rs].wrapping_add(greg[rt]);
                dbg!(rs, rt, shift);
            }
            0x00 if bar.func() == 0x20 => {
                eprintln!("add");
                let rs = bar.rs();
                let rt = bar.rt();
                let rd = bar.rd();
                let shift = bar.shift();
                greg[rd] = greg[rs].wrapping_add_signed(greg[rt] as i32);
                dbg!(rs, rt, shift);
            }
            0x01 => {
                eprintln!("bal");
                let rs = bar.rs();
                let imm = (bar.imm() as i16) << 2;
                dbg!(rs, imm, foo.pc);
                greg[RA] = foo.pc as u32 + 8;
                foo.pc = foo.pc.wrapping_add_signed(imm as isize);
                // greg[rd] = greg[rs] | greg[rt];
                dbg!(rs, imm, foo.pc);
            }
            0x00 if bar.func() == 0x25 => {
                eprintln!("or (move?) {:08x}", bar.0);
                let rs = bar.rs();
                let rt = bar.rt();
                let rd = bar.rd();
                let shift = bar.shift();
                greg[rd] = greg[rs] | greg[rt];
                dbg!(rs, rt, rd, shift);
            }
            0x00 if bar.func() == 0x03 => {
                eprintln!("sra");
                let rs = bar.rs();
                let rt = bar.rt();
                let rd = bar.rd();
                let shift = bar.shift();
                // $d = $t >> a
                greg[rd] = greg[rt] >> shift;
                dbg!(rs, rt, shift);
            }
            0x00 if bar.func() == 0x00 => {
                eprintln!("sll");
                let rs = bar.rs();
                let rt = bar.rt();
                let rd = bar.rd();
                let shift = bar.shift();
                // $d = $t << a
                greg[rd] = greg[rt] << shift;
                dbg!(rs, rt, shift);
            }
            0x00 if bar.func() == 0x08 => {
                eprintln!("jr");
                let rs = bar.rs();
                foo.pc = greg[rs] as usize;
                dbg!(rs);
            }
            0x23 => {
                eprintln!("lw");
                let rs = bar.rs();
                let rt = bar.rt();
                let imm = bar.imm();
                // $t = MEM [$s + i]:4
                dbg!(greg[rt]);
                greg[rt] = u32::from_le_bytes(
                    mem[greg[rs] as usize + imm as usize..][..4]
                        .try_into()
                        .unwrap(),
                );
                dbg!(greg[rt]);
                dbg!(rs, rt, imm);
            }
            0x0f => {
                eprintln!("lui");
                let rs = bar.rs();
                let rt = bar.rt();
                let imm = bar.imm();
                greg[rt] = imm as u32;
                dbg!(rs, rt, imm);
            }
            0x0d => {
                eprintln!("ori");
                let rs = bar.rs();
                let rt = bar.rt() as u32;
                let imm = bar.imm() as u32;
                greg[rs] = rt | imm;
                dbg!(rs, rt, imm);
            }
            0x00 if bar.func() == 0x0c => {
                eprintln!("syscall");
                syscall(&data, &mut greg);
            }
            0x00 if bar.func() == 0x2a => {
                eprintln!("slt");
                let rs = bar.rs();
                let rt = bar.rt();
                let rd = bar.rd();
                // $d = ($s < $t)
                greg[rd] = u32::from(bool::from(greg[rs as usize] < greg[rt as usize]));
            }
            0x2b => {
                eprintln!("sw");
                let rs = bar.rs();
                let rt = bar.rt();
                let imm = bar.imm();
                dbg!(rs, rt, imm);
                mem[(greg[rs] as usize + imm as usize)..][..4]
                    .copy_from_slice(&(greg[rt] as u32).to_le_bytes());
                dbg!(&mem[(greg[rs] as usize + imm as usize)..][..4]);
            }
            0x28 => {
                eprintln!("sb");
                // MEM [$s + i]:1 = LB ($t)
                let rs = bar.rs();
                let rt = bar.rt();
                let imm = bar.imm();
                dbg!(rs, rt, imm);
                mem[rs as usize + imm as usize] = rt as u8;
            }
            0x30 => {
                eprintln!("ll");
                // $rt = MEM[$base+$offset]
                let rs = bar.rs();
                let rt = bar.rt();
                let imm = bar.imm();
                dbg!(rt, rs, imm);
                greg[rt] = mem[greg[rs] as usize + imm as usize] as u32;
            }
            0x31 => {
                eprintln!("[NYI] lwci");
                // $ft = memory[base+offset]
                let rs = bar.rs();
                let rt = bar.rt();
                let imm = bar.imm();
                dbg!(rt, rs, imm);
                // greg[rt] = mem[greg[rs] as usize + imm as usize] as u32;
            }
            0x00 if bar.func() == 0x24 => {
                eprintln!("and");
                // $d = $s & $t
                let rd = bar.rs();
                let rs = bar.rs() as u32;
                let rt = bar.rt() as u32;
                dbg!(rd, rs, rt);
                greg[rd] = rs & rt;
                // mem[rs as usize + imm as usize] = rt as u8;
            }
            0x05 => {
                eprintln!("bne");
                let rs = bar.rs();
                let rt = bar.rt();
                let imm = bar.imm() as i16;
                dbg!(rs, rt, imm);
                dbg!(greg[rs], greg[rt]);
                let imm = (imm << 2) as i32;
                pause();
                if greg[rs] != greg[rt] {
                    foo.pc = foo.pc.wrapping_add_signed(imm as isize);
                }
            }
            0x38 => {
                eprintln!("sc");
                let rs = bar.rs();
                let rt = bar.rt();
                let imm = bar.imm() as i16;
                greg[rt] = 0;
            }
            0x2f => {
                eprintln!("CACHE OP {}", bar.rt());
            }
            op => todo!(
                "full = 0b{:032b}, op = 0x{:02x} (0b{:06b}), func = 0x{:02x} (0b{:06b})",
                bar.0,
                op,
                op,
                bar.func(),
                bar.func()
            ),
        }
    }
}
