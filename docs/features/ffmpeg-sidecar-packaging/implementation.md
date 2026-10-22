# 详细实现文档：FFmpeg Sidecar 打包与 libfreetype 文字渲染

- 状态：implementing
- Feature slug：ffmpeg-sidecar-packaging
- 分支：feature/ffmpeg-sidecar-packaging
- 负责人：待定
- 日期：2026-10-22

## 1. 文件和模块计划

| 路径 | 变更 | 目的 |
| --- | --- | --- |
| `src-tauri/tauri.conf.json` | 增加 `bundle.externalBin` 配置 | 让 Tauri 2 支持打包 sidecar 二进制 |
| `.gitignore` | 增加 `src-tauri/binaries/*` 忽略项（除 `.gitkeep`） | 防止几十兆到几百兆二进制大文件被误提交入 Git |
| `src-tauri/binaries/.gitkeep` | 新建占位文件 | 保证二进制目录结构存在 |
| `scripts/download-ffmpeg-sidecar.mjs` | 新增自动化下载脚本 | 支持拉取指定架构或当前平台的自包含静态 FFmpeg（含 libfreetype） |
| `package.json` | 增加 `"prepare:sidecar"` 脚本命令 | 方便开发者一键拉取依赖二进制 |
| `src-tauri/src/lib.rs` | 重构二进制解析与 drawtext 参数构建 | 增加多级 sidecar 定位与系统中英文字体绑定 |
| `docs/features/ffmpeg-sidecar-packaging/` | 维护全套文档 | 记录设计、实现与自测 |

## 2. 静态二进制与第三方库信息

- **来源**：
  - macOS: Martin Riedl Builds (`https://ffmpeg.martin-riedl.de`) / eugeneware-static，包含 `libfreetype`、`libharfbuzz`、`fontconfig`。
  - Windows/Linux: `yt-dlp/FFmpeg-Builds` / `BtbN/FFmpeg-Builds` GPL 全功能静态包。
- **许可证**：GPL 3.0 / LGPL 3.0。
- **分发注意**：在打包或发布应用时，保留对应的开源协议说明。

## 3. 测试方案

1. 验证下载脚本能否在 macOS arm64 环境下成功下载并命名为 `ffmpeg-aarch64-apple-darwin` 与 `ffprobe-aarch64-apple-darwin`，并赋予可执行权限；
2. 验证二进制是否包含 `drawtext` 滤镜；
3. 验证 Rust 单元测试和类型检查；
4. 端到端冒烟测试：使用带文字叠加的参数将样例视频导出为带有中文字幕的真实 GIF，并验证产物可播放、文字可见。

## 4. 自测记录

*(待实现后更新)*
