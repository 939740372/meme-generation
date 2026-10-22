# 需求说明书：macOS DMG 桌面安装包构建与分发验收

- 状态：proposed
- Feature slug: desktop-packaging
- 分支：feature/desktop-packaging
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 特性概述

本特性完成 Meme Studio 面向终端创作者的桌面客户端独立安装包构建与分发验收。通过配置与构建管道生成独立的 macOS DMG 镜像文件与 `.app` 应用程序包，并将静态编译的 FFmpeg 与 FFprobe 完整嵌入其中，实现开箱即用的离线媒体转码能力。

## 2. 用户故事

- **US-1 免环境安装**：作为创作者，从 GitHub Release 或官网下载 DMG 后，我只需将“表情工坊”拖入 Applications 即可完成安装，电脑上无需安装 Node.js、Homebrew 或命令行工具。
- **US-2 独立媒体处理**：作为创作者，在脱离任何开发环境独立双击启动应用后，能够正常导入视频、探测元信息、添加文字描边与字幕框、并成功导出 GIF 与 WebP 动图。
- **US-3 规范的打包产物**：作为开发者/维护者，通过标准脚本能够稳定构建出符合 macOS Bundle 规范的 Release 安装包，且 sidecar 文件权限可执行。

## 3. 功能需求清单

### 3.1 构建配置与依赖
- **FR-1 Bundle 配置规范化**：
  - 确认 `src-tauri/tauri.conf.json` 中 `externalBin` 声明 `binaries/ffmpeg` 与 `binaries/ffprobe`；
  - 针对 macOS 设定最低支持版本（如 `10.15` 或 `11.0`）；
  - 图标与应用元信息完整配置。
- **FR-2 二进制打包嵌入**：
  - 构建产物 `.app` 内部包含可执行权限（`+x`）的 `ffmpeg` 与 `ffprobe` sidecar 二进制。

### 3.2 运行时寻址增强
- **FR-3 生产环境多路径容错寻址**：
  - `resolve_binary` 支持在 `Contents/MacOS/` 同级与 `Contents/Resources/` 下自动适配带 target-triple 或去重后缀的二进制名称；
  - 找不到时抛出友好的标准错误，绝不在终端崩溃退出。

### 3.3 构建与交付产物
- **FR-4 一键构建脚本**：
  - `pnpm tauri:build` 执行前端优化编译（`pnpm build`）、Rust Release 编译以及 DMG 打包打包流程；
  - 产出标准 DMG 安装包：`表情工坊_<version>_<arch>.dmg`。

## 4. 非功能需求

- **NFR-1 独立性**：导出的 `.dmg` 安装包完全自包含，无任何未声明的外部动态库缺失；
- **NFR-2 体积控制**：记录并评估最终 DMG 安装包体积；
- **NFR-3 启动性能**：脱离开发环境独立启动后，窗口首屏展示时间小于 1 秒。

## 5. 验收标准 (Acceptance Criteria)

1. **AC-1**：运行打包命令成功生成 Release 产物 `.dmg` 与 `.app`，退出码为 0。
2. **AC-2**：检查生成的 `表情工坊.app` 目录结构，确认内置的 `ffmpeg` 与 `ffprobe` 存在且具备可执行权限（`chmod +x`）。
3. **AC-3**：在完全不依赖开发环境环境变量的条件下，通过命令行或原生方式对 Bundle 内部的主可执行文件与 sidecar 运行 smoke 校验，成功输出正确的版本与媒体信息。
4. **AC-4**：全量代码通过 `pnpm build`、`cargo check`、`cargo test` 与 `git diff --check`。
