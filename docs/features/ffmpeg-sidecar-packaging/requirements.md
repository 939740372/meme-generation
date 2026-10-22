# 需求文档：FFmpeg Sidecar 打包与 libfreetype 文字渲染

- 状态：ready-to-merge
- Feature slug：ffmpeg-sidecar-packaging
- 分支：feature/ffmpeg-sidecar-packaging
- 负责人：待定
- 日期：2026-10-22

## 1. 目标

为 Meme Studio 配置标准 Tauri 2 Sidecar 打包体系，引入内置 `libfreetype` 支持的 FFmpeg / ffprobe 静态二进制，使桌面客户端不仅能摆脱宿主机 FFmpeg 依赖，还能完整支持中英文梗图文字渲染与叠加导出。

## 2. 范围

### 包含
- 在 `src-tauri/tauri.conf.json` 中配置 `bundle.externalBin`（`ffmpeg` 与 `ffprobe`）；
- 建立 `src-tauri/binaries/` 目录结构与 `.gitignore` 规则；
- 编写跨平台二进制下载脚本 `scripts/download-ffmpeg-sidecar.mjs`，支持自动下载当前平台或指定平台的静态二进制（包含 `libfreetype`）；
- 升级 Rust 侧可执行文件定位引擎：支持多级定位（环境变量优先、Tauri Sidecar 资源目录、本地 binaries 开发目录、系统 PATH 回退）；
- 升级 Rust 侧 `drawtext` 滤镜构建逻辑：针对静态二进制自动关联系统常见可用字体文件（如 macOS 苹方/Arial，Windows 微软雅黑等）；
- 验证带文字的端到端 GIF 渲染与导出冒烟；
- 验证 `pnpm tauri:build` 或本地构建产物中正确嵌入二进制。

### 不包含
- 用户在界面上手动上传自定义 `.ttf/.otf` 字体文件的文件选择器（后置到 P2 创作工作区）；
- 在 Git 仓库历史中直接提交数百兆的跨平台二进制大文件。

## 3. 用户故事

- 作为一名 Mac/Windows 普通用户，我打开打包好的应用后无需折腾终端或安装 Homebrew，就能直接导入视频制作 GIF。
- 作为梗图制作者，我在界面上输入“王境泽：真香”，导出的 GIF 画面正上方能清晰渲染出我输入的文字，字号、颜色和位置均符合预期。

## 4. 功能需求

| 编号 | 需求 | 优先级 | 验收方式 |
| --- | --- | --- | --- |
| FR-01 | 在 `tauri.conf.json` 中声明 `bundle.externalBin` | P0 | 配置文件包含 ffmpeg 与 ffprobe |
| FR-02 | 提供二进制下载与校验脚本 | P0 | 执行脚本能在 `src-tauri/binaries` 准备好对应 triple 的二进制 |
| FR-03 | Rust 端多级二进制定位策略 | P0 | 在无全局 FFmpeg 环境下依然能正常调用 Sidecar |
| FR-04 | 包含 `libfreetype` 且支持 `drawtext` 滤镜 | P0 | 过滤检测通过，不再返回 `DRAW_TEXT_UNAVAILABLE` |
| FR-05 | 自动适配平台中英文字体路径 | P0 | 文字能正常在画面上绘制，无乱码或 missing font 错误 |
| FR-06 | 端到端文字叠加真实渲染导出 | P0 | 输出的 GIF 经打开后能在画面中看到输入的文字 |

## 5. 非功能需求

- **轻量仓库**：二进制文件不进入 Git 跟踪，由下载脚本维护。
- **稳定性**：静态构建不依赖外部动态链接库，确保不同系统版本下不会发生 dynamic link 崩溃。
- **跨平台兼容**：遵循 Tauri 2 triple 规范（`aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`）。
- **许可证声明**：在项目文档中明确标注文档与静态构建的 GPL/LGPL 许可证说明。

## 6. 验收标准

- [x] `scripts/download-ffmpeg-sidecar.mjs` 能成功下载当前平台的静态 FFmpeg 与 ffprobe（含 `drawtext`）。
- [x] `tauri.conf.json` 包含 `bundle.externalBin` 配置。
- [x] Rust 端二进制定位支持 Sidecar，优先采用打包二进制。
- [x] 真实媒体测试中，配置文字（如中文与英文）可成功转码生成带文字叠加的 GIF，不再提示 `DRAW_TEXT_UNAVAILABLE`。
- [x] `pnpm build`、`cargo check` 与 `cargo test` 全部通过。
