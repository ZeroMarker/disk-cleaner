# disk-cleaner

A fast disk cleanup tool written in Rust.

## Features

- Scan directories for junk files (cache, temp, logs, system files)
- Recognize 30+ package managers and build tools; clean detected caches where supported
- Uses native cleanup commands (e.g. `npm cache clean`, `pip cache purge`) where available; a failed command does not trigger directory deletion
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

Cache cleanup uses native commands where available. Directory deletion is used for tools without a native command, or when the command is unavailable. If a command runs and fails, that tool's cache is left untouched.

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
| **Container** | docker | manual cleanup only |
| **System** | flatpak | manual cleanup only |
| **Logs** | journalctl | native command |

## Detected Junk (Directory Scan)

| Category     | Examples                                  |
|-------------|-------------------------------------------|
| cache/build | `node_modules`, `__pycache__`, `.cache`, Rust `target` beside `Cargo.toml` |
| system      | `.DS_Store`, `Thumbs.db`, `desktop.ini`   |
| temp/log    | `*.tmp`, `*.bak`, `*.log`, `*.swp`        |

`build` and `dist` directories are not automatically classified as junk. Review every path shown by `clean` before confirming deletion.

## License

MIT
