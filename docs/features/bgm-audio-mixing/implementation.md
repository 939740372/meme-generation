# 实现方案：自定义背景音乐与音效混流 (bgm-audio-mixing)

- 状态：ready-to-merge
- Feature slug: bgm-audio-mixing
- 分支：feature/bgm-audio-mixing
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 文件级实现计划

| 文件 | 变更类型 | 说明 |
|------|----------|------|
| `scripts/generate-sfx.mjs` | 新增 | 基于 FFmpeg 合成 8 款经典二创音效（MP3 格式）到 `src-tauri/sfx/` |
| `src-tauri/sfx/*.mp3` | 新增 | 8 款内置音效资源文件（消音哔哔、叮、低音炸裂、警报、弹簧、欢呼、打脸、笑声） |
| `src-tauri/src/lib.rs` | 修改 | 新增 `BgmOverlay`, `BuiltinSfx`，扩展 `MediaInfo` 与 `RenderPlan`，实现 `resolve_sfx_dir`, `get_builtin_sfx`, `has_audio_stream`，并在 `build_ffmpeg_args_with_hw` 中组装 `amix` 混流管线；新增单元测试 |
| `src-tauri/tauri.conf.json` | 修改 | 将 `sfx/` 纳入打包 bundle resources |
| `src/lib/desktop.ts` | 修改 | 导出 `BgmOverlay`, `BuiltinSfx`, `getBuiltinSfx`, `pickAudioFile` |
| `src/lib/templates.ts` | 修改 | 模板与草稿结构兼容 `bgmOverlay` 与 `hasAudio` 字段 |
| `src/lib/queue.ts` | 修改 | 批量导出队列任务兼容 `bgmOverlay` 与 `hasAudio` 字段 |
| `src/App.vue` | 修改 | 抽屉新增“🎵 音效库”Tab、控制面板新增音频音量与素材选择、试听播放器逻辑 |
| `src/styles.css` | 修改 | 音效素材卡片、试听按钮、音量双滑块样式 |
| `scripts/smoke-test-bgm-mixing.mjs` | 新增 | 端到端自动化冒烟测试（含双路混音、无声视频加 BGM、变速混音与 ffprobe 验证） |
| `TODO.md` | 修改 | 记录特性完成进度与文档关联 |

## 2. 兼容性与迁移策略

- **向下兼容**：`bgm_overlay` 与 `has_audio` 均为可选字段（`Option<...>`），旧版草稿、历史记录和现有模板反序列化时自动保持默认值，无破坏性变更。
- **动图输出隔离**：对于 GIF / WebP / AVIF 等动图格式，自动保持 `-an`，不受任何音频混流逻辑影响。
- **跨平台资源定位**：`resolve_sfx_dir` 覆盖开发环境相对路径与 macOS / Windows 打包后的 Resources 目录，保持自包含体验。

## 3. 测试方案

1. **Rust 单元测试**：
   - `builds_mp4_args_with_bgm_mixing`：验证有原声 + 有 BGM 时的 `amix` 混流参数及 input 索引；
   - `builds_mp4_args_with_bgm_when_orig_muted_or_no_audio`：验证无原声 + 有 BGM 时不使用 `0:a` 且正确映射 `[aout]`；
   - `builds_mp4_args_with_bgm_and_sticker_overlay`：验证贴纸与 BGM 并存时 input 索引自增到 2；
   - `resolves_sfx_dir_and_loads_builtin_sfx`：验证 8 款内置音效目录解析与 base64 编码完备。
2. **端到端冒烟测试 (`smoke-test-bgm-mixing.mjs`)**：
   - 测试场景 1：生成带音频源片 + 选用 BGM -> 导出 MP4，验证时长与双音轨混音；
   - 测试场景 2：生成无音频源片 + 选用 BGM -> 导出 MP4，验证音轨成功创建且视频正常播放；
   - 测试场景 3：开启 2x 鬼畜加速 + 选用 BGM -> 导出 MP4，验证原声变速且 BGM 正常；
   - 测试场景 4：导出 GIF 格式验证自动 `-an` 静音。
3. **质量门槛与构建检查**：
   - `pnpm build`（`vue-tsc` + `vite`）
   - `cargo test`
   - `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`
   - `git diff --check`

## 4. 自测记录

- 状态：全部通过 (PASS)
- 测试平台：macOS 15.3 (Apple Silicon aarch64)
- 执行记录：
  1. **内置音效生成与资源完备性**：
     - 执行：`node scripts/generate-sfx.mjs`
     - 结果：生成 8 款高品质免版权音效（消音哔哔、绝了清脆叮、震撼低音808、高能警报、弹簧搞笑、欢呼和弦、打脸啪啪、滑稽笑声），总大小仅约 160KB，通过 `src-tauri/tauri.conf.json` 配置打包入 `resources`。
  2. **Rust 单元测试**：
     - 命令：`PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml`
     - 结果：27 项测试全部通过（含 BGM 双路混音、无声视频单轨 BGM 注入、贴纸与 BGM 复合三路输入索引管理、8 款内置音效读取与 base64 验证、音频 MIME 识别等）。
  3. **Cargo locked 严格检查**：
     - 命令：`PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`
     - 结果：0 errors, 0 warnings 通过。
  4. **端到端冒烟测试**：
     - 命令：`node scripts/smoke-test-bgm-mixing.mjs`
     - 结果：全部 6 项测试场景均绿灯通过：
       - 内置音效时长与格式探测通过；
       - 双路混流（原声 + 消音哔哔）导出 MP4，验证 1 视频流 + 1 AAC 音频流，时长严格 2.000s 对齐；
       - 无原声音频的纯静音视频 + 震撼低音，成功合成出包含 AAC 音轨的 MP4，零报错；
       - 2x 鬼畜变速视频 + 清脆叮，原声 2x atempo 变速，时长对齐 1.05s；
       - GIF 动图模式自动 `-an` 隔离验证通过。
  5. **前端 TypeScript 与构建检查**：
     - 命令：`pnpm build` (`vue-tsc --noEmit && vite build`)
     - 结果：通过，产物正常构建无警告。
  6. **Git 代码差异格式检查**：
     - 命令：`git diff --check`
     - 结果：无空白或格式异常。
