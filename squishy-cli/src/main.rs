use appimage::AppImage;
use clap::Parser;
use cli::Args;

mod appimage;
mod cli;

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
        } => {
            if file.exists() {
                let appimage = match AppImage::new(filter.as_deref(), &file, offset) {
                    Ok(appimage) => appimage,
                    Err(e) => {
                        eprintln!("{}", e);
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

                let output_name = original_name
                    .then_some(None)
                    .or(Some(file.file_name()))
                    .unwrap();

                if desktop {
                    if let Some(desktop) = appimage.find_desktop() {
                        if let Some(ref write_path) = write_path {
                            appimage
                                .write(&desktop.path, write_path, output_name)
                                .unwrap();
                        } else {
                            println!("Desktop file: {}", desktop.path.display());
                        }
                    } else {
                        eprintln!("No desktop file found.");
                    };
                }
                if icon {
                    if let Some(icon) = appimage.find_icon() {
                        if let Some(ref write_path) = write_path {
                            appimage.write(&icon.path, write_path, output_name).unwrap();
                        } else {
                            println!("Icon: {}", icon.path.display());
                        }
                    } else {
                        eprintln!("No icon found.");
                    };
                }
                if appstream {
                    if let Some(icon) = appimage.find_appstream() {
                        if let Some(ref write_path) = write_path {
                            appimage.write(&icon.path, write_path, output_name).unwrap();
                        } else {
                            println!("Appstream file: {}", icon.path.display());
                        }
                    } else {
                        eprintln!("No appstream file found.");
                    };
                }
            }
        }
    }
}
