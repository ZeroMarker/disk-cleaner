# disk-cleaner

A fast disk cleanup tool written in Rust.

## Features

- Scan directories for junk files (cache, temp, logs, system files)
- Clean caches from 30+ package managers and build tools
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

# Clean junk files with confirmation
disk-cleaner clean --path /home/user

# Dry run (preview only)
disk-cleaner clean --path /home/user --dry-run
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

| Category | Tools |
|----------|-------|
| **Node.js** | npm, pnpm, yarn, bun, deno |
| **Python** | pip, poetry, conda, pdm, uv |
| **Rust** | cargo |
| **Go** | go (build cache + modules) |
| **Ruby** | gem |
| **PHP** | composer |
| **Java** | maven, gradle |
| **Elixir** | hex |
| **Dart/Flutter** | pub |
| **.NET** | nuget |
| **C/C++** | vcpkg |
| **System** | apt, dnf, pacman, zypper, snap, flatpak, brew, winget |
| **Runtime** | mise |
| **Container** | docker |
| **Logs** | journalctl |

## Detected Junk (Directory Scan)

| Category     | Examples                                  |
|-------------|-------------------------------------------|
| cache/build | `node_modules`, `__pycache__`, `target`, `.cache` |
| system      | `.DS_Store`, `Thumbs.db`, `desktop.ini`   |
| temp/log    | `*.tmp`, `*.bak`, `*.log`, `*.swp`        |

## License

MIT
