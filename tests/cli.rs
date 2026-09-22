use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "disk-cleaner-cli-test-{}-{}",
            std::process::id(),
            NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn clean_preview_shows_every_full_path_before_cleanup() {
    let root = TestDir::new();
    let long_name = format!("{}.log", "long-name-".repeat(8));
    for index in 0..50 {
        fs::write(root.0.join(format!("item-{index:02}.log")), b"junk").unwrap();
    }
    fs::write(root.0.join(&long_name), b"junk").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_disk-cleaner"))
        .args(["clean", "--path", root.0.to_str().unwrap(), "--dry-run"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let preview = stdout.split("  [dry-run]").next().unwrap();
    assert_eq!(preview.matches(".log").count(), 51);
    assert!(preview.contains(&root.0.join(long_name).display().to_string()));
}

#[test]
fn scan_ignores_generic_build_and_dist_directories() {
    let root = TestDir::new();
    for name in ["build", "dist", "target"] {
        fs::create_dir(root.0.join(name)).unwrap();
        fs::write(root.0.join(name).join("data.bin"), b"keep").unwrap();
    }

    let output = Command::new(env!("CARGO_BIN_EXE_disk-cleaner"))
        .args(["scan", "--path", root.0.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("No junk files found.")
    );
}

#[cfg(unix)]
#[test]
fn failed_native_command_keeps_cache_directory() {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let root = TestDir::new();
    let bin = root.0.join("bin");
    let cache = root.0.join("cache");
    fs::create_dir(&bin).unwrap();
    fs::create_dir(&cache).unwrap();
    fs::write(cache.join("keep.bin"), b"keep").unwrap();

    let npm = bin.join("npm");
    fs::write(&npm, "#!/bin/sh\nexit 9\n").unwrap();
    fs::set_permissions(&npm, fs::Permissions::from_mode(0o755)).unwrap();

    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());
    let mut child = Command::new(env!("CARGO_BIN_EXE_disk-cleaner"))
        .args(["cache", "--tool", "npm"])
        .env("NPM_CONFIG_CACHE", &cache)
        .env("PATH", path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"y\n").unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(!output.status.success());
    assert!(cache.join("keep.bin").exists());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("cache after command failure")
    );
}
