# 🗜️ Squishy

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)


A convenient wrapper around the [backhand](https://github.com/wcampbell0x2a/backhand) library for reading and extracting files from SquashFS filesystems. Provides both a library and CLI tool.

## Features

- 📚 **Library Features**
  - Read and extract files from SquashFS filesystems
  - Traverse filesystem entries
  - Handle symlinks with cycle detection
  - Search for files using custom predicates

- 🛠️ **CLI Features**
  - Extract AppImage resources:
    - Icon files (PNG/SVG)
    - Desktop entries
    - AppStream metadata
  - Flexible output options

## Installation

### From crates.io

```bash
cargo install squishy-cli
```

### From source

```bash
git clone https://github.com/pkgforge/squishy-rs
cd squishy-rs
cargo install --path squishy-cli
```

## Library Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
squishy = "0.1.0"
```

### Example

```rust
use squishy::SquashFS;
use std::path::Path;

// Open a SquashFS file
let squashfs = SquashFS::from_path(&Path::new("example.squashfs"))?;

// List all entries
for entry in squashfs.entries() {
    println!("{}", entry.path.display());
}

// Read a specific file
let contents = squashfs.read_file("path/to/file.txt")?;

// Extract a file
squashfs.write_file("source/path.txt", "destination/path.txt")?;
```

## CLI Usage

The CLI tool provides convenient commands for working with AppImage files.

### Basic Commands

```bash
# Extract icon from an AppImage
squishy appimage --app myapp --file path/to/app.AppImage --icon

# Extract desktop file
squishy appimage --app myapp --file path/to/app.AppImage --desktop

# Extract AppStream metadata
squishy appimage --app myapp --file path/to/app.AppImage --appstream

# Extract and save files to a specific directory
squishy appimage --app myapp --file path/to/app.AppImage --icon --write /output/path

# Extract multiple resources at once
squishy appimage --app myapp --file path/to/app.AppImage --icon --desktop --appstream --write
```

### Command Options

- `--app`: Name of the application (required)
- `--file`: Path to the AppImage file (required)
- `--icon`: Extract application icon
- `--desktop`: Extract desktop entry file
- `--appstream`: Extract AppStream metadata
- `--write`: Write files to disk (optional path argument)

## License

This project is licensed under the [MIT] License - see the [LICENSE](LICENSE) file for details.