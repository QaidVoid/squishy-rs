use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use goblin::elf::Elf;
use squishy::{error::SquishyError, EntryKind, SquashFS, SquashFSEntry};

pub type Result<T> = std::result::Result<T, SquishyError>;

pub struct AppImage<'a> {
    filter: Option<&'a str>,
    squashfs: SquashFS<'a>,
}

fn get_offset<P: AsRef<Path>>(path: P) -> std::io::Result<u64> {
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

impl<'a> AppImage<'a> {
    pub fn new<P: AsRef<Path>>(
        filter: Option<&'a str>,
        path: &'a P,
        offset: Option<u64>,
    ) -> Result<Self> {
        let offset = offset.unwrap_or(get_offset(path)?);
        let squashfs = SquashFS::from_path_with_offset(path, offset).map_err(|_| {
            SquishyError::InvalidSquashFS(
                "Couldn't find squashfs. Try providing valid offset.".to_owned(),
            )
        })?;
        Ok(AppImage { filter, squashfs })
    }

    pub fn find_icon(&self) -> Option<SquashFSEntry> {
        let icon = self
            .search_diricon()
            .or_else(|| self.find_largest_icon_path())
            .or_else(|| self.find_png_icon())
            .or_else(|| self.find_svg_icon());

        if let Some(icon) = &icon {
            if let EntryKind::Symlink(_) = icon.kind {
                let final_entry = self.squashfs.resolve_symlink(icon).unwrap();
                return final_entry;
            }
        }
        icon
    }

    fn search_diricon(&self) -> Option<SquashFSEntry> {
        self.squashfs
            .entries()
            .find(|entry| entry.path.to_string_lossy() == "/.DirIcon")
    }

    fn filter_path(&self, path: &str) -> bool {
        self.filter
            .as_ref()
            .map_or(true, |filter| path.contains(filter))
    }

    fn find_largest_icon_path(&self) -> Option<SquashFSEntry> {
        let png_entries = self.squashfs.entries().filter(|entry| {
            let path = entry.path.to_string_lossy().to_lowercase();
            path.starts_with("/usr/share/icons/")
                && self.filter_path(&path)
                && path.ends_with(".png")
        });

        if let Some(entry) = png_entries.max_by_key(|entry| entry.size) {
            return Some(entry);
        }

        self.squashfs.entries().find(|entry| {
            let path = entry.path.to_string_lossy().to_lowercase();
            path.starts_with("/usr/share/icons")
                && self.filter_path(&path)
                && path.ends_with(".svg")
        })
    }

    fn find_png_icon(&self) -> Option<SquashFSEntry> {
        let png_entries = self.squashfs.entries().filter(|entry| {
            let p = entry.path.to_string_lossy().to_lowercase();
            self.filter_path(&p) && p.ends_with(".png")
        });
        if let Some(entry) = png_entries.max_by_key(|entry| entry.size) {
            return Some(entry);
        }
        None
    }

    fn find_svg_icon(&self) -> Option<SquashFSEntry> {
        self.squashfs.entries().find(|entry| {
            let path = entry.path.to_string_lossy().to_lowercase();
            self.filter_path(&path) && path.ends_with(".svg")
        })
    }

    pub fn find_desktop(&self) -> Option<SquashFSEntry> {
        let desktop = self.squashfs.entries().find(|entry| {
            let path = entry.path.to_string_lossy().to_lowercase();
            self.filter_path(&path) && path.ends_with(".desktop")
        });

        if let Some(desktop) = &desktop {
            if let EntryKind::Symlink(_) = desktop.kind {
                let final_entry = self.squashfs.resolve_symlink(desktop).unwrap();
                return final_entry;
            }
        }
        desktop
    }

    pub fn find_appstream(&self) -> Option<SquashFSEntry> {
        let appstream = self.squashfs.entries().find(|entry| {
            let path = entry.path.to_string_lossy().to_lowercase();
            self.filter_path(&path)
                && (path.ends_with("appdata.xml") || path.ends_with("metadata.xml"))
        });

        if let Some(appstream) = &appstream {
            if let EntryKind::Symlink(_) = appstream.kind {
                let final_entry = self.squashfs.resolve_symlink(appstream).unwrap();
                return final_entry;
            }
        }
        appstream
    }

    pub fn write<P: AsRef<Path>>(&self, file: P, output_dir: P) -> Result<()> {
        let file = file.as_ref();
        let file_name = file.file_name().unwrap();
        let output_path = output_dir.as_ref().join(file_name);
        fs::create_dir_all(output_path.parent().unwrap())?;
        self.squashfs.write_file(file, &output_path)?;
        println!(
            "Wrote {} to {}",
            file_name.to_string_lossy(),
            output_path.to_string_lossy()
        );
        Ok(())
    }
}
