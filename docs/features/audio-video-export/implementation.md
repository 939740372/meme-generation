# 实现与迁移计划：带音轨的短视频表情导出 (audio-video-export)

- 状态：implementing
- Feature slug: audio-video-export
- 分支：feature/audio-video-export
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 文件变更清单

| 文件路径 | 变更类型 | 说明 |
|----------|----------|------|
| `docs/features/audio-video-export/*` | 新增 | 全套五件套设计开发文档 |
| `src-tauri/src/lib.rs` | 修改 | `RenderPlan` 增加 `include_audio`；完善 `validate_render_plan` 兼容 `.mp4`；构建 MP4 参数流（libx264, aac, faststart, 容错映射, 音频滤镜）；扩展 MIME 映射；添加单元测试 |
| `src/lib/desktop.ts` | 修改 | `ExportFormat` 扩展 `"mp4"`；`RenderPlan` 增加 `includeAudio?: boolean` |
| `src/lib/templates.ts` | 修改 | `StudioDraft` 支持 `includeAudio` 与 `"mp4"` 格式 |
| `src/lib/queue.ts` | 修改 | `QueueFormat` 与 `QueueItem` 支持 `"mp4"` 和 `includeAudio`；`generateQueueOutputPath` 支持 `.mp4` |
| `src/App.vue` | 修改 | 格式栏增加 MP4 选项，增加 `includeAudio` 开关；按钮文案联动更新；历史记录渲染视频预览 |
| `src/styles.css` | 修改 | 增加 `.format-badge.mp4` 与 `.format-pill.pill-mp4` 样式 |
| `scripts/smoke-test-audio-video.mjs` | 新增 | 端到端冒烟测试（含音频与无音频双场景验证） |

## 2. 兼容性与降级策略

1. **旧草稿与旧历史数据**：
   - 缺少 `format` 或 `includeAudio` 字段的历史数据均自动向后兼容回退为默认值（format 默认 `"gif"`, includeAudio 默认 `true`）；
2. **输入视频无音轨情况**：
   - 使用 `-map "0:a?"` 避免 FFmpeg 抛出致命错误，优雅降级为无声 MP4。

## 3. 自测与验收记录

- [ ] 单元测试：`cargo test`
- [ ] 代码检查：`cargo check --manifest-path src-tauri/Cargo.toml --locked`
- [ ] 前端构建与类型：`pnpm build`
- [ ] 端到端冒烟测试：`node scripts/smoke-test-audio-video.mjs`
- [ ] 格式与规范：`git diff --check`
