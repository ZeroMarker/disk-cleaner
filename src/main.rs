use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use walkdir::WalkDir;

const SUPPORTED_TOOLS: &[&str] = &[
    "uv",
    "npm",
    "pnpm",
    "yarn",
    "bun",
    "deno",
    "cargo",
    "go",
    "pip",
    "poetry",
    "conda",
    "pdm",
    "gem",
    "composer",
    "maven",
    "gradle",
    "hex",
    "pub",
    "nuget",
    "journalctl",
    "apt",
    "snap",
    "brew",
    "mise",
    "pacman",
    "dnf",
    "zypper",
    "flatpak",
    "docker",
    "winget",
    "vcpkg",
];

#[derive(Parser)]
#[command(
    name = "disk-cleaner",
    about = "Scan and safely remove disposable files, build artifacts, and tool caches",
    long_about = "A fast disk cleanup tool for finding and removing disposable files, build artifacts, temporary files, and package-manager caches.\n\nStart with a dry run to review what would be removed.",
    version,
    after_help = "Examples:\n  disk-cleaner scan --path ~/projects\n  disk-cleaner clean --path . --dry-run\n  disk-cleaner cache --dry-run\n  disk-cleaner cache --tool npm\n\nRun 'disk-cleaner <COMMAND> --help' for command-specific options."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Recursively find junk files and directories without deleting anything.
    #[command(
        long_about = "Recursively scan a directory for known junk files and directories. Results are sorted by reclaimable size.\n\nThis command is read-only.",
        after_help = "Examples:\n  disk-cleaner scan\n  disk-cleaner scan --path ~/projects"
    )]
    Scan {
        /// Directory to scan (defaults to the current directory).
        #[arg(short, long, default_value = ".", value_name = "DIR")]
        path: String,
    },
    /// Scan for junk, then optionally delete the listed items.
    #[command(
        long_about = "Scan a directory and ask for confirmation before deleting detected junk. Use --dry-run to preview the cleanup without changing files.\n\nMatched cache/build directories are treated as a single item and their contents are not listed separately.",
        after_help = "Examples:\n  disk-cleaner clean --path . --dry-run\n  disk-cleaner clean --path ~/projects"
    )]
    Clean {
        /// Directory to clean (defaults to the current directory).
        #[arg(short, long, default_value = ".", value_name = "DIR")]
        path: String,
        /// Show what would be deleted without changing anything.
        #[arg(short, long)]
        dry_run: bool,
    },
    /// Find and clean package-manager and system-tool caches.
    #[command(
        long_about = "Find caches for supported package managers and development tools. Native cleanup commands are preferred when available; otherwise, validated cache directories are removed.\n\nUse --dry-run first. Some system caches may require elevated permissions.",
        after_help = "Examples:\n  disk-cleaner cache --dry-run\n  disk-cleaner cache --tool npm --dry-run\n  disk-cleaner cache --tool cargo\n\nSupported tools include: uv, npm, pnpm, yarn, bun, deno, cargo, go, pip, poetry, conda, pdm, gem, composer, maven, gradle, hex, pub, nuget, apt, snap, brew, mise, pacman, dnf, zypper, winget, and vcpkg."
    )]
    Cache {
        /// Only clean this tool's cache; omit to scan all supported tools.
        #[arg(short, long, value_name = "TOOL")]
        tool: Option<String>,
        /// Show what would be cleaned without changing anything.
        #[arg(short, long)]
        dry_run: bool,
    },
}

struct JunkFile {
    path: PathBuf,
    size: u64,
    category: String,
}

fn is_junk(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Never classify symlinked directories as cleanup targets. A symlink can
    // point outside the scanned tree and must not become a recursive delete.
    let is_real_dir = fs::symlink_metadata(path)
        .map(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
        .unwrap_or(false);
    if is_real_dir {
        match name {
            "node_modules" | "__pycache__" | ".pytest_cache" | ".mypy_cache" | "target"
            | ".gradle" | ".cache" | ".npm" | ".yarn" | "dist" | "build" => {
                return Some("cache/build".into());
            }
            _ => {}
        }
    }

    match name {
        ".DS_Store" | "Thumbs.db" | "desktop.ini" => Some("system".into()),
        _ => match ext {
            "tmp" | "temp" | "swp" | "swo" | "bak" | "log" => Some("temp/log".into()),
            "cargo" | "gradle" | "hex" | "vcpkg" | "snap" | "pacman" | "winget" => None,
            _ => None,
        },
    }
}

fn dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

fn scan(dir: &str) -> Result<Vec<JunkFile>> {
    let mut junk = Vec::new();
    let mut entries = WalkDir::new(dir).into_iter();
    while let Some(entry) = entries.next() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) if error.depth() == 0 => return Err(error.into()),
            Err(error) => {
                eprintln!("  {} inaccessible path: {}", "skipped".yellow(), error);
                continue;
            }
        };
        if let Some(category) = is_junk(entry.path()) {
            let size = if entry.file_type().is_dir() {
                dir_size(entry.path())
            } else {
                entry.metadata().map(|metadata| metadata.len()).unwrap_or(0)
            };
            junk.push(JunkFile {
                path: entry.path().to_path_buf(),
                size,
                category,
            });

            // A matched directory is one cleanup item. Do not traverse its
            // contents, otherwise every nested file is reported as a second
            // item and later deletion attempts fail after the parent is gone.
            if entry.file_type().is_dir() {
                entries.skip_current_dir();
            }
        }
    }
    junk.sort_by_key(|item| std::cmp::Reverse(item.size));
    Ok(junk)
}

fn home_dir() -> PathBuf {
    dirs_or_home()
}

fn dirs_or_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn xdg_cache_dir(home: &Path) -> PathBuf {
    env_path("XDG_CACHE_HOME").unwrap_or_else(|| home.join(".cache"))
}

fn command_path(cmd: &str, args: &[&str]) -> Option<PathBuf> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

fn is_protected_path(path: &Path) -> bool {
    let protected = [
        Path::new("/"),
        Path::new("/bin"),
        Path::new("/boot"),
        Path::new("/dev"),
        Path::new("/etc"),
        Path::new("/home"),
        Path::new("/lib"),
        Path::new("/lib64"),
        Path::new("/opt"),
        Path::new("/proc"),
        Path::new("/root"),
        Path::new("/run"),
        Path::new("/sbin"),
        Path::new("/srv"),
        Path::new("/sys"),
        Path::new("/tmp"),
        Path::new("/usr"),
        Path::new("/var"),
    ];
    protected.contains(&path)
}

fn has_expected_cache_shape(tool: &str, path: &Path) -> bool {
    let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
    match tool {
        "uv" => matches!(name, "uv" | "cache"),
        "npm" => matches!(name, ".npm" | "npm" | "cache"),
        "pnpm" => name == "store",
        "yarn" => matches!(name, "yarn" | "cache"),
        "bun" => name == "cache",
        "deno" => matches!(name, "deno" | "cache"),
        "cargo" => matches!(name, "registry" | "git"),
        "go" => name == "go-build" || path.ends_with("pkg/mod"),
        "pip" => matches!(name, "pip" | "cache"),
        "poetry" => matches!(name, "pypoetry" | "cache"),
        "conda" => name == "pkgs",
        "pdm" => matches!(name, "pdm" | "cache"),
        "composer" => matches!(name, "composer" | "cache"),
        "maven" => name == "repository",
        "gradle" => name == "caches",
        "pub" => name == ".pub-cache",
        "nuget" => name == "packages",
        "mise" => matches!(name, "mise" | "cache"),
        "brew" => name == "Homebrew",
        "vcpkg" => matches!(name, "buildtrees" | "downloads" | "packages"),
        "winget" => name.eq_ignore_ascii_case("WinGet"),
        "gem" => name == "cache",
        "hex" => name == "packages",
        "apt" => path == Path::new("/var/cache/apt/archives"),
        "snap" => path == Path::new("/var/lib/snapd/cache"),
        "journalctl" => path == Path::new("/var/log/journal"),
        "pacman" => path == Path::new("/var/cache/pacman/pkg"),
        "dnf" => path == Path::new("/var/cache/dnf"),
        "zypper" => path == Path::new("/var/cache/zypp"),
        _ => true,
    }
}

fn is_safe_cache_path(tool: &str, path: &Path) -> bool {
    if !path.is_absolute() || is_protected_path(path) {
        return false;
    }

    let Ok(metadata) = fs::symlink_metadata(path) else {
        return false;
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return false;
    }

    let Ok(canonical) = path.canonicalize() else {
        return false;
    };
    if canonical != path {
        return false;
    }
    if is_protected_path(&canonical) || !has_expected_cache_shape(tool, &canonical) {
        return false;
    }

    // A cache directory must be below the user profile or nested sufficiently
    // deeply in the filesystem. The expected-shape check above still applies.
    let home = home_dir().canonicalize().ok();
    home.as_ref()
        .is_some_and(|home| canonical.starts_with(home) && canonical != *home)
        || canonical.components().count() >= 4
}

fn get_cache_dirs(tool: &str) -> Vec<(PathBuf, String)> {
    let home = home_dir();
    let xdg_cache = xdg_cache_dir(&home);
    match tool {
        "uv" => {
            let cache = env_path("UV_CACHE_DIR")
                .or_else(|| command_path("uv", &["cache", "dir"]))
                .unwrap_or_else(|| xdg_cache.join("uv"));
            if cache.exists() {
                vec![(cache, "uv cache".into())]
            } else {
                vec![]
            }
        }
        "npm" => {
            let cache = env_path("NPM_CONFIG_CACHE")
                .or_else(|| command_path("npm", &["config", "get", "cache"]))
                .unwrap_or_else(|| home.join(".npm"));
            if cache.exists() {
                vec![(cache, "npm cache".into())]
            } else {
                vec![]
            }
        }
        "cargo" => {
            let cargo_home = env_path("CARGO_HOME").unwrap_or_else(|| home.join(".cargo"));
            let registry = cargo_home.join("registry");
            let git = cargo_home.join("git");
            let mut dirs = Vec::new();
            if registry.exists() {
                dirs.push((registry, "cargo crate source cache".into()));
            }
            if git.exists() {
                dirs.push((git, "cargo git source cache".into()));
            }
            dirs
        }
        "journalctl" => {
            let log_dir = PathBuf::from("/var/log/journal");
            if log_dir.exists() {
                vec![(log_dir, "journalctl".into())]
            } else {
                vec![]
            }
        }
        "apt" => {
            let archives = PathBuf::from("/var/cache/apt/archives");
            let mut dirs = Vec::new();
            if archives.exists() {
                dirs.push((archives, "apt archives".into()));
            }
            dirs
        }
        "snap" => {
            let cache_dir = PathBuf::from("/var/lib/snapd/cache");
            let mut dirs = Vec::new();
            if cache_dir.exists() {
                dirs.push((cache_dir, "snap cache".into()));
            }
            dirs
        }
        "winget" => {
            // Packages contains installed portable applications, not disposable cache.
            let Some(local) = env_path("LOCALAPPDATA") else {
                return vec![];
            };
            let cache = local.join("Temp").join("WinGet");
            if cache.exists() {
                vec![(cache, "winget cache".into())]
            } else {
                vec![]
            }
        }
        "mise" => {
            let cache = env_path("MISE_CACHE_DIR").unwrap_or_else(|| xdg_cache.join("mise"));
            if cache.exists() {
                vec![(cache, "mise cache".into())]
            } else {
                vec![]
            }
        }
        "brew" => {
            let mut dirs = Vec::new();
            let cache = command_path("brew", &["--cache"]).unwrap_or_else(|| {
                if cfg!(target_os = "macos") {
                    home.join("Library/Caches/Homebrew")
                } else {
                    xdg_cache.join("Homebrew")
                }
            });
            if cache.exists() {
                dirs.push((cache, "brew cache".into()));
            }
            dirs
        }
        "pip" => {
            let cache = env_path("PIP_CACHE_DIR")
                .or_else(|| command_path("pip", &["cache", "dir"]))
                .unwrap_or_else(|| xdg_cache.join("pip"));
            if cache.exists() {
                vec![(cache, "pip cache".into())]
            } else {
                vec![]
            }
        }
        "poetry" => {
            let cache = env_path("POETRY_CACHE_DIR").unwrap_or_else(|| xdg_cache.join("pypoetry"));
            if cache.exists() {
                vec![(cache, "poetry cache".into())]
            } else {
                vec![]
            }
        }
        "conda" => {
            let cache = env_path("CONDA_PKGS_DIRS")
                .and_then(|p| std::env::split_paths(&p).next())
                .unwrap_or_else(|| home.join(".conda").join("pkgs"));
            if cache.exists() {
                vec![(cache, "conda pkgs".into())]
            } else {
                vec![]
            }
        }
        "pdm" => {
            let cache = env_path("PDM_CACHE_DIR").unwrap_or_else(|| xdg_cache.join("pdm"));
            if cache.exists() {
                vec![(cache, "pdm cache".into())]
            } else {
                vec![]
            }
        }
        "go" => {
            let gopath = env_path("GOPATH")
                .or_else(|| command_path("go", &["env", "GOPATH"]))
                .unwrap_or_else(|| home.join("go"));
            let cache = env_path("GOCACHE")
                .or_else(|| command_path("go", &["env", "GOCACHE"]))
                .unwrap_or_else(|| xdg_cache.join("go-build"));
            let mod_cache = env_path("GOMODCACHE")
                .or_else(|| command_path("go", &["env", "GOMODCACHE"]))
                .unwrap_or_else(|| gopath.join("pkg").join("mod"));
            let mut dirs = Vec::new();
            if cache.exists() {
                dirs.push((cache, "go build cache".into()));
            }
            if mod_cache.exists() {
                dirs.push((mod_cache, "go mod cache".into()));
            }
            dirs
        }
        "gem" => {
            let cache = env_path("GEM_HOME")
                .or_else(|| command_path("gem", &["env", "home"]))
                .unwrap_or_else(|| home.join(".gem"))
                .join("cache");
            if cache.exists() {
                vec![(cache, "gem cache".into())]
            } else {
                vec![]
            }
        }
        "maven" => {
            let cache =
                env_path("MAVEN_REPO_LOCAL").unwrap_or_else(|| home.join(".m2").join("repository"));
            if cache.exists() {
                vec![(cache, "maven dependency cache".into())]
            } else {
                vec![]
            }
        }
        "gradle" => {
            let cache = env_path("GRADLE_USER_HOME")
                .unwrap_or_else(|| home.join(".gradle"))
                .join("caches");
            if cache.exists() {
                vec![(cache, "gradle caches".into())]
            } else {
                vec![]
            }
        }
        "pnpm" => {
            let store = home.join(".local").join("share").join("pnpm").join("store");
            if store.exists() {
                vec![(store, "pnpm store".into())]
            } else {
                vec![]
            }
        }
        "yarn" => {
            let candidates = [xdg_cache.join("yarn"), home.join(".yarn/berry/cache")];
            candidates
                .into_iter()
                .filter(|p| p.exists())
                .map(|p| (p, "yarn cache".into()))
                .collect()
        }
        "bun" => {
            let cache = home.join(".bun").join("install").join("cache");
            if cache.exists() {
                vec![(cache, "bun cache".into())]
            } else {
                vec![]
            }
        }
        "deno" => {
            let cache = env_path("DENO_DIR").unwrap_or_else(|| xdg_cache.join("deno"));
            if cache.exists() {
                vec![(cache, "deno cache".into())]
            } else {
                vec![]
            }
        }
        "composer" => {
            let cache = env_path("COMPOSER_CACHE_DIR")
                .or_else(|| command_path("composer", &["config", "cache-dir", "--global"]))
                .unwrap_or_else(|| xdg_cache.join("composer"));
            if cache.exists() {
                vec![(cache, "composer cache".into())]
            } else {
                vec![]
            }
        }
        "docker" => {
            // /var/lib/docker includes live data and cannot be used as a reclaimable-size estimate.
            vec![]
        }
        "hex" => {
            let cache = env_path("HEX_HOME")
                .unwrap_or_else(|| home.join(".hex"))
                .join("packages");
            if cache.exists() {
                vec![(cache, "hex cache".into())]
            } else {
                vec![]
            }
        }
        "pub" => {
            let cache = env_path("PUB_CACHE").unwrap_or_else(|| home.join(".pub-cache"));
            if cache.exists() {
                vec![(cache, "pub cache".into())]
            } else {
                vec![]
            }
        }
        "nuget" => {
            let cache =
                env_path("NUGET_PACKAGES").unwrap_or_else(|| home.join(".nuget").join("packages"));
            if cache.exists() {
                vec![(cache, "nuget packages".into())]
            } else {
                vec![]
            }
        }
        "vcpkg" => {
            let root =
                env_path("VCPKG_ROOT").unwrap_or_else(|| PathBuf::from("/usr/local/share/vcpkg"));
            let mut dirs = Vec::new();
            for subdir in &["buildtrees", "downloads", "packages"] {
                let cache = root.join(subdir);
                if cache.exists() {
                    dirs.push((cache, format!("vcpkg {}", subdir)));
                }
            }
            dirs
        }
        "zypper" => {
            let cache = PathBuf::from("/var/cache/zypp");
            if cache.exists() {
                vec![(cache, "zypper cache".into())]
            } else {
                vec![]
            }
        }
        "dnf" => {
            let cache = PathBuf::from("/var/cache/dnf");
            if cache.exists() {
                vec![(cache, "dnf cache".into())]
            } else {
                vec![]
            }
        }
        "pacman" => {
            let cache = PathBuf::from("/var/cache/pacman/pkg");
            if cache.exists() {
                vec![(cache, "pacman cache".into())]
            } else {
                vec![]
            }
        }
        "flatpak" => {
            // `flatpak uninstall --unused` manages installations, not a cache directory.
            vec![]
        }
        _ => vec![],
    }
}

fn scan_cache(tools: &[String]) -> Result<Vec<JunkFile>> {
    let mut junk = Vec::new();
    for tool in tools {
        for (dir, category) in get_cache_dirs(tool) {
            if is_safe_cache_path(tool, &dir) {
                let size = dir_size(&dir);
                if size > 0 {
                    junk.push(JunkFile {
                        path: dir,
                        size,
                        category,
                    });
                }
            } else if dir.exists() {
                eprintln!(
                    "  {} unsafe or unexpected {} path: {}",
                    "skipped".yellow(),
                    tool,
                    dir.display()
                );
            }
        }
    }
    junk.sort_by_key(|item| std::cmp::Reverse(item.size));
    Ok(junk)
}

fn get_clean_cmd(tool: &str) -> Option<(&'static str, Vec<&'static str>)> {
    match tool {
        "npm" => Some(("npm", vec!["cache", "clean", "--force"])),
        "pnpm" => Some(("pnpm", vec!["store", "prune"])),
        "yarn" => Some(("yarn", vec!["cache", "clean"])),
        "bun" => Some(("bun", vec!["pm", "cache", "rm"])),
        "deno" => Some(("deno", vec!["clean"])),
        "go" => Some(("go", vec!["clean", "-cache", "-modcache"])),
        "pip" => Some(("pip", vec!["cache", "purge"])),
        "poetry" => Some(("poetry", vec!["cache", "clear", "--all", "."])),
        "conda" => Some(("conda", vec!["clean", "--all", "-y"])),
        "pdm" => Some(("pdm", vec!["cache", "clear"])),
        "composer" => Some(("composer", vec!["clear-cache"])),
        "pub" => Some(("dart", vec!["pub", "cache", "clean"])),
        "nuget" => Some(("dotnet", vec!["nuget", "locals", "all", "--clear"])),
        "apt" => Some(("apt-get", vec!["clean"])),
        "brew" => Some(("brew", vec!["cleanup", "--cache"])),
        "mise" => Some(("mise", vec!["cache", "clear"])),
        "dnf" => Some(("dnf", vec!["clean", "all"])),
        "zypper" => Some(("zypper", vec!["clean"])),
        "journalctl" => Some(("journalctl", vec!["--vacuum-time=3d"])),
        "uv" => Some(("uv", vec!["cache", "clean"])),
        _ => None,
    }
}

fn run_clean_cmd(tool: &str, dry_run: bool) -> Result<bool> {
    let Some((cmd, args)) = get_clean_cmd(tool) else {
        return Ok(false);
    };
    if dry_run {
        println!("  {} {} {}", "[dry-run]".yellow(), cmd, args.join(" "));
        return Ok(true);
    }
    let output = match Command::new(cmd).args(&args).output() {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            println!("  {} command not found: {cmd}", "fallback".yellow());
            return Ok(false);
        }
        Err(error) => return Err(error.into()),
    };
    if output.status.success() {
        let out = String::from_utf8_lossy(&output.stderr);
        if out.is_empty() {
            let out = String::from_utf8_lossy(&output.stdout);
            for line in out.lines() {
                println!("  {}", line.dimmed());
            }
        } else {
            for line in out.lines() {
                println!("  {}", line.dimmed());
            }
        }
        Ok(true)
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        println!(
            "  {} {} {} ({})",
            "failed".red(),
            cmd,
            args.join(" "),
            err.trim()
        );
        Ok(false)
    }
}

fn clean_cache(junk: &[JunkFile], dry_run: bool) -> Result<()> {
    let mut cleaned = 0u64;
    let mut cmd_cleaned = std::collections::HashSet::new();
    for item in junk {
        let tool = item.category.split_whitespace().next().unwrap_or("");
        if get_clean_cmd(tool).is_some() {
            if cmd_cleaned.insert(tool.to_string()) {
                if run_clean_cmd(tool, dry_run)? {
                    if !dry_run {
                        cleaned += reclaimed_by_command(junk, tool);
                    }
                    continue;
                }
                cmd_cleaned.remove(tool);
            } else {
                continue;
            }
        }
        if dry_run {
            println!(
                "  {} remove directory recursively: {}",
                "[dry-run]".yellow(),
                item.path.display()
            );
        } else {
            if !is_safe_cache_path(tool, &item.path) {
                println!(
                    "  {} unsafe or changed path: {}",
                    "skipped".yellow(),
                    item.path.display()
                );
                continue;
            }
            let res = fs::remove_dir_all(&item.path);
            match res {
                Ok(_) => {
                    cleaned += item.size;
                    println!("  {} {}", "removed".green(), item.path.display());
                }
                Err(e) => {
                    println!("  {} {} ({})", "failed".red(), item.path.display(), e);
                }
            }
        }
    }

    if dry_run {
        println!(
            "\n{} Dry run complete. {} would be freed.",
            "Done!".bold().cyan(),
            format_size(junk.iter().map(|j| j.size).sum::<u64>()).yellow()
        );
    } else {
        println!(
            "\n{} Cleaned up {} of disk space.",
            "Done!".bold().green(),
            format_size(cleaned).yellow()
        );
    }
    Ok(())
}

fn reclaimed_by_command(junk: &[JunkFile], tool: &str) -> u64 {
    junk.iter()
        .filter(|item| item.category.split_whitespace().next() == Some(tool))
        .map(|item| {
            let remaining = if item.path.is_dir() {
                dir_size(&item.path)
            } else {
                0
            };
            item.size.saturating_sub(remaining)
        })
        .sum()
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn display_results(junk: &[JunkFile]) {
    if junk.is_empty() {
        println!("{}", "No junk files found.".green());
        return;
    }

    let total: u64 = junk.iter().map(|j| j.size).sum();
    println!(
        "\n{} Found {} items totaling {}\n",
        "Scan Results:".bold().cyan(),
        junk.len().to_string().yellow(),
        format_size(total).red().bold()
    );

    println!(
        "  {:<60} {:>10}  {}",
        "Path".bold(),
        "Size".bold(),
        "Category".bold()
    );
    println!("  {}", "-".repeat(90));

    for item in junk.iter().take(50) {
        let path_str = item.path.display().to_string();
        let display = truncate_path(&path_str, 58);
        println!(
            "  {:<60} {:>10}  {}",
            display,
            format_size(item.size),
            item.category
        );
    }

    if junk.len() > 50 {
        println!(
            "\n  {} and {} more items...",
            "...".dimmed(),
            (junk.len() - 50).to_string().yellow()
        );
    }

    println!(
        "\n  {} {}",
        "Total reclaimable:".bold(),
        format_size(total).red().bold()
    );
}

fn truncate_path(path: &str, max_chars: usize) -> String {
    let char_count = path.chars().count();
    if char_count <= max_chars {
        return path.to_owned();
    }

    let suffix_len = max_chars.saturating_sub(3);
    let suffix: String = path.chars().skip(char_count - suffix_len).collect();
    format!("...{suffix}")
}

fn clean(junk: &[JunkFile], dry_run: bool) -> Result<()> {
    let mut cleaned = 0u64;
    for item in junk {
        if dry_run {
            println!("  {} {}", "[dry-run]".yellow(), item.path.display());
        } else {
            let metadata = match fs::symlink_metadata(&item.path) {
                Ok(metadata) => metadata,
                Err(error) => {
                    println!(
                        "  {} {} ({})",
                        "skipped".yellow(),
                        item.path.display(),
                        error
                    );
                    continue;
                }
            };
            if metadata.file_type().is_symlink() {
                println!("  {} symlink: {}", "skipped".yellow(), item.path.display());
                continue;
            }
            let res = if metadata.is_dir() {
                fs::remove_dir_all(&item.path)
            } else {
                fs::remove_file(&item.path)
            };
            match res {
                Ok(_) => {
                    cleaned += item.size;
                    println!("  {} {}", "removed".green(), item.path.display());
                }
                Err(e) => {
                    println!("  {} {} ({})", "failed".red(), item.path.display(), e);
                }
            }
        }
    }

    if dry_run {
        println!(
            "\n{} Dry run complete. {} would be freed.",
            "Done!".bold().cyan(),
            format_size(junk.iter().map(|j| j.size).sum::<u64>()).yellow()
        );
    } else {
        println!(
            "\n{} Cleaned up {} of disk space.",
            "Done!".bold().green(),
            format_size(cleaned).yellow()
        );
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { path } => {
            println!("{} {}", "Scanning".bold().cyan(), path);
            let junk = scan(&path)?;
            display_results(&junk);
        }
        Commands::Clean { path, dry_run } => {
            println!("{} {}", "Scanning".bold().cyan(), path);
            let junk = scan(&path)?;
            if junk.is_empty() {
                println!("{}", "Nothing to clean.".green());
                return Ok(());
            }
            display_results(&junk);

            if !dry_run {
                println!(
                    "\n{} This will permanently delete the files listed above.",
                    "Warning!".red().bold()
                );
                println!("Proceed? [y/N] ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            clean(&junk, dry_run)?;
        }
        Commands::Cache { tool, dry_run } => {
            let tools = match tool {
                Some(t) if SUPPORTED_TOOLS.contains(&t.as_str()) => vec![t],
                Some(t) => anyhow::bail!(
                    "unsupported tool '{t}'; supported tools: {}",
                    SUPPORTED_TOOLS.join(", ")
                ),
                None => SUPPORTED_TOOLS.iter().map(|tool| (*tool).into()).collect(),
            };

            println!(
                "{} {}",
                "Scanning caches for:".bold().cyan(),
                tools.join(", ")
            );
            let junk = scan_cache(&tools)?;
            if junk.is_empty() {
                println!("{}", "No caches found.".green());
                return Ok(());
            }
            display_results(&junk);

            if !dry_run {
                println!(
                    "\n{} This will permanently delete the caches listed above.",
                    "Warning!".red().bold()
                );
                println!("Proceed? [y/N] ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            clean_cache(&junk, dry_run)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_cache_resources_are_not_cleaned_by_commands() {
        for tool in ["docker", "flatpak", "winget", "gem", "maven"] {
            assert!(
                get_clean_cmd(tool).is_none(),
                "unexpected command for {tool}"
            );
        }
    }

    #[test]
    fn apt_uses_the_matching_archive_cleanup_command() {
        assert_eq!(get_clean_cmd("apt"), Some(("apt-get", vec!["clean"])));
    }

    #[test]
    fn formats_binary_sizes() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1_048_576), "1.00 MB");
    }

    #[test]
    fn truncates_utf8_paths_at_character_boundaries() {
        let path = format!("/tmp/{}/artifact.log", "项目".repeat(40));
        let display = truncate_path(&path, 58);

        assert_eq!(display.chars().count(), 58);
        assert!(display.starts_with("..."));
        assert!(display.ends_with("artifact.log"));
    }

    #[test]
    fn command_reclaim_uses_post_cleanup_size() {
        let root =
            std::env::temp_dir().join(format!("disk-cleaner-reclaim-test-{}", std::process::id()));
        let cache = root.join("cache");
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("remaining.bin"), vec![0u8; 25]).unwrap();
        let junk = vec![JunkFile {
            path: cache,
            size: 100,
            category: "npm cache".into(),
        }];

        assert_eq!(reclaimed_by_command(&junk, "npm"), 75);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cli_exposes_package_version() {
        use clap::CommandFactory;

        assert_eq!(
            Cli::command().get_version(),
            Some(env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn rejects_protected_and_mismatched_cache_paths() {
        assert!(!is_safe_cache_path("npm", Path::new("/")));
        assert!(!is_safe_cache_path("npm", &home_dir()));
        assert!(!has_expected_cache_shape(
            "winget",
            Path::new("/Users/test/AppData/Local/Microsoft/WinGet/Packages")
        ));
        assert!(has_expected_cache_shape(
            "winget",
            Path::new("/Users/test/AppData/Local/Temp/WinGet")
        ));
    }

    #[test]
    fn scan_aggregates_and_skips_contents_of_junk_directories() {
        let root = std::env::temp_dir().join(format!(
            "disk-cleaner-test-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("scan")
        ));
        let target = root.join("target");
        fs::create_dir_all(target.join("debug")).unwrap();
        fs::write(target.join("debug/artifact.bin"), vec![0u8; 128]).unwrap();
        fs::write(root.join("notes.log"), vec![0u8; 7]).unwrap();

        let results = scan(root.to_str().unwrap()).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(
            results
                .iter()
                .find(|item| item.path == target)
                .map(|item| item.size),
            Some(128)
        );
        assert!(
            !results
                .iter()
                .any(|item| item.path.ends_with("artifact.bin"))
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cargo_uses_directory_cleanup_fallback() {
        assert!(get_clean_cmd("cargo").is_none());
        assert!(has_expected_cache_shape(
            "cargo",
            Path::new("/home/user/.cargo/registry")
        ));
        assert!(has_expected_cache_shape(
            "cargo",
            Path::new("/home/user/.cargo/git")
        ));
    }
}
