# Changelog

## v0.2.0

- 使用原生清理命令替代目录删除（22 个工具）
- 修复 mise 缓存清理误删已安装工具
- 修复 vcpkg 缓存清理误删安装目录
- 修复 docker 缓存清理误删容器数据
- 新增 GitHub Actions CI/Release 工作流

## v0.1.0

- 初始版本
- 支持 31 个工具的缓存扫描和清理
- 支持目录扫描识别垃圾文件
- 支持 dry-run 模式
