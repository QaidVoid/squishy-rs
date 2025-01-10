use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use goblin::elf::Elf;
use rayon::iter::ParallelIterator;

use crate::{error::SquishyError, EntryKind, SquashFS, SquashFSEntry};

pub type Result<T> = std::result::Result<T, SquishyError>;

/// Get offset for AppImage. This is used by default if no offset is provided.
///
/// # Arguments
/// * `path` - Path to the appimage file.
///
/// # Returns
/// Offset of the appimage, or an error if it fails to parse Elf
pub fn get_offset<P: AsRef<Path>>(path: P) -> std::io::Result<u64> {
    let mut file = File::open(path)?;

    let mut elf_header_raw = [0; 64];
    file.read_exact(&mut elf_header_raw)?;

    let section_table_offset = u64::from_le_bytes(elf_header_raw[40..48].try_into().unwrap());
    let section_count = u16::from_le_bytes(elf_header_raw[60..62].try_into().unwrap());

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

/// Check if the provided AppImage is static
///
/// # Arguments
/// * `path` - Path to the appimage file.
///
/// # Returns
/// Boolean indicating whether the appimage is static or not
pub fn is_static_appimage<P: AsRef<Path>>(path: P) -> std::io::Result<bool> {
    let mut file = File::open(path)?;
    let mut buffer = [0_u8; 4];
    file.seek(SeekFrom::Start(24))?;
    if file.read_exact(&mut buffer).is_ok() {
        let expected_bytes: [u8; 4] = [89, 171, 65, 0];
        return Ok(buffer[..] == expected_bytes);
    }
    Ok(false)
}

pub struct AppImage<'a> {
    filter: Option<&'a str>,
    pub squashfs: SquashFS<'a>,
}

impl<'a> AppImage<'a> {
    /// Creates a new AppImage instance
    ///
    /// # Arguments
    ///
    /// * `filter` - Filter to apply
    /// * `path` - Path to AppImage
    /// * `offset` - Offset to seek to
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

    /// Find icon in AppImage, filtered
    /// It looks for icon in order:
    /// - DirIcon at AppImage root
    /// - Largest png icon in /usr/share/icons
    /// - Largest svg icon in /usr/share/icons
    /// - Largest png icon in any path
    /// - Largest svg icon in any path
    ///
    /// # Returns
    /// A SquashFS entry to the icon, if found
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

    /// Find DirIcon at AppImage root
    ///
    /// # Returns
    /// A SquashFS entry to the icon, if found
    fn search_diricon(&self) -> Option<SquashFSEntry> {
        self.squashfs
            .par_entries()
            .find_first(|entry| entry.path.to_string_lossy() == "/.DirIcon")
    }

    /// Helper method to filter paths
    ///
    /// # Returns
    /// boolean stating if the path matches the filter
    fn filter_path(&self, path: &str) -> bool {
        self.filter
            .as_ref()
            .map_or(true, |filter| path.contains(filter))
    }

    /// Find largest png (preferred) or svg icon in /usr/share/icons, filtered
    ///
    /// # Returns
    /// A SquashFS entry to the icon, if found
    fn find_largest_icon_path(&self) -> Option<SquashFSEntry> {
        let png_entries = self.squashfs.par_entries().filter(|entry| {
            let path = entry.path.to_string_lossy().to_lowercase();
            path.starts_with("/usr/share/icons/")
                && self.filter_path(&path)
                && path.ends_with(".png")
        });

        if let Some(entry) = png_entries.max_by_key(|entry| entry.size) {
            return Some(entry);
        }

        self.squashfs.par_entries().find_first(|entry| {
            let path = entry.path.to_string_lossy().to_lowercase();
            path.starts_with("/usr/share/icons")
                && self.filter_path(&path)
                && path.ends_with(".svg")
        })
    }

    /// Find largest png icon in AppImage, filtered
    ///
    /// # Returns
    /// A SquashFS entry to the icon, if found
    fn find_png_icon(&self) -> Option<SquashFSEntry> {
        let png_entries = self.squashfs.par_entries().filter(|entry| {
            let p = entry.path.to_string_lossy().to_lowercase();
            self.filter_path(&p) && p.ends_with(".png")
        });
        if let Some(entry) = png_entries.max_by_key(|entry| entry.size) {
            return Some(entry);
        }
        None
    }

    /// Find largest svg icon in AppImage, filtered
    ///
    /// # Returns
    /// A SquashFS entry to the icon, if found
    fn find_svg_icon(&self) -> Option<SquashFSEntry> {
        self.squashfs.par_entries().find_first(|entry| {
            let path = entry.path.to_string_lossy().to_lowercase();
            self.filter_path(&path) && path.ends_with(".svg")
        })
    }

    /// Find desktop file in AppImage, filtered
    ///
    /// # Returns
    /// A SquashFS entry to the desktop file, if found
    pub fn find_desktop(&self) -> Option<SquashFSEntry> {
        let desktop = self.squashfs.par_entries().find_first(|entry| {
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

    /// Find appstream file in AppImage (appdata.xml | metainfo.xml)
    ///
    /// # Returns
    /// A SquashFS entry to the appstream, if found
    pub fn find_appstream(&self) -> Option<SquashFSEntry> {
        let appstream = self.squashfs.par_entries().find_first(|entry| {
            let path = entry.path.to_string_lossy().to_lowercase();
            self.filter_path(&path)
                && (path.ends_with("appdata.xml") || path.ends_with("metainfo.xml"))
        });

        if let Some(appstream) = &appstream {
            if let EntryKind::Symlink(_) = appstream.kind {
                let final_entry = self.squashfs.resolve_symlink(appstream).unwrap();
                return final_entry;
            }
        }
        appstream
    }
}
