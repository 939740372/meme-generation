# 详细实现文档：FFmpeg Sidecar 打包与 libfreetype 文字渲染

- 状态：merged
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
  - macOS: Martin Riedl Builds (`https://ffmpeg.martin-riedl.de`)，静态构建包含 `libfreetype`、`libharfbuzz`、`fontconfig`。
  - Windows/Linux: `yt-dlp/FFmpeg-Builds` / `BtbN/FFmpeg-Builds` GPL 全功能静态包。
- **许可证**：GPL 3.0 / LGPL 3.0。
- **分发注意**：在打包或发布应用时，保留对应的开源协议说明。

## 3. 测试方案

1. 验证下载脚本能否在 macOS arm64 环境下成功下载并命名为 `ffmpeg-aarch64-apple-darwin` 与 `ffprobe-aarch64-apple-darwin`，并赋予可执行权限；
2. 验证二进制是否包含 `drawtext` 滤镜；
3. 验证 Rust 单元测试和类型检查；
4. 端到端冒烟测试：使用带文字叠加的参数将样例视频导出为带有中文字幕的真实 GIF，并验证产物可播放、文字可见。

## 4. 自测记录

- 环境：macOS 15+ arm64；Node v22.22.0；pnpm 11.17.0；Rust 1.94.1；下载的静态构建 FFmpeg 9.0.2（内置 libfreetype、libharfbuzz、libass）。
- `scripts/download-ffmpeg-sidecar.mjs` 执行测试：
  - 下载目标：`aarch64-apple-darwin`
  - 成功下载并解压提取：`src-tauri/binaries/ffmpeg-aarch64-apple-darwin` 与 `ffprobe-aarch64-apple-darwin`。
  - 验证命令 `./src-tauri/binaries/ffmpeg-aarch64-apple-darwin -filters | grep drawtext` 返回包含 `drawtext`。
  - 二进制被 `.gitignore` 规则有效忽略，未泄露进 Git 跟踪。
- `git diff --check`：通过，无空白异常或格式问题。
- `pnpm build`（含 `vue-tsc --noEmit` & `vite build`）：通过。
- `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`：通过，零警告/错误。
- `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml`：通过，5 个单元测试全部通过（新增了 drawtext 滤镜构建、字体关联测试及二进制环境变量重写测试）。
- 真实媒体端到端冒烟（包含文字叠加）：通过。
  - 输入：640×360、30 FPS、2 秒 MP4 样例；
  - 滤镜组合：裁切 (480×270)、缩放 (320 宽)、帧率 (12 FPS)、自动关联系统字体（`/System/Library/Fonts/PingFang.ttc`）绘制文字“表情包测试 Hello Meme”，经由 `split + palettegen + paletteuse` 调色板管线；
  - 产物输出：大小 148,503 字节，时长 1.16 秒，尺寸 320×180，帧率 12 FPS，文字绘制清晰，画面完整可循环播放。
  - 测试完毕后清理临时资源。
