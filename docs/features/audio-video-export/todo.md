# 细化开发任务清单：带音轨的短视频表情导出 (audio-video-export)

- 状态：implementing
- Feature slug: audio-video-export
- 分支：feature/audio-video-export
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 文档与规范准备
- [x] 探测本地 FFmpeg sidecar 对 `libx264`, `aac` 与 `atempo` / `areverse` 的支持
- [x] 编写 `research.md`、`requirements.md`、`design.md`、`implementation.md`、`todo.md`
- [ ] 提交初始五件套规范文档

## 2. Rust 后端实现 (`src-tauri/src/lib.rs`)
- [ ] `RenderPlan` 结构体新增 `include_audio: Option<bool>` 字段与 `is_mp4(&self) -> bool` 方法
- [ ] `validate_render_plan` 增加对 `.mp4` 扩展名校验
- [ ] `build_ffmpeg_args` 增加 MP4 分支：
  - [ ] 编码器指定 `-c:v libx264 -pix_fmt yuv420p -crf 23 -preset medium -movflags +faststart`
  - [ ] 当 `include_audio` 为 true（默认）：使用 `-map 0:v -map "0:a?" -c:a aac -b:a 128k`
  - [ ] 挂载匹配的音频滤镜（2x 加速 `atempo=2.0`、慢放 `atempo=0.5`、倒放 `areverse`）
  - [ ] 当 `include_audio` 为 false：使用 `-an`
- [ ] `read_file_as_data_url` 增加 `video/mp4` MIME 映射
- [ ] 编写原生单元测试：
  - [ ] `builds_mp4_args_with_audio_and_faststart`
  - [ ] `builds_mp4_args_muted_when_include_audio_false`
  - [ ] `builds_mp4_args_with_atempo_when_speed_filter`
- [ ] 运行并通过 `cargo test`

## 3. 前端模型与类型扩展 (`src/lib/*`)
- [ ] `src/lib/desktop.ts`: `ExportFormat` 扩展 `"mp4"`，`RenderPlan` 增加 `includeAudio?: boolean`
- [ ] `src/lib/templates.ts`: `StudioDraft` 支持 `includeAudio` 与 `"mp4"`
- [ ] `src/lib/queue.ts`: `QueueFormat` 与 `QueueItem` 支持 `"mp4"` 和 `includeAudio`，`generateQueueOutputPath` 映射 `.mp4`

## 4. 前端界面与样式适配 (`src/App.vue`, `src/styles.css`)
- [ ] `src/App.vue`:
  - [ ] `exportFormat` 加入 `"mp4"` 选项
  - [ ] 添加 `includeAudio = ref(true)` 状态，在导出设置面板中渲染开关
  - [ ] 导出按钮动态文案：`导出 MP4 ↗`
  - [ ] 渲染计划组装中传递 `includeAudio`
  - [ ] 历史记录支持 MP4 预览展示
- [ ] `src/styles.css`:
  - [ ] 增加 `.format-badge.mp4` 样式（科技橙红渐变徽章）
  - [ ] 增加 `.format-pill.pill-mp4` 胶囊高亮状态样式

## 5. 质量验证与门槛检查
- [ ] 编写端到端冒烟测试 `scripts/smoke-test-audio-video.mjs`
- [ ] 执行全套自动化验证：`cargo test`, `pnpm build`, `cargo check --locked`
- [ ] 执行端到端 smoke test 验证有声和无声输入场景
- [ ] 执行 `git diff --check`
- [ ] 更新文档状态为 ready-to-merge 并提交

## 6. 合并至主分支
- [ ] 切换到 `main` 分支执行非快进合并
- [ ] 更新 `TODO.md` 产品大方向
- [ ] 推送到远程仓库并清理本地分支
