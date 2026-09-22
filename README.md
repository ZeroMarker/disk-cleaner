# disk-cleaner

A fast disk cleanup tool written in Rust.

## Features

- Scan directories for known build caches and system files; optionally include temporary, swap, backup, and log files
- Clean detected caches for 29 package managers and development tools
- Use native cleanup commands (e.g. `npm cache clean`, `pip cache purge`) where defined; an unavailable or failed command does not trigger directory deletion
- Dry-run mode to preview before deleting
- Colorful terminal output
- Cross-platform support (Linux, macOS, Windows)

## Install

```bash
cargo install --path .
```

## Usage

### Directory Scan

```bash
# Scan current directory for junk files
disk-cleaner scan

# Scan a specific path
disk-cleaner scan --path /home/user

# Include temporary, swap, backup, and log files
disk-cleaner scan --path /home/user --include-files

# Clean junk files with confirmation
disk-cleaner clean --path /home/user

# Dry run (preview only)
disk-cleaner clean --path /home/user --dry-run

# Preview those additional file types before deleting them
disk-cleaner clean --path /home/user --include-files --dry-run
```

### Cache Cleanup

```bash
# Scan all tool caches
disk-cleaner cache --dry-run

# Clean all tool caches
disk-cleaner cache

# Clean specific tool
disk-cleaner cache --tool npm
disk-cleaner cache --tool cargo --dry-run
```

## Supported Tools

Cache cleanup uses native commands where defined. Directory deletion is used only for tools without a native command. If a command is unavailable or fails, that tool's cache is left untouched and the command exits with an error.

| Category | Tools | Cleanup Method |
|----------|-------|----------------|
| **Node.js** | npm, pnpm, yarn, bun | native command |
| **Node.js** | deno | native command |
| **Python** | pip, poetry, conda, pdm, uv | native command |
| **Rust** | cargo | directory |
| **Go** | go | native command (`go clean`) |
| **Ruby** | gem | directory |
| **PHP** | composer | native command |
| **Java** | maven | directory |
| **Gradle** | gradle | directory |
| **Elixir** | hex | directory |
| **Dart/Flutter** | pub | native command |
| **.NET** | nuget | native command |
| **C/C++** | vcpkg | directory |
| **System** | apt, dnf, zypper, brew | native command |
| **System** | pacman, snap, winget | directory |
| **Runtime** | mise | native command |
| **Logs** | journalctl | native command |

Docker and Flatpak are not supported by `cache`; use their own tools for manual cleanup.

## Detected Junk (Directory Scan)

| Category     | Examples                                  |
|-------------|-------------------------------------------|
| cache/build | `node_modules`, `__pycache__`, `.pytest_cache`, `.mypy_cache`, `.gradle`, Rust `target` beside `Cargo.toml` |
| system      | `.DS_Store`, `Thumbs.db`, `desktop.ini`   |
| temp/log (opt-in) | `*.tmp`, `*.bak`, `*.log`, `*.swp`        |

`build`, `dist`, `.cache`, `.npm`, and `.yarn` directories are not automatically classified as junk by directory scanning. Review every path shown by `clean` before confirming deletion. Reported sizes are estimates based on file lengths, not measured free disk space.

## License

MIT
