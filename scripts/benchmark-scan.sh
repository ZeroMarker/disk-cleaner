#!/usr/bin/env bash
set -euo pipefail

count=${1:-10000}
if [[ ! $count =~ ^[1-9][0-9]*$ ]]; then
  echo "usage: $0 [positive-file-count]" >&2
  exit 2
fi

repo_dir=$(cd "$(dirname "$0")/.." && pwd)
cargo build --release --manifest-path "$repo_dir/Cargo.toml" >/dev/null

fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/ordinary" "$fixture/project/node_modules"

for ((index = 0; index < count; index++)); do
  : > "$fixture/ordinary/file-$index.txt"
done
for ((index = 0; index < count / 10; index++)); do
  : > "$fixture/project/node_modules/file-$index.bin"
done

echo "Scanning $count ordinary files and $((count / 10)) matched directory files"
/usr/bin/time -p "$repo_dir/target/release/disk-cleaner" scan --path "$fixture" >/dev/null
