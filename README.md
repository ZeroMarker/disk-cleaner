# disk-cleaner

A fast disk cleanup tool written in Rust.

## Features

- Scan directories for junk files (cache, temp, logs, system files)
- Clean caches from 30+ package managers and build tools
- Uses native cleanup commands (e.g. `npm cache clean`, `pip cache purge`) where available
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

Cache cleanup uses native commands where available, falling back to directory deletion otherwise.

| Category | Tools | Cleanup Method |
|----------|-------|----------------|
| **Node.js** | npm, pnpm, yarn, bun | native command |
| **Node.js** | deno | directory |
| **Python** | pip, poetry, conda, pdm, uv | native command |
| **Rust** | cargo | directory |
| **Go** | go | native command (`go clean`) |
| **Ruby** | gem | native command |
| **PHP** | composer | native command |
| **Java** | maven | native command |
| **Gradle** | gradle | directory |
| **Elixir** | hex | directory |
| **Dart/Flutter** | pub | native command |
| **.NET** | nuget | native command |
| **C/C++** | vcpkg | directory |
| **System** | apt, dnf, zypper, brew, flatpak | native command |
| **System** | pacman, snap, winget | directory |
| **Runtime** | mise | native command |
| **Container** | docker | native command (`docker system prune`) |
| **Logs** | journalctl | native command |

## Detected Junk (Directory Scan)

| Category     | Examples                                  |
|-------------|-------------------------------------------|
| cache/build | `node_modules`, `__pycache__`, `target`, `.cache` |
| system      | `.DS_Store`, `Thumbs.db`, `desktop.ini`   |
| temp/log    | `*.tmp`, `*.bak`, `*.log`, `*.swp`        |

## License

MIT
