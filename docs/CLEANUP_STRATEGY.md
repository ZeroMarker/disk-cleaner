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
| cache/build | 目录名精确匹配 | `node_modules`, `__pycache__`, `.pytest_cache`, `.mypy_cache`, `target`, `.gradle`, `.cache`, `.npm`, `.yarn` |
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

### 清理方式

优先使用各工具的原生清理命令，无原生命令时回退到目录删除：

- **原生命令**：工具自行管理缓存清理，最安全
- **目录删除**：直接删除缓存目录，下次使用时自动重建

---

## 安全等级说明

| 等级 | 说明 | 工具 |
|------|------|------|
| **安全** | 缓存可随时重建，删除后下次使用自动下载 | npm, pnpm, yarn, bun, cargo, go, pip, poetry, conda, pdm, gem, composer, maven, gradle, hex, pub, nuget, uv, brew, apt, dnf, zypper, pacman, flatpak, snap, winget, vcpkg |
| **谨慎** | 删除后需重新下载工具或耗时较长 | mise, docker, journalctl, deno |

---

## 各工具详细策略

### Node.js 生态

| 工具 | 缓存路径 | 清理命令 | 说明 |
|------|----------|----------|------|
| npm | `~/.npm` | `npm cache clean --force` | 包下载缓存 |
| pnpm | `~/.local/share/pnpm/store` | `pnpm store prune` | 内容寻址存储，清理未引用包 |
| yarn | `~/.cache/yarn` | `yarn cache clean` | Yarn 1.x 缓存目录 |
| bun | `~/.bun/install/cache` | `bun pm cache rm` | Bun 包缓存 |
| deno | `~/.cache/deno` | 目录删除 | TypeScript/JS 编译缓存和远程模块缓存 |

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
| cargo | `~/.cargo/registry`, `~/.cargo/git` | 目录删除 | Crate 注册表和 git 依赖缓存 |
| go | `~/.cache/go-build`, `~/go/pkg/mod` | `go clean -cache -modcache` | 编译缓存和模块缓存 |
| maven | `~/.m2/repository` | `mvn dependency:purge-local-repository` | Maven 依赖仓库 |
| gradle | `~/.gradle/caches` | 目录删除 | Gradle 构建缓存 |
| vcpkg | `/usr/local/share/vcpkg/{buildtrees,downloads,packages}` | 目录删除 | 仅清理缓存子目录，不删除安装本体 |

### 其他语言

| 工具 | 缓存路径 | 清理命令 | 说明 |
|------|----------|----------|------|
| gem | `~/.gem/cache` | `gem cleanup` | Ruby gem 缓存 |
| composer | `~/.cache/composer` | `composer clear-cache` | PHP Composer 包缓存 |
| hex | `~/.cache/hex` | 目录删除 | Elixir/Hex 包缓存 |
| pub | `~/.pub-cache` | `dart pub cache clean` | Dart/Flutter 包缓存 |
| nuget | `~/.nuget/packages` | `dotnet nuget locals all --clear` | .NET NuGet 包缓存 |

### 系统包管理器

| 工具 | 缓存路径 | 清理命令 | 说明 |
|------|----------|----------|------|
| apt | `/var/cache/apt/archives` | `apt-get clean` | deb 包缓存 |
| dnf | `/var/cache/dnf` | `dnf clean all` | RPM 包缓存 |
| pacman | `/var/cache/pacman/pkg` | 目录删除 | Arch Linux 包缓存 |
| zypper | `/var/cache/zypper` | `zypper clean` | openSUSE 包缓存 |
| snap | `/var/lib/snapd/cache` | 目录删除 | Snap 缓存 |
| flatpak | `~/.cache/flatpak` | `flatpak uninstall --unused` | Flatpak 未使用运行时 |
| brew | `~/.cache/Homebrew` 或 `/home/linuxbrew/.cache/Homebrew` | `brew cleanup --cache` | Homebrew 下载缓存 |
| winget | `%LOCALAPPDATA%/Microsoft/WinGet/Packages` | 目录删除 | Windows 包缓存 |

### 运行时与容器

| 工具 | 缓存路径 | 清理命令 | 说明 |
|------|----------|----------|------|
| mise | `~/.cache/mise` | `mise cache clear` | 仅清理下载缓存，已安装工具不受影响 |
| docker | `/var/lib/docker` | `docker system prune -f` | 清理悬空镜像、停止的容器、未使用的网络 |

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
# 使用 disk-cleaner 内置清理（等同 docker system prune -f）
disk-cleaner cache --tool docker

# 或手动使用更精细的命令
docker image prune          # 清理悬空镜像
docker system prune         # 清理未使用的容器、网络、镜像
docker system prune --volumes  # 包括卷（谨慎）
```

### 注意事项

1. **首次运行建议使用 `--dry-run`**：确认清理范围后再执行
2. **cargo/pip 等缓存删除后**：首次编译/安装会重新下载，耗时较长
3. **mise 仅清理下载缓存**：已安装的工具版本不受影响
4. **docker 使用 `system prune`**：安全清理悬空资源，不影响运行中的容器
5. **系统包管理器缓存**（apt/dnf/pacman）：删除后不影响已安装的软件

---

## License

MIT
