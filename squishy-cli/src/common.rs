use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use goblin::elf::Elf;

pub fn get_offset<P: AsRef<Path>>(path: P) -> std::io::Result<u64> {
    let mut file = File::open(path)?;

    let mut elf_header_raw = [0; 64];
    file.read_exact(&mut elf_header_raw)?;

    let section_table_offset = u64::from_le_bytes(elf_header_raw[40..48].try_into().unwrap()); // e_shoff
    let section_count = u16::from_le_bytes(elf_header_raw[60..62].try_into().unwrap()); // e_shnum

    let section_table_size = section_count as u64 * 64;
    let required_bytes = section_table_offset + section_table_size;

    let mut header_data = vec![0; required_bytes as usize];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut header_data)?;

    let elf = Elf::parse(&header_data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let section_table_end =
        elf.header.e_shoff + (elf.header.e_shentsize as u64 * elf.header.e_shnum as u64);

    let last_section_end = elf
        .section_headers
        .last()
        .map(|section| section.sh_offset + section.sh_size)
        .unwrap_or(0);

    Ok(section_table_end.max(last_section_end))
}
