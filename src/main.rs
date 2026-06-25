use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "disk-cleaner", about = "A fast disk cleanup tool written in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Scan {
        #[arg(short, long, default_value = ".")]
        path: String,
    },
    Clean {
        #[arg(short, long, default_value = ".")]
        path: String,
        #[arg(short, long)]
        dry_run: bool,
    },
    Cache {
        #[arg(short, long)]
        tool: Option<String>,
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

    if path.is_dir() {
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
            _ => None,
        },
    }
}

fn dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().map(|m| m.is_file()).unwrap_or(false))
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

fn scan(dir: &str) -> Result<Vec<JunkFile>> {
    let mut junk = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if let Some(category) = is_junk(entry.path()) {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            junk.push(JunkFile {
                path: entry.path().to_path_buf(),
                size,
                category,
            });
        }
    }
    junk.sort_by(|a, b| b.size.cmp(&a.size));
    Ok(junk)
}

fn home_dir() -> PathBuf {
    dirs_or_home()
}

fn dirs_or_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/root"))
}

fn get_cache_dirs(tool: &str) -> Vec<(PathBuf, String)> {
    let home = home_dir();
    match tool {
        "uv" => {
            let cache = home.join(".cache").join("uv");
            if cache.exists() {
                vec![(cache, "uv cache".into())]
            } else {
                vec![]
            }
        }
        "npm" => {
            let cache = home.join(".npm");
            if cache.exists() {
                vec![(cache, "npm cache".into())]
            } else {
                vec![]
            }
        }
        "cargo" => {
            let registry = home.join(".cargo").join("registry");
            let git = home.join(".cargo").join("git");
            let mut dirs = Vec::new();
            if registry.exists() {
                dirs.push((registry, "cargo registry".into()));
            }
            if git.exists() {
                dirs.push((git, "cargo git".into()));
            }
            dirs
        }
        "journalctl" => {
            let log_dir = PathBuf::from("/var/log/journal");
            if log_dir.exists() {
                vec![(log_dir, "journalctl logs".into())]
            } else {
                vec![]
            }
        }
        "apt" => {
            let archives = PathBuf::from("/var/cache/apt/archives");
            let lists = PathBuf::from("/var/lib/apt/lists");
            let mut dirs = Vec::new();
            if archives.exists() {
                dirs.push((archives, "apt archives".into()));
            }
            if lists.exists() {
                dirs.push((lists, "apt lists".into()));
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
            let local = std::env::var("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/tmp"));
            let cache = local.join("Microsoft/WinGet/Packages");
            if cache.exists() {
                vec![(cache, "winget cache".into())]
            } else {
                vec![]
            }
        }
        "mise" => {
            let data = home.join(".local").join("share").join("mise");
            let cache = home.join(".cache").join("mise");
            let mut dirs = Vec::new();
            if cache.exists() {
                dirs.push((cache, "mise cache".into()));
            }
            if data.exists() {
                dirs.push((data, "mise data".into()));
            }
            dirs
        }
        "brew" => {
            let mut dirs = Vec::new();
            let linux_cache = PathBuf::from("/home/linuxbrew/.cache/Homebrew");
            let mac_cache = home.join("Library/Caches/Homebrew");
            if linux_cache.exists() {
                dirs.push((linux_cache, "brew cache".into()));
            }
            if mac_cache.exists() {
                dirs.push((mac_cache, "brew cache".into()));
            }
            dirs
        }
        "pip" => {
            let cache = home.join(".cache").join("pip");
            if cache.exists() {
                vec![(cache, "pip cache".into())]
            } else {
                vec![]
            }
        }
        "poetry" => {
            let cache = home.join(".cache").join("pypoetry");
            if cache.exists() {
                vec![(cache, "poetry cache".into())]
            } else {
                vec![]
            }
        }
        "conda" => {
            let cache = home.join(".conda").join("pkgs");
            if cache.exists() {
                vec![(cache, "conda pkgs".into())]
            } else {
                vec![]
            }
        }
        "pdm" => {
            let cache = home.join(".cache").join("pdm");
            if cache.exists() {
                vec![(cache, "pdm cache".into())]
            } else {
                vec![]
            }
        }
        "go" => {
            let gopath = std::env::var("GOPATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home.join("go"));
            let cache = home.join(".cache").join("go-build");
            let mod_cache = gopath.join("pkg").join("mod");
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
            let cache = home.join(".gem").join("cache");
            if cache.exists() {
                vec![(cache, "gem cache".into())]
            } else {
                vec![]
            }
        }
        "maven" => {
            let cache = home.join(".m2").join("repository");
            if cache.exists() {
                vec![(cache, "maven repo".into())]
            } else {
                vec![]
            }
        }
        "gradle" => {
            let cache = home.join(".gradle").join("caches");
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
            let cache = home.join(".cache").join("yarn");
            if cache.exists() {
                vec![(cache, "yarn cache".into())]
            } else {
                vec![]
            }
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
            let cache = home.join(".cache").join("deno");
            if cache.exists() {
                vec![(cache, "deno cache".into())]
            } else {
                vec![]
            }
        }
        "composer" => {
            let cache = home.join(".cache").join("composer");
            if cache.exists() {
                vec![(cache, "composer cache".into())]
            } else {
                vec![]
            }
        }
        "docker" => {
            let root = PathBuf::from("/var/lib/docker");
            if root.exists() {
                vec![(root, "docker data".into())]
            } else {
                vec![]
            }
        }
        "hex" => {
            let cache = home.join(".cache").join("hex");
            if cache.exists() {
                vec![(cache, "hex cache".into())]
            } else {
                vec![]
            }
        }
        "pub" => {
            let cache = home.join(".pub-cache");
            if cache.exists() {
                vec![(cache, "pub cache".into())]
            } else {
                vec![]
            }
        }
        "nuget" => {
            let cache = home.join(".nuget").join("packages");
            if cache.exists() {
                vec![(cache, "nuget packages".into())]
            } else {
                vec![]
            }
        }
        "vcpkg" => {
            let cache = PathBuf::from("/usr/local/share/vcpkg");
            if cache.exists() {
                vec![(cache, "vcpkg".into())]
            } else {
                vec![]
            }
        }
        "zypper" => {
            let cache = PathBuf::from("/var/cache/zypper");
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
            let cache = home.join(".cache").join("flatpak");
            if cache.exists() {
                vec![(cache, "flatpak cache".into())]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

fn scan_cache(tools: &[String]) -> Result<Vec<JunkFile>> {
    let mut junk = Vec::new();
    for tool in tools {
        for (dir, category) in get_cache_dirs(tool) {
            if dir.exists() {
                let size = dir_size(&dir);
                if size > 0 {
                    junk.push(JunkFile {
                        path: dir,
                        size,
                        category,
                    });
                }
            }
        }
    }
    junk.sort_by(|a, b| b.size.cmp(&a.size));
    Ok(junk)
}

fn clean_journalctl(dry_run: bool) -> Result<u64> {
    if dry_run {
        println!("  {} journalctl --vacuum-time=3d", "[dry-run]".yellow());
        return Ok(0);
    }
    let output = Command::new("journalctl")
        .args(["--vacuum-time=3d"])
        .output()?;
    if output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines() {
            println!("  {}", line.dimmed());
        }
        Ok(0)
    } else {
        println!(
            "  {} journalctl cleanup failed (need sudo?)",
            "failed".red()
        );
        Ok(0)
    }
}

fn clean_cache(junk: &[JunkFile], dry_run: bool) -> Result<()> {
    let mut cleaned = 0u64;
    for item in junk {
        if item.category == "journalctl logs" {
            cleaned += clean_journalctl(dry_run)?;
            continue;
        }
        if dry_run {
            println!("  {} {}", "[dry-run]".yellow(), item.path.display());
        } else {
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
        let display = if path_str.len() > 58 {
            format!("...{}", &path_str[path_str.len() - 55..])
        } else {
            path_str
        };
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

fn clean(junk: &[JunkFile], dry_run: bool) -> Result<()> {
    let mut cleaned = 0u64;
    for item in junk {
        if dry_run {
            println!("  {} {}", "[dry-run]".yellow(), item.path.display());
        } else {
            let res = if item.path.is_dir() {
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
                Some(t) => vec![t],
                None => vec![
                    "uv".into(),
                    "npm".into(),
                    "pnpm".into(),
                    "yarn".into(),
                    "bun".into(),
                    "deno".into(),
                    "cargo".into(),
                    "go".into(),
                    "pip".into(),
                    "poetry".into(),
                    "conda".into(),
                    "pdm".into(),
                    "gem".into(),
                    "composer".into(),
                    "maven".into(),
                    "gradle".into(),
                    "hex".into(),
                    "pub".into(),
                    "nuget".into(),
                    "journalctl".into(),
                    "apt".into(),
                    "snap".into(),
                    "brew".into(),
                    "mise".into(),
                    "pacman".into(),
                    "dnf".into(),
                    "zypper".into(),
                    "flatpak".into(),
                    "docker".into(),
                    "winget".into(),
                    "vcpkg".into(),
                ],
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
