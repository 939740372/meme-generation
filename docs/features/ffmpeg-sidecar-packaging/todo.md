# 细化 TODO：FFmpeg Sidecar 打包与 libfreetype 文字渲染

- 状态：ready-to-merge
- Feature slug：ffmpeg-sidecar-packaging
- 分支：feature/ffmpeg-sidecar-packaging
- 负责人：待定
- 日期：2026-10-22

## 1. 配置与项目准备

- [x] 在 `.gitignore` 中增加 `src-tauri/binaries/*` 并排除 `.gitkeep`
- [x] 创建 `src-tauri/binaries/.gitkeep` 占位文件
- [x] 在 `src-tauri/tauri.conf.json` 中配置 `bundle.externalBin` 包含 `binaries/ffmpeg` 与 `binaries/ffprobe`
- [x] 在 `package.json` 中增加 `"prepare:sidecar"` 脚本配置

## 2. 跨平台静态二进制下载脚本

- [x] 编写 `scripts/download-ffmpeg-sidecar.mjs`，包含平台架构判断、下载、解压、文件重命名为 target-triple 规范及 chmod +x 处理
- [x] 针对当前平台（macOS arm64）执行下载脚本，获取包含 `libfreetype` 的静态二进制
- [x] 验证下载的 FFmpeg 包含 `drawtext` 滤镜（`ffmpeg -filters | grep drawtext`）

## 3. Rust 核心逻辑升级

- [x] 实现 `resolve_binary`：支持多级定位（环境变量 -> Tauri Sidecar 资源目录 -> `src-tauri/binaries/` -> 系统 PATH）
- [x] 更新 `media_probe` 和 `render_gif` 中的二进制调用
- [x] 实现系统默认字体路径检测（macOS / Windows / Linux）
- [x] 在 `build_drawtext_filter` 中支持字体路径自动注入与转义
- [x] 补充 Rust 单元测试

## 4. 自测与端到端验证

- [x] 执行 `git diff --check`
- [x] 执行 `pnpm build`
- [x] 执行 `cargo check --locked` 与 `cargo test`
- [x] 端到端文字渲染冒烟：使用真实样例视频，叠加中文字符，生成 GIF 并验证文字绘制结果
- [x] 将自测数据与结论记录到 `implementation.md`

## 5. 文档与收尾

- [x] 更新文档状态为 `ready-to-merge`
- [ ] 合并到 `main` 分支并将状态更新为 `merged`
- [ ] 同步更新根目录 `TODO.md`
