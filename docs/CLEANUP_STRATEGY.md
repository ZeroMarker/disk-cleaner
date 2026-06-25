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
| cache/build | 目录名精确匹配 | `node_modules`, `__pycache__`, `.pytest_cache`, `.mypy_cache`, `target`, `.gradle`, `.cache`, `.npm`, `.yarn`, `dist`, `build` |
| system | 文件名精确匹配 | `.DS_Store`, `Thumbs.db`, `desktop.ini` |
| temp/log | 扩展名匹配 | `*.tmp`, `*.temp`, `*.swp`, `*.swo`, `*.bak`, `*.log` |

### 排序规则

结果按文件大小降序排列，优先展示占用空间最大的文件。

---

## 缓存清理策略

`disk-cleaner cache` 扫描各包管理工具的缓存目录，支持两种模式：

| 模式 | 命令 | 行为 |
|------|------|------|
| 预览 | `cache --dry-run` | 仅展示可清理项，不执行删除 |
| 执行 | `cache` | 展示后需用户确认 `y/N`，确认后删除 |

### 默认扫描范围

不指定 `--tool` 时，扫描全部 31 个工具：

```
uv, npm, pnpm, yarn, bun, deno, cargo, go, pip, poetry, conda, pdm,
gem, composer, maven, gradle, hex, pub, nuget, journalctl, apt, snap,
brew, mise, pacman, dnf, zypper, flatpak, docker, winget, vcpkg
```

---

## 安全等级说明

| 等级 | 说明 | 工具 |
|------|------|------|
| **安全** | 缓存可随时重建，删除后下次使用自动下载 | npm, pnpm, yarn, bun, deno, cargo, go, pip, poetry, conda, pdm, gem, composer, maven, gradle, hex, pub, nuget, uv, mise, brew, apt, dnf, zypper, pacman, flatpak, snap, winget, vcpkg |
| **谨慎** | 删除后需要重新配置或耗时较长 | docker, journalctl |

---

## 各工具详细策略

### Node.js 生态

| 工具 | 缓存路径 | 清理方式 | 说明 |
|------|----------|----------|------|
| npm | `~/.npm` | 删除目录 | 包下载缓存，下次 `npm install` 自动重新下载 |
| pnpm | `~/.local/share/pnpm/store` | 删除目录 | 内容寻址存储，硬链接源 |
| yarn | `~/.cache/yarn` | 删除目录 | Yarn 1.x 缓存目录 |
| bun | `~/.bun/install/cache` | 删除目录 | Bun 包缓存 |
| deno | `~/.cache/deno` | 删除目录 | TypeScript/JS 编译缓存和远程模块缓存 |

### Python 生态

| 工具 | 缓存路径 | 清理方式 | 说明 |
|------|----------|----------|------|
| pip | `~/.cache/pip` | 删除目录 | wheel 和源码包缓存 |
| poetry | `~/.cache/pypoetry` | 删除目录 | Poetry 包缓存 |
| conda | `~/.conda/pkgs` | 删除目录 | Conda 包缓存，环境不受影响 |
| pdm | `~/.cache/pdm` | 删除目录 | PDM 包缓存 |
| uv | `~/.cache/uv` | 删除目录 | uv 工具缓存 |

### 编译型语言

| 工具 | 缓存路径 | 清理方式 | 说明 |
|------|----------|----------|------|
| cargo | `~/.cargo/registry`, `~/.cargo/git` | 删除目录 | Crate 注册表和 git 依赖缓存 |
| go | `~/.cache/go-build`, `~/go/pkg/mod` | 删除目录 | 编译缓存和模块缓存 |
| maven | `~/.m2/repository` | 删除目录 | Maven 依赖仓库 |
| gradle | `~/.gradle/caches` | 删除目录 | Gradle 构建缓存 |
| vcpkg | `/usr/local/share/vcpkg` | 删除目录 | C/C++ 库缓存 |

### 其他语言

| 工具 | 缓存路径 | 清理方式 | 说明 |
|------|----------|----------|------|
| gem | `~/.gem/cache` | 删除目录 | Ruby gem 缓存 |
| composer | `~/.cache/composer` | 删除目录 | PHP Composer 包缓存 |
| hex | `~/.cache/hex` | 删除目录 | Elixir/Hex 包缓存 |
| pub | `~/.pub-cache` | 删除目录 | Dart/Flutter 包缓存 |
| nuget | `~/.nuget/packages` | 删除目录 | .NET NuGet 包缓存 |

### 系统包管理器

| 工具 | 缓存路径 | 清理方式 | 说明 |
|------|----------|----------|------|
| apt | `/var/cache/apt/archives`, `/var/lib/apt/lists` | 删除目录 | deb 包缓存和软件源列表 |
| dnf | `/var/cache/dnf` | 删除目录 | RPM 包缓存 |
| pacman | `/var/cache/pacman/pkg` | 删除目录 | Arch Linux 包缓存 |
| zypper | `/var/cache/zypper` | 删除目录 | openSUSE 包缓存 |
| snap | `/var/lib/snapd/cache` | 删除目录 | Snap 缓存 |
| flatpak | `~/.cache/flatpak` | 删除目录 | Flatpak 缓存 |
| brew | `~/.cache/Homebrew` 或 `/home/linuxbrew/.cache/Homebrew` | 删除目录 | Homebrew 下载缓存 |
| winget | `%LOCALAPPDATA%/Microsoft/WinGet/Packages` | 删除目录 | Windows 包缓存 |

### 运行时与容器

| 工具 | 缓存路径 | 清理方式 | 说明 |
|------|----------|----------|------|
| mise | `~/.local/share/mise`, `~/.cache/mise` | 删除目录 | 运行时版本管理器数据和缓存 |
| docker | `/var/lib/docker` | **需手动** | 建议使用 `docker system prune` 命令 |

### 日志

| 工具 | 缓存路径 | 清理方式 | 说明 |
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
disk-cleaner cache --tool mise      # 通常最大
disk-cleaner cache --tool pip
disk-cleaner cache --tool apt

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

### Docker 清理（建议使用原生命令）

```bash
# 清理悬空镜像
docker image prune

# 清理未使用的容器、网络、镜像
docker system prune

# 包括卷（谨慎）
docker system prune --volumes
```

### 注意事项

1. **首次运行建议使用 `--dry-run`**：确认清理范围后再执行
2. **cargo/pip 等缓存删除后**：首次编译/安装会重新下载，耗时较长
3. **mise 数据删除后**：需要重新安装运行时版本
4. **docker 目录不建议直接删除**：使用 `docker system prune` 更安全
5. **系统包管理器缓存**（apt/dnf/pacman）：删除后无法回滚已安装包，但不影响已安装的软件

---

## License

MIT
