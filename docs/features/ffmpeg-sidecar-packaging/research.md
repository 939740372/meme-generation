# 需求调研：FFmpeg Sidecar 打包与 libfreetype 文字渲染

- 状态：merged
- Feature slug：ffmpeg-sidecar-packaging
- 分支：feature/ffmpeg-sidecar-packaging
- 负责人：待定
- 日期：2026-10-22

## 1. 背景和问题

在上一阶段（`feature/ffmpeg-gif-mvp`）中，Meme Studio 完成了本地视频到 GIF 的转码管线、调色板优化、Rust 长任务进度管理及桌面基础工作台。然而，该版本目前仍有两个关键问题亟待解决：
1. **分发依赖环境**：当前应用强依赖宿主机已安装的 `ffmpeg` 和 `ffprobe`（从 PATH 中寻找），如果普通用户未安装 Homebrew 或 FFmpeg，应用将直接无法使用。必须通过 Tauri Sidecar 将 FFmpeg 打包进最终客户端。
2. **文字叠加不可用**：开发机默认通过 Homebrew 安装的精简版 FFmpeg 未编译 `libfreetype`，导致缺少 `drawtext` 滤镜，前端配置的梗图文字在导出时会被能力检测拦截（返回 `DRAW_TEXT_UNAVAILABLE`）。必须引入包含 `libfreetype`、`libharfbuzz` 等文本渲染引擎的完整静态 FFmpeg 二进制。

## 2. 用户和场景

- **小白用户**：下载 `.dmg` 或 `.exe` 安装包后开箱即用，无需配置复杂的系统环境或命令行工具。
- **梗图制作场景**：用户在视频画面中添加“经典台词”、“表情包文本”，导出能够清晰渲染中英文字符、自定义颜色及位置的最终 GIF。
- **CI / CD 打包**：自动化流水线能够根据目标平台拉取对应的静态 FFmpeg 二进制并无缝打包为安装包。

## 3. 现有方案调研

- **Tauri 2 External Bin (Sidecar)**：
  - Tauri 2 提供了 `bundle.externalBin` 机制，支持将平台特定命名的外部二进制打包至应用资源目录；
  - 命名格式为 `<binary-name>-<target-triple>`（如 `ffmpeg-aarch64-apple-darwin`、`ffmpeg-x86_64-pc-windows-msvc.exe`）；
  - 打包时 Tauri 会自动将对应平台的二进制拷贝到应用的 macOS `Contents/MacOS/` 或 Windows 安装包根目录中。
- **FFmpeg 跨平台静态构建**：
  - **macOS**：Martin Riedl（ffmpeg.martin-riedl.de）及 eugeneware/ffmpeg-static 提供了经过签名、包含 `libfreetype`、`libharfbuzz`、`libass`、`fontconfig` 的自包含静态独立二进制（支持 arm64 与 x86_64）。
  - **Windows / Linux**：`yt-dlp/FFmpeg-Builds` / `BtbN/FFmpeg-Builds` 提供了广泛使用的 GPL 静态构建，包含完整的 `libfreetype` 且无需动态依赖外部库。

## 4. 技术调研与约束

1. **二进制提交策略**：
   - 单个静态 FFmpeg/FFprobe 二进制通常在 30MB ~ 80MB，若将 4 个平台（macOS arm64, macOS x64, Win x64, Linux x64）的所有二进制直接 commit 到 Git 仓库，会导致仓库体积暴涨数倍，违反仓库规范；
   - 解决方案：在 `.gitignore` 中忽略 `src-tauri/binaries/*`（保留 `.gitkeep`），通过自动化下载脚本（如 Node/pnpm 脚本 `scripts/download-ffmpeg-sidecar.mjs`）在本地开发和 CI 构建前按需下载并校验。
2. **字体与 drawtext 特殊限制**：
   - 纯静态 FFmpeg 在不同操作系统上，若未通过 `fontfile` 明确指定具体字体文件，fontconfig 可能无法在 macOS 或 Windows 上正确定位默认字体；
   - 解决方案：在 Rust 侧构建 `drawtext` 参数时，增加系统字体自动探测（macOS: `/System/Library/Fonts/PingFang.ttc` 或 `/System/Library/Fonts/Helvetica.ttc`，Windows: `C:\Windows\Fonts\msyh.ttc`，Linux: `/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`），显式传递 `fontfile` 参数，保证 100% 稳定出字。
3. **许可证与合规**：
   - 引入的 FFmpeg 二进制为 GPL-3.0 / LGPL-3.0 许可证，必须在文档与分发说明中明确保留 FFmpeg 的版权与许可证声明。

## 5. 已确认事实

- Tauri 2 的 `bundle.externalBin` 支持配置外部二进制并在构建时嵌入。
- 静态构建的 FFmpeg（如 Martin Riedl 构建）通过了 macOS 静态链接验证并内置 `libfreetype`。
- Rust 层可以通过运行时环境、Tauri 资源目录、`src-tauri/binaries/` 及系统 PATH 建立 4 级查找链路，兼顾开发调试与生产分发。

## 6. 当前假设

- 本地开发环境先下载当前平台架构（`aarch64-apple-darwin`）的 FFmpeg & ffprobe 进行打包与真实文字渲染冒烟测试。
- 多平台二进制下载脚本可供 CI 流水线复用。

## 7. 未决问题

- 用户自定义字体选择功能在当前阶段暂未展开，本轮先支持系统默认字体与基础中英文渲染。

## 8. 调研结论和决策

| 结论/决策 | 理由 | 影响 |
| --- | --- | --- |
| 使用 `bundle.externalBin` 打包 | Tauri 2 官方推荐的 sidecar 分发方式 | 规范二进制目录与目标架构三元组命名 |
| 引入含 `libfreetype` 的静态二进制 | 解决 drawtext 不可用问题，实现文字渲染 | 补全 Meme 图核心文字叠加功能 |
| 二进制通过下载脚本按需拉取，不入 Git | 保护仓库轻量，遵守 AGENTS.md 约束 | 本地构建或 CI 前需运行准备脚本 |
| Rust 构建 drawtext 时自动指定系统字体路径 | 避免静态构建的 fontconfig 缺失导致找不到字体 | 跨平台稳定渲染文字 |

## 9. 参考资料和实验记录

- Tauri 2 External Bin 文档
- Martin Riedl FFmpeg Builds: `https://ffmpeg.martin-riedl.de`
- FFmpeg Filter drawtext 文档
