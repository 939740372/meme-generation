# 跨平台自动化构建、Windows 加固与远程更新检查调研 (Research)

- **状态**: merged
- **分支**: `feature/cross-platform-release-updater`
- **日期**: 2026-10-23

## 1. 背景与目标

用户希望在没有本地 Windows 实体机的情况下，通过 GitHub Actions 云端自动构建 Windows（`.exe` / `.msi`）与 macOS（`.dmg`）安装包，同时解决 Windows 下运行 FFmpeg 的专属兼容问题，并为客户端补齐基于 GitHub Releases 的远程版本检查与更新引导能力，从而完成 `TODO.md` 中 P3 阶段的最后一项「跨平台安装包和自动更新」。

## 2. 技术调研与关键发现

### 2.1 Windows 平台下 FFmpeg Sidecar 的两个典型问题
1. **控制台黑框闪现（Console Window Flashing）**：
   - 在 Windows GUI 子系统（`#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`）下，使用标准库 `std::process::Command` 启动控制台程序（如 `ffmpeg.exe`、`ffprobe.exe`、`powershell.exe`）时，Windows 默认会为每个子进程弹出一个黑色 CMD 窗口。在抽帧预览或探测媒体信息时会导致屏幕频繁闪现黑框。
   - **解决方案**：在 Windows 平台使用 `std::os::windows::process::CommandExt::creation_flags(0x08000000)`（即 `CREATE_NO_WINDOW`），统一封装 `new_hidden_command()` 构造函数。
2. **Windows 字体路径盘符冒号与反斜杠转义**：
   - FFmpeg 的 `drawtext` 滤镜在解析 `fontfile='C:\Windows\Fonts\msyh.ttc'` 时，`\` 会被滤镜图解析器二次消费，导致找不到字体。
   - **解决方案**：在 `detect_system_font()` 中统一返回正斜杠路径（如 `C:/Windows/Fonts/msyh.ttc`），配合 `escape_drawtext()` 将 `:` 转义为 `\:`，最终生成 `fontfile='C\:/Windows/Fonts/msyh.ttc'`，这是 FFmpeg 官方文档推荐的 Windows 字体路径写法。

### 2.2 GitHub Actions 跨平台打包策略
- 使用 `windows-latest`（Windows Server 2022 x64，预装 MSVC、WebView2、WiX、NSIS）与 `macos-latest`（Apple Silicon `aarch64-apple-darwin`）双矩阵并行构建。
- 支持两种触发方式：
  1. **手动一键触发 (`workflow_dispatch`)**：在 GitHub Actions 页面点击即可编译双端安装包并上传到 Artifacts，还可通过可选参数 `release_tag` 直接创建 GitHub Release。
  2. **标签推送触发 (`push: tags: ['v*']`)**：推送 `v0.1.1` 等版本标签时自动编译并发布到 GitHub Releases。

### 2.3 免密钥开箱即用的 GitHub Releases 远程更新检查
- 若强依赖 `tauri-plugin-updater` 的强制私钥签名，用户在未配置 `TAURI_SIGNING_PRIVATE_KEY` 仓库 Secret 时会导致 GitHub Actions 构建直接报错失败。
- 采用直接对接 GitHub Releases API（`https://api.github.com/repos/939740372/meme-generation/releases/latest`）的轻量更新检查器：
  - 无需预先配置复杂的签名密钥，GitHub Actions 第一次运行即可 100% 成功；
  - 支持语义化版本号对比（如 `0.1.0` vs `v0.2.0`）；
  - 自动根据当前运行系统（`windows` / `macos`）匹配 Release Assets 中的 `-setup.exe`、`.msi` 或 `.dmg` 下载链接，支持在应用内查看更新日志并调用系统浏览器/下载器一键获取最新安装包。
