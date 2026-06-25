# disk-cleaner

A fast disk cleanup tool written in Rust.

## Features

- Scan directories for junk files (cache, temp, logs, system files)
- Dry-run mode to preview before deleting
- Colorful terminal output
- Cross-platform support

## Install

```bash
cargo install --path .
```

## Usage

```bash
# Scan current directory
disk-cleaner scan

# Scan a specific path
disk-cleaner scan --path /home/user

# Clean with confirmation prompt
disk-cleaner clean --path /home/user

# Dry run (preview only)
disk-cleaner clean --path /home/user --dry-run
```

## Detected Junk

| Category     | Examples                                  |
|-------------|-------------------------------------------|
| cache/build | `node_modules`, `__pycache__`, `target`, `.cache` |
| system      | `.DS_Store`, `Thumbs.db`, `desktop.ini`   |
| temp/log    | `*.tmp`, `*.bak`, `*.log`, `*.swp`        |

## License

MIT
