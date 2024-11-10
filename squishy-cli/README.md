# 🗜️ Squishy

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A convenient wrapper around the [backhand](https://github.com/wcampbell0x2a/backhand) library for reading and extracting files from SquashFS filesystems.

## Features

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

## Usage

The CLI tool provides convenient commands for working with AppImage files.

### Basic Commands

```bash
# Extract icon from an AppImage
squishy appimage path/to/app.AppImage --icon

# Extract desktop file
squishy appimage path/to/app.AppImage --desktop

# Extract AppStream metadata
squishy appimage path/to/app.AppImage --appstream

# Extract and save files to a specific directory
squishy appimage path/to/app.AppImage --icon --write /output/path

# Extract multiple resources at once
squishy appimage path/to/app.AppImage --icon --desktop --appstream --write

# Filter path by query
squishy appimage path/to/app.AppImage --filter "squishy" --icon --desktop --appstream --write

# Provide custom offset (it'd be calculated automatically if not provided)
# Appimage offset can be read using `path/to/app.AppImage --appimage-offset`
squishy appimage path/to/app.AppImage --offset 128128 --icon --desktop --appstream --write
```

### Command Options

- `--offset`: Custom offset (i.e. the size of ELF)
- `--filter`: Filter the files using provided query
- `--icon`: Extract application icon
- `--desktop`: Extract desktop entry file
- `--appstream`: Extract AppStream metadata
- `--write`: Write files to disk (optional path argument)

## License

This project is licensed under the [MIT] License - see the [LICENSE](LICENSE) file for details.
