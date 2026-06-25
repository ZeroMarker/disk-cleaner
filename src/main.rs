use std::fs;
use std::path::{Path, PathBuf};

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
        "Path".bold(), "Size".bold(), "Category".bold()
    );
    println!("  {}", "-".repeat(90));

    for item in junk.iter().take(50) {
        let path_str = item.path.display().to_string();
        let display = if path_str.len() > 58 {
            format!("...{}", &path_str[path_str.len() - 55..])
        } else {
            path_str
        };
        println!("  {:<60} {:>10}  {}", display, format_size(item.size), item.category);
    }

    if junk.len() > 50 {
        println!(
            "\n  {} and {} more items...",
            "...".dimmed(),
            (junk.len() - 50).to_string().yellow()
        );
    }

    println!("\n  {} {}", "Total reclaimable:".bold(), format_size(total).red().bold());
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
    }

    Ok(())
}
