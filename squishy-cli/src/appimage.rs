use std::{fs, path::Path};

use squishy::{error::SquishyError, EntryKind, SquashFS, SquashFSEntry};

pub type Result<T> = std::result::Result<T, SquishyError>;

pub struct AppImage<'a> {
    app: &'a str,
    squashfs: SquashFS<'a>,
}

impl<'a> AppImage<'a> {
    pub fn new<P: AsRef<Path>>(app: &'a str, path: &'a P) -> Result<Self> {
        let squashfs = SquashFS::from_path(path)?;
        Ok(AppImage { app, squashfs })
    }

    pub fn find_icon(&self) -> Option<SquashFSEntry> {
        let icon = if let Some(icon) = self.search_diricon() {
            Some(icon)
        } else if let Some(icon) = self.find_largest_icon() {
            Some(icon)
        } else if let Some(icon) = self.find_icon_in_all_path() {
            Some(icon)
        } else {
            self.find_contains_icon_in_all_path()
        };

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

    fn find_largest_icon(&self) -> Option<SquashFSEntry> {
        let app = self.app;
        let png_entries = self.squashfs.find_entries(move |p| {
            let p = p.to_string_lossy();
            p.starts_with("/usr/share/icons/")
                && p.to_lowercase().ends_with(&format!("{}.png", &app))
        });

        if let Some(entry) = png_entries.max_by_key(|entry| entry.size) {
            return Some(entry);
        }

        self.squashfs.entries().find(|entry| {
            let path = entry.path.to_string_lossy();
            path.starts_with("/usr/share/icons")
                && path.to_lowercase().ends_with(&format!("{}.svg", &app))
        })
    }

    fn find_icon_in_all_path(&self) -> Option<SquashFSEntry> {
        let app = self.app;
        let png_entries = self.squashfs.find_entries(move |p| {
            let p = p.to_string_lossy();
            p.to_lowercase().ends_with(&format!("{}.png", &app))
        });

        if let Some(entry) = png_entries.max_by_key(|entry| entry.size) {
            return Some(entry);
        }

        self.squashfs.entries().find(|entry| {
            let path = entry.path.to_string_lossy();
            path.to_lowercase().ends_with(&format!("{}.svg", &app))
        })
    }

    fn find_contains_icon_in_all_path(&self) -> Option<SquashFSEntry> {
        let app = self.app;
        let png_entries = self.squashfs.find_entries(move |p| {
            let p = p.to_string_lossy();
            p.to_lowercase().contains(&format!("{}.png", &app))
        });

        if let Some(entry) = png_entries.max_by_key(|entry| entry.size) {
            return Some(entry);
        }

        self.squashfs.entries().find(|entry| {
            let path = entry.path.to_string_lossy();
            path.to_lowercase().ends_with(&format!("{}.svg", &app))
        })
    }

    pub fn find_desktop(&self) -> Option<SquashFSEntry> {
        let desktop = self.squashfs.entries().find(|entry| {
            let path = entry.path.to_string_lossy();
            path.ends_with(".desktop")
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
            let path = entry.path.to_string_lossy();
            path.ends_with("appdata.xml") || path.ends_with("metadata.xml")
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
