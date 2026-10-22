# 实现与迁移计划：带音轨的短视频表情导出 (audio-video-export)

- 状态：merged
- Feature slug: audio-video-export
- 分支：feature/audio-video-export
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 文件变更清单

| 文件路径 | 变更类型 | 说明 |
|----------|----------|------|
| `docs/features/audio-video-export/*` | 新增 | 全套五件套设计开发文档 |
| `src-tauri/src/lib.rs` | 修改 | `RenderPlan` 增加 `include_audio` 与 `is_mp4()`；完善 `validate_render_plan` 兼容 `.mp4`；构建 MP4 参数流（libx264, aac, faststart, 0:a? 容错映射, 音频滤镜）；扩展 MIME 映射；添加单元测试 |
| `src/lib/desktop.ts` | 修改 | `ExportFormat` 扩展 `"mp4"`；`RenderPlan` 增加 `includeAudio?: boolean`；`pickMediaOutput` 支持 MP4 筛选 |
| `src/lib/templates.ts` | 修改 | `StudioDraft` 支持 `includeAudio` 与 `"mp4"` 格式 |
| `src/lib/queue.ts` | 修改 | `QueueFormat` 与 `QueueItem` 支持 `"mp4"` 和 `includeAudio`；`generateQueueOutputPath` 支持 `.mp4` |
| `src/App.vue` | 修改 | 格式栏增加 MP4 选项，增加 `includeAudio` 开关；按钮文案联动更新；历史记录渲染视频预览与视频徽章 |
| `src/styles.css` | 修改 | 增加 `.format-badge.mp4`、`.format-pill.pill-mp4`、`.history-format-badge.badge-mp4` 与 `.audio-switch-row` 样式 |
| `scripts/test-queue.mjs` | 修改 | 扩展队列对 MP4 命名防重与参数序列化断言 |
| `scripts/smoke-test-audio-video.mjs` | 新增 | 端到端冒烟测试（含音频、无音频、静音导出、2x 加速与倒放同步验证） |

## 2. 兼容性与降级策略

1. **旧草稿与旧历史数据**：
   - 缺少 `format` 或 `includeAudio` 字段的历史数据均自动向后兼容回退为默认值（format 默认 `"gif"`, includeAudio 默认 `true`）；
2. **输入视频无音轨情况**：
   - 使用 `-map "0:a?"` 避免 FFmpeg 抛出致命错误，优雅降级为无声 MP4。

## 3. 自测与验收记录

- [x] 单元测试：`PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml`
  - 19 个 Rust 单元测试全部通过（包含 `builds_mp4_filter_graph_and_args`, `builds_mp4_args_muted_when_include_audio_false`, `builds_mp4_args_with_audio_filters_when_speed_active`, `read_file_as_data_url_encodes_mp4_file` 等）。
- [x] 代码检查：`PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`（0 warning, 0 error）
- [x] 前端构建与类型检查：`pnpm build`（`vue-tsc --noEmit` + `vite build` 绿灯）
- [x] 队列扩展自测：`node scripts/test-queue.mjs`（通过）
- [x] 端到端冒烟测试：`node scripts/smoke-test-audio-video.mjs`
  - 成功测试包含 AAC 音频的正常 MP4 导出；
  - 成功测试 `-an` 静音 MP4 导出；
  - 成功测试输入源无音频流时的安全优雅降级；
  - 成功测试 2x 鬼畜加速声画同步（`setpts=0.5*PTS` + `atempo=2.0`）；
  - 成功测试倒放声画同步（`reverse` + `areverse`）。
- [x] 全套回归测试：`scripts/smoke-test-queue.mjs`, `smoke-test-avif.mjs`, `smoke-test-filters.mjs`, `test-templates.mjs`, `test-stickers.mjs` 全部通过。
- [x] 格式与规范：`git diff --check`（无异常）。
