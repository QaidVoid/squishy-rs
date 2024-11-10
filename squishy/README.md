# 🗜️ Squishy

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A convenient wrapper around the [backhand](https://github.com/wcampbell0x2a/backhand) library for reading and extracting files from SquashFS filesystems. Provides both a library and CLI tool.

## Features

- Read and extract files from SquashFS filesystems
- Traverse filesystem entries
- Handle symlinks with cycle detection
- Search for files using custom predicates

## Usage

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

## License

This project is licensed under the [MIT] License - see the [LICENSE](LICENSE) file for details.
