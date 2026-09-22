# disk-cleaner 清理策略文档

## 目录

- [目录扫描策略](#目录扫描策略)
- [缓存清理策略](#缓存清理策略)
- [安全等级说明](#安全等级说明)
- [各工具详细策略](#各工具详细策略)
- [使用建议](#使用建议)

---

## 目录扫描策略

`disk-cleaner scan` 递归扫描指定目录，识别以下类型的垃圾文件：

### 识别规则

| 类别 | 匹配方式 | 目标 |
|------|----------|------|
| cache/build | 目录名精确匹配 | `node_modules`, `__pycache__`, `.pytest_cache`, `.mypy_cache`, `.gradle`；`target` 仅在父目录有 `Cargo.toml` 时匹配 |
| system | 文件名精确匹配 | `.DS_Store`, `Thumbs.db`, `desktop.ini` |
| temp/log | 使用 `--include-files` 时按扩展名匹配 | `*.tmp`, `*.temp`, `*.swp`, `*.swo`, `*.bak`, `*.log` |

默认不匹配通用 `.cache`、`.npm`、`.yarn`、`build` 或 `dist` 目录。扫描遇到无法完整读取的路径时会报告错误，`clean` 不会继续执行。

### 排序规则

结果按文件逻辑大小降序排列。该数字是内容大小估计值，不等于实际释放的磁盘空间。

---

## 缓存清理策略

`disk-cleaner cache` 扫描各包管理工具的缓存目录，支持两种模式：

| 模式 | 命令 | 行为 |
|------|------|------|
| 预览 | `cache --dry-run` | 仅展示可清理项，不执行删除 |
| 执行 | `cache` | 展示后需用户确认 `y/N`，确认后删除 |

### 默认扫描范围

不指定 `--tool` 时，检查以下 29 个工具：

```
uv, npm, pnpm, yarn, bun, deno, cargo, go, pip, poetry, conda, pdm,
gem, composer, maven, gradle, hex, pub, nuget, journalctl, apt, snap,
brew, mise, pacman, dnf, zypper, winget, vcpkg
```

### 清理方式

有原生清理命令的工具只使用该命令；命令不可用或执行失败时跳过该工具，并返回错误。仅无原生命令的工具使用目录删除：

- **原生命令**：工具自行管理缓存清理，最安全
- **目录删除**：直接删除缓存目录，下次使用时自动重建

### 路径安全校验

扫描和实际删除前都会校验路径。以下路径会被跳过：

- 根目录、用户主目录以及 `/etc`、`/usr`、`/var` 等系统关键目录
- 不存在、不是目录或任一路径组件为符号链接的路径
- 与工具预期目录形态不匹配的路径，例如把 WinGet `Packages` 当成缓存
- 层级过浅、可能代表磁盘或应用数据根目录的路径

实际删除前会再次校验，避免扫描后目录被替换。

---

## 安全等级说明

| 等级 | 说明 | 工具 |
|------|------|------|
| **安全** | 缓存可随时重建，删除后下次使用自动下载 | npm, pnpm, yarn, bun, cargo, go, pip, poetry, conda, pdm, gem, composer, maven, gradle, hex, pub, nuget, uv, brew, apt, dnf, zypper, pacman, snap, winget, vcpkg |
| **谨慎** | 删除后需重新下载工具或耗时较长 | mise, journalctl, deno |

Docker 和 Flatpak 不自动清理，需使用各自工具手动管理。

---

## 各工具详细策略

### Node.js 生态

| 工具 | 缓存路径 | 清理命令 | 说明 |
|------|----------|----------|------|
| npm | `~/.npm` | `npm cache clean --force` | 包下载缓存 |
| pnpm | `~/.local/share/pnpm/store` | `pnpm store prune` | 内容寻址存储，清理未引用包 |
| yarn | `~/.cache/yarn` | `yarn cache clean` | Yarn 1.x 缓存目录 |
| bun | `~/.bun/install/cache` | `bun pm cache rm` | Bun 包缓存 |
| deno | `~/.cache/deno` | `deno clean` | TypeScript/JS 编译缓存和远程模块缓存 |

### Python 生态

| 工具 | 缓存路径 | 清理命令 | 说明 |
|------|----------|----------|------|
| pip | `~/.cache/pip` | `pip cache purge` | wheel 和源码包缓存 |
| poetry | `~/.cache/pypoetry` | `poetry cache clear --all .` | Poetry 包缓存 |
| conda | `~/.conda/pkgs` | `conda clean --all -y` | Conda 包缓存，环境不受影响 |
| pdm | `~/.cache/pdm` | `pdm cache clear` | PDM 包缓存 |
| uv | `~/.cache/uv` | `uv cache clean` | uv 工具缓存 |

### 编译型语言

| 工具 | 缓存路径 | 清理命令 | 说明 |
|------|----------|----------|------|
| cargo | `$CARGO_HOME/{registry,git}` | 目录删除 | 未设置时回退到 `~/.cargo` |
| go | `$GOCACHE`, `$GOMODCACHE` | `go clean -cache -modcache` | 优先读取环境变量和 `go env` |
| maven | `$MAVEN_REPO_LOCAL` 或 `~/.m2/repository` | 目录删除 | Maven 本地依赖仓库 |
| gradle | `~/.gradle/caches` | 目录删除 | Gradle 构建缓存 |
| vcpkg | `/usr/local/share/vcpkg/{buildtrees,downloads,packages}` | 目录删除 | 仅清理缓存子目录，不删除安装本体 |

### 其他语言

| 工具 | 缓存路径 | 清理命令 | 说明 |
|------|----------|----------|------|
| gem | `$GEM_HOME/cache`（回退到 `gem env home`） | 目录删除 | Ruby 下载包缓存 |
| composer | `~/.cache/composer` | `composer clear-cache` | PHP Composer 包缓存 |
| hex | `$HEX_HOME/packages` | 目录删除 | 未设置时回退到 `~/.hex/packages` |
| pub | `$PUB_CACHE` | `dart pub cache clean` | 未设置时回退到 `~/.pub-cache` |
| nuget | `$NUGET_PACKAGES` | `dotnet nuget locals all --clear` | 未设置时回退到 `~/.nuget/packages` |

### 系统包管理器

| 工具 | 缓存路径 | 清理命令 | 说明 |
|------|----------|----------|------|
| apt | `/var/cache/apt/archives` | `apt-get clean` | deb 包缓存；不把 apt lists 计入可回收空间 |
| dnf | `/var/cache/dnf` | `dnf clean all` | RPM 包缓存 |
| pacman | `/var/cache/pacman/pkg` | 目录删除 | Arch Linux 包缓存 |
| zypper | `/var/cache/zypp` | `zypper clean` | openSUSE 包缓存 |
| snap | `/var/lib/snapd/cache` | 目录删除 | Snap 缓存 |
| flatpak | 不进行目录扫描 | 不自动清理 | 未使用运行时不是缓存目录，避免错误估算 |
| brew | `brew --cache` 返回的目录 | `brew cleanup --cache` | Homebrew 下载缓存 |
| winget | `%LOCALAPPDATA%/Temp/WinGet` | 目录删除 | WinGet 临时下载缓存；不删除便携包安装目录 |

### 运行时与容器

| 工具 | 缓存路径 | 清理命令 | 说明 |
|------|----------|----------|------|
| mise | `~/.cache/mise` | `mise cache clear` | 仅清理下载缓存，已安装工具不受影响 |
| docker | 不进行目录扫描 | 不自动清理 | 数据根目录含运行中数据，不能作为可回收空间估算 |

### 日志

| 工具 | 缓存路径 | 清理命令 | 说明 |
|------|----------|----------|------|
| journalctl | `/var/log/journal` | `journalctl --vacuum-time=3d` | 保留最近 3 天日志，需 sudo 权限 |

---

## 使用建议

### 日常维护

```bash
# 每周预览一次，了解缓存占用
disk-cleaner cache --dry-run

# 清理安全类缓存（可随时重建）
disk-cleaner cache --tool npm
disk-cleaner cache --tool pip
disk-cleaner cache --tool cargo
```

### 磁盘空间紧张时

```bash
# 1. 先看最大的缓存
disk-cleaner cache --dry-run

# 2. 清理占用最大的工具
disk-cleaner cache --tool pip
disk-cleaner cache --tool apt
disk-cleaner cache --tool npm

# 3. 清理日志（需 sudo）
sudo journalctl --vacuum-time=3d
```

### 项目目录清理

```bash
# 扫描项目目录中的 node_modules、target 等
disk-cleaner scan --path ~/projects

# 清理前预览
disk-cleaner clean --path ~/projects --dry-run

# 确认清理
disk-cleaner clean --path ~/projects
```

### Docker 清理

```bash
# 先查看 Docker 自己计算的空间，再使用原生命令清理
docker system df
docker image prune          # 清理悬空镜像
docker system prune         # 清理未使用的容器、网络、镜像
docker system prune --volumes  # 包括卷（谨慎）
```

### 注意事项

1. **首次运行建议使用 `--dry-run`**：确认清理范围后再执行
2. **cargo/pip 等缓存删除后**：首次编译/安装会重新下载，耗时较长
3. **mise 仅清理下载缓存**：已安装的工具版本不受影响
4. **Docker 不纳入自动目录清理**：使用 `docker system df/prune`，避免把数据根目录误算为可回收空间
5. **系统包管理器缓存**（apt/dnf/pacman）：删除后不影响已安装的软件

---

## License

MIT
