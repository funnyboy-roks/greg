use std::io::{self, Read};

fn read_byte<R>(r: &mut R) -> io::Result<u8>
where
    R: Read,
{
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_n<const N: usize>(r: &mut impl Read) -> io::Result<[u8; N]> {
    let mut buf = [0u8; N];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Elf {
    abi: u8,
    flags: u32,
    headers: Vec<ProgramHeader>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
struct ProgramHeader {
    /// 0-3 	Type of segment (see below)
    kind: u32,
    /// 4-7 	The offset in the file that the data for this segment can be found (p_offset)
    p_offset: u32,
    /// 8-11 	Where you should start to put this segment in virtual memory (p_vaddr)
    p_vaddr: u32,
    /// 12-15 	Reserved for segment's physical address (p_paddr)
    p_paddr: u32,
    /// 16-19 	Size of the segment in the file (p_filesz)
    p_filesz: u32,
    /// 20-23 	Size of the segment in memory (p_memsz, at least as big as p_filesz)
    p_memsz: u32,
    /// 24-27 	Flags (see below)
    flags: u32,
    /// 28-31 	The required alignment for this section (usually a power of 2)
    alignment: u32,
}

impl Elf {
    pub fn read_elf<R>(r: &mut R) -> io::Result<Self>
    where
        R: Read,
    {
        assert_eq!(b"\x7fELF", &read_n::<4>(r)?);

        assert_eq!(1, read_byte(r)?, "Only ELF32 is supported");
        assert_eq!(
            1,
            read_byte(r)?,
            "Only little endian is supported at the moment",
        );

        let header_version = read_byte(r)?;
        dbg!(header_version);

        let abi = read_byte(r)?;

        // padding - unused
        read_n::<8>(r)?;

        let _kind = u16::from_le_bytes(read_n(r)?);
        dbg!(_kind);
        // assert_eq!(2, kind);

        let isa = u16::from_le_bytes(read_n(r)?);
        assert_eq!(0x08, isa, "Expected MIPS ISA");

        let version = u32::from_le_bytes(read_n(r)?);
        assert_eq!(0x01, version);

        let program_entry_offset = u32::from_be_bytes(read_n(r)?);
        let program_header_table_offset = u32::from_be_bytes(read_n(r)?);
        let section_header_table_offset = u32::from_be_bytes(read_n(r)?);
        let flags = u32::from_le_bytes(read_n(r)?);
        let elf_header_size = u16::from_be_bytes(read_n(r)?);
        let program_header_entry_size = u16::from_be_bytes(read_n(r)?);
        let program_header_entries = u16::from_be_bytes(read_n(r)?);
        let section_header_entry_size = u16::from_be_bytes(read_n(r)?);
        let section_header_entries = u16::from_be_bytes(read_n(r)?);

        dbg![
            program_entry_offset,
            program_header_table_offset,
            section_header_table_offset,
            elf_header_size,
            program_header_entry_size,
            program_header_entries,
            section_header_entry_size,
            section_header_entries,
        ];

        Ok(Self {
            abi,
            flags,
            headers: Vec::new(),
        })
    }
}
