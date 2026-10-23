# 跨平台自动化构建、Windows 加固与远程更新检查需求说明 (Requirements)

- **状态**: merged
- **分支**: `feature/cross-platform-release-updater`
- **日期**: 2026-10-23

## 1. 需求范围

1. **Windows 平台运行时加固**：
   - 所有外部子进程调用（`ffmpeg`、`ffprobe`、`powershell`）在 Windows 下必须附加 `CREATE_NO_WINDOW (0x08000000)` 标志，杜绝黑框闪现。
   - Windows 系统字体候选列表扩展支持微软雅黑（`msyh.ttc` / `msyhbd.ttc`）、黑体（`simhei.ttf`）、宋体（`simsun.ttc`）及 Arial，并统一转换为正斜杠路径格式以兼容 FFmpeg `drawtext` 滤镜。
   - Sidecar 下载脚本 `scripts/download-ffmpeg-sidecar.mjs` 支持在 Windows / Linux / macOS 全平台校验 `ffmpeg -version` 与 `drawtext` 滤镜可用性。
2. **GitHub Actions 跨平台自动构建与发布**：
   - 新增 `.github/workflows/build-release.yml`。
   - 支持 `workflow_dispatch`（网页手动触发，可选填发布版本号）和 `v*` 标签推送触发。
   - 自动构建 Windows NSIS (`-setup.exe`)、Windows MSI (`.msi`) 和 macOS (`.dmg`) 安装包，上传为 Actions Artifacts 并在指定标签时自动发布到 GitHub Releases。
3. **客户端远程更新检查与下载引导**：
   - 应用启动后静默检查 GitHub Releases 最新版本，若有新版本则在顶栏显示醒目的新版本提示徽章。
   - 用户也可随时点击顶栏版本胶囊手动检查更新，弹出更新详情弹窗（展示当前版本、最新版本、发布说明及一键下载对应系统的安装包按钮）。

## 2. 验收标准

- [ ] `src-tauri/src/lib.rs` 中所有 `Command::new` 均通过统一的无窗口构造函数创建，并包含语义化版本比对单测与 Windows 字体路径转义单测。
- [ ] 顶栏显示当前版本号与更新状态，点击可触发版本检查并展示更新弹窗。
- [ ] `.github/workflows/build-release.yml` 语法规范完整，覆盖 Windows 与 macOS 双端构建、Sidecar 准备、产物上传与自动 Release 发布。
- [ ] `pnpm build`、`cargo check --locked`、`cargo test` 全部通过。
