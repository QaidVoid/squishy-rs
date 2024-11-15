use std::{
    fs::{self, Permissions},
    os::unix::{self, fs::PermissionsExt},
};

use appimage::AppImage;
use clap::Parser;
use cli::Args;
use common::get_offset;
use rayon::iter::ParallelIterator;
use squishy::{error::SquishyError, EntryKind, SquashFS};

mod appimage;
mod cli;
mod common;

macro_rules! log {
    ($quiet:expr, $($arg:tt)*) => {
        if !$quiet {
            println!($($arg)*);
        }
    };
}

macro_rules! elog {
    ($quiet:expr, $($arg:tt)*) => {
        if !$quiet {
            eprintln!($($arg)*);
        }
    };
}

fn main() {
    let args = Args::parse();

    match args.command {
        cli::Commands::AppImage {
            offset,
            filter,
            file,
            icon,
            desktop,
            appstream,
            write,
            original_name,
            copy_permissions,
        } => {
            if file.exists() {
                let appimage = match AppImage::new(filter.as_deref(), &file, offset) {
                    Ok(appimage) => appimage,
                    Err(e) => {
                        elog!(args.quiet, "{}", e);
                        std::process::exit(-1);
                    }
                };

                let write_path = if let Some(write) = write {
                    if let Some(path) = write {
                        Some(path)
                    } else {
                        Some(std::env::current_dir().unwrap())
                    }
                } else {
                    None
                };

                let output_name = if original_name {
                    None
                } else {
                    file.file_name()
                };

                if desktop {
                    if let Some(desktop) = appimage.find_desktop() {
                        if let Some(ref write_path) = write_path {
                            appimage
                                .write(&desktop, write_path, output_name, copy_permissions)
                                .unwrap();
                        } else {
                            log!(args.quiet, "Desktop file: {}", desktop.path.display());
                        }
                    } else {
                        elog!(args.quiet, "No desktop file found.");
                    };
                }
                if icon {
                    if let Some(icon) = appimage.find_icon() {
                        if let Some(ref write_path) = write_path {
                            appimage
                                .write(&icon, write_path, output_name, copy_permissions)
                                .unwrap();
                        } else {
                            log!(args.quiet, "Icon: {}", icon.path.display());
                        }
                    } else {
                        elog!(args.quiet, "No icon found.");
                    };
                }
                if appstream {
                    if let Some(appstream) = appimage.find_appstream() {
                        if let Some(ref write_path) = write_path {
                            appimage
                                .write(&appstream, write_path, output_name, copy_permissions)
                                .unwrap();
                        } else {
                            log!(args.quiet, "Appstream file: {}", appstream.path.display());
                        }
                    } else {
                        elog!(args.quiet, "No appstream file found.");
                    };
                }
            }
        }
        cli::Commands::Unsquashfs {
            offset,
            file,
            write,
        } => {
            let write_path = if let Some(write) = write {
                if let Some(path) = write {
                    fs::create_dir_all(&path).unwrap();
                    Some(path)
                } else {
                    Some(std::env::current_dir().unwrap())
                }
            } else {
                None
            };

            let offset = offset.unwrap_or(get_offset(&file).unwrap());
            let squashfs = SquashFS::from_path_with_offset(&file, offset)
                .map_err(|_| {
                    SquishyError::InvalidSquashFS(
                        "Couldn't find squashfs. Try providing valid offset.".to_owned(),
                    )
                })
                .unwrap();

            squashfs.par_entries().for_each(|entry| {
                if let Some(output_dir) = &write_path {
                    let file_path = entry.path.strip_prefix("/").unwrap_or(&entry.path);
                    let output_path = output_dir.join(file_path);
                    fs::create_dir_all(output_path.parent().unwrap()).unwrap();

                    match entry.kind {
                        EntryKind::File(basic_file) => {
                            if output_path.exists() {
                                return;
                            }
                            let _ = squashfs.write_file_with_permissions(
                                basic_file,
                                &output_path,
                                entry.header,
                            );
                            log!(
                                args.quiet,
                                "Wrote {} to {}",
                                entry.path.display(),
                                output_path.display()
                            );
                        }
                        EntryKind::Directory => {
                            if output_path.exists() {
                                return;
                            }
                            fs::create_dir_all(&output_path).unwrap();
                            fs::set_permissions(
                                &output_path,
                                Permissions::from_mode(u32::from(entry.header.permissions)),
                            ).unwrap();
                            log!(
                                args.quiet,
                                "Wrote {} to {}",
                                entry.path.display(),
                                output_path.display()
                            );
                        }
                        EntryKind::Symlink(e) => {
                            if output_path.exists() {
                                return;
                            }
                            let original_path = e.strip_prefix("/").unwrap_or(&e);
                            let _ = unix::fs::symlink(original_path, &output_path);
                            log!(
                                args.quiet,
                                "Wrote {} to {}",
                                entry.path.display(),
                                output_path.display()
                            );
                        }
                        _ => {}
                    };
                } else {
                    log!(args.quiet, "{}", entry.path.display());
                }
            });
        }
    }
}
