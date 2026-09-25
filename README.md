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

# Review all supported tool caches and confirm cleanup
disk-cleaner cache

# Clean specific tool
disk-cleaner cache --tool npm
disk-cleaner cache --tool cargo --dry-run
```

## Supported Tools

Cache cleanup uses native commands where defined. Directory deletion is used only for tools without a native command. If a command is unavailable or fails, that tool's cache is left untouched and the command exits with an error.

`cache --dry-run` checks whether native commands are available and shows the action planned for each detected cache. A missing command is marked as skipped; it is never replaced with directory deletion. The dry run remains read-only but exits with an error when a detected cache cannot be safely cleaned. A native command may clean other caches managed by that tool beyond the paths shown in the scan, and may leave some matched content in place.

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

`build`, `dist`, `.cache`, `.npm`, and `.yarn` directories are not automatically classified as junk by directory scanning. `scan` shows the 50 largest matches; `clean` shows every full path before confirmation. If any path cannot be inspected or sized, the scan exits with an error and cleanup does not start.

## Safety and size estimates

`scan` and both `--dry-run` modes do not delete files. `clean` and `cache` show the targets and require `y` to proceed; pressing Enter or any other key cancels. An incomplete scan or an unsafe cache path cancels cleanup. Changed targets that fail revalidation are skipped and the command exits nonzero. Earlier successful deletions are not rolled back.

Directory cleanup and direct cache deletion use paths relative to directories opened during the scan. A replaced scan root or symlinked parent is rejected before directory cleanup. Directly deleted caches must match the tool's configured full path; cache paths containing symlinks are rejected. Native cleanup commands may affect caches outside the listed paths, so review the command shown by `cache --dry-run` before confirming.

All sizes are estimates based on file lengths, not measured free disk space. For direct deletion, the result uses the size measured before cleanup. For native commands, it uses the decrease in size of matched paths after the command; the command may clean more or less elsewhere.

## Scan Benchmark

On Linux or macOS, run the benchmark from the repository root:

```bash
scripts/benchmark-scan.sh 10000
```

The script builds a release binary, creates a temporary tree with the requested number of ordinary files and one tenth as many files inside a matched `node_modules` directory, times one scan, and removes the tree. Use the same count on both revisions when comparing performance. Timing is excluded from CI because it depends on the machine and filesystem.

## License

MIT
