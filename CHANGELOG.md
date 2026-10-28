# Changelog

## Unreleased

- 默认的 `cache` 全部工具扫描遇到普通用户无法读取的系统缓存（apt、snap、pacman、dnf、zypper）时列为未包含并继续，不再取消整次清理；显式 `--tool` 仍严格检查
- 修复 brew 清理命令：`brew cleanup --cache` 是无效选项，改为 `brew cleanup --prune=all`
- 修复 pub 清理会等待确认：改为 `dart pub cache clean --force`
- 原生命令不再继承标准输入，需要确认的命令直接失败而不是在隐藏提示处挂起
- `cache` 并行扫描各工具，并缓存重复的外部命令查询结果（本机 `cache --dry-run` 约 1.4s → 0.4s，输出不变）
- 大小统计按 inode 去重硬链接文件（pnpm/npm 存储），避免高估
- `journalctl` 清理会删除系统日志，现在默认的全部工具扫描会跳过它，须显式使用 `--tool journalctl`
- `cache` 在未设置 `HOME`/`USERPROFILE` 时报错，不再回退到当前目录
- pnpm 缓存位置遵循 `XDG_DATA_HOME`，bun 遵循 `BUN_INSTALL`
- 目录扫描不再匹配用户主目录下的 `~/.gradle`（含 wrapper 与配置），仅匹配项目内的 `.gradle`
- 确认提示在同一行等待输入；原生命令的 stdout 与 stderr 均会显示
- 目录清理和直接删除缓存改用已打开目录句柄；直接删除的缓存路径须匹配工具配置的完整位置
- 补充真实删除与路径变化测试、扫描基准脚本，并明确缓存大小估算的含义
- 缓存项显式记录工具身份；清理前复核全部缓存路径，未知工具和缺失身份的项目会被拒绝
- 缓存原生命令缺失或失败时不再回退到递归删除；dry-run 显示命令是否可用
- 默认目录扫描不再匹配通用 `.cache`、`.npm`、`.yarn` 和临时/备份/日志文件；后者可通过 `--include-files` 显式启用
- 扫描不完整或清理部分失败时返回错误；清理前展示全部目标路径，并将大小标为估计值
- Docker、Flatpak 改为明确提示手动清理；CI 覆盖 Linux、macOS 和 Windows
- 修复原生清理命令执行成功后匹配路径大小变化始终显示为 0 B
- 修复长中文路径在结果列表中截断时可能崩溃
- 扫描时报告无法访问的子目录，并取消不完整扫描后的清理
- 为未知的 `cache --tool` 参数返回明确错误

## v0.2.0

- 使用原生清理命令替代目录删除（22 个工具）
- 修复 mise 缓存清理误删已安装工具
- 修复 vcpkg 缓存清理误删安装目录
- 修复 docker 缓存清理误删容器数据
- 新增 GitHub Actions CI/Release 工作流

## v0.1.0

- 初始版本
- 将 31 个工具纳入初始缓存识别列表
- 支持目录扫描识别垃圾文件
- 支持 dry-run 模式
