use std::{
    ffi::{OsStr, OsString},
    fs,
    path::Path,
};

use rayon::iter::ParallelIterator;
use squishy::{error::SquishyError, EntryKind, SquashFS, SquashFSEntry};

use crate::common::get_offset;

pub type Result<T> = std::result::Result<T, SquishyError>;

pub struct AppImage<'a> {
    filter: Option<&'a str>,
    squashfs: SquashFS<'a>,
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
            .par_entries()
            .find_first(|entry| entry.path.to_string_lossy() == "/.DirIcon")
    }

    fn filter_path(&self, path: &str) -> bool {
        self.filter
            .as_ref()
            .map_or(true, |filter| path.contains(filter))
    }

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

    fn find_svg_icon(&self) -> Option<SquashFSEntry> {
        self.squashfs.par_entries().find_first(|entry| {
            let path = entry.path.to_string_lossy().to_lowercase();
            self.filter_path(&path) && path.ends_with(".svg")
        })
    }

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

    pub fn find_appstream(&self) -> Option<SquashFSEntry> {
        let appstream = self.squashfs.par_entries().find_first(|entry| {
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

    pub fn write<P: AsRef<Path>>(
        &self,
        entry: &SquashFSEntry,
        output_dir: P,
        output_name: Option<&OsStr>,
        copy_permissions: bool,
    ) -> Result<()> {
        if let EntryKind::File(basic_file) = entry.kind {
            let file = &entry.path;
            let file_name = output_name
                .map(|output_name| {
                    let name_with_extension = file
                        .extension()
                        .map(|ext| {
                            let file_str = file.file_name().unwrap().to_string_lossy();
                            if file_str.ends_with("appdata.xml") || file.ends_with("appdata.xml") {
                                let base_name = if file_str.ends_with("appdata.xml") {
                                    "appdata"
                                } else {
                                    "metadata"
                                };
                                format!(
                                    "{}.{}.{}",
                                    output_name.to_string_lossy(),
                                    base_name,
                                    ext.to_string_lossy()
                                )
                            } else {
                                format!(
                                    "{}.{}",
                                    output_name.to_string_lossy(),
                                    ext.to_string_lossy()
                                )
                            }
                        })
                        .unwrap_or_else(|| file.file_name().unwrap().to_string_lossy().to_string());

                    OsString::from(name_with_extension)
                })
                .unwrap_or_else(|| file.file_name().unwrap().to_os_string());

            fs::create_dir_all(&output_dir)?;
            let output_path = output_dir.as_ref().join(file_name);
            if copy_permissions {
                self.squashfs.write_file_with_permissions(
                    basic_file,
                    &output_path,
                    entry.header,
                )?;
            } else {
                self.squashfs.write_file(basic_file, &output_path)?;
            }
            println!("Wrote {} to {}", file.display(), output_path.display());
        }
        Ok(())
    }
}
