# 全帧画质精修与色彩滤镜任务清单 (TODO)

- **状态**: merged
- **分支**: `feature/frame-image-adjustments`

## 任务拆解

- [x] 调研 sidecar FFmpeg 可用的全帧图像增强滤镜（`eq`, `unsharp`, `gblur`, `hue`, `vignette`）并完成五件套文档
- [x] 在 `src-tauri/src/lib.rs` 中实现 `ImageAdjustments` 数据模型、参数校验、FFmpeg 滤镜链构建与单元测试
- [x] 在 `src/lib/desktop.ts` 与 `src/lib/templates.ts` 中扩展 `ImageAdjustments` 类型定义
- [x] 在 `src/App.vue` 与 `src/styles.css` 中实现「全帧画质与色彩精修」面板、一键精修预设、SVG 卷积锐化实时预览及草稿/历史持久化
- [x] 运行 `pnpm build`、`cargo check`、`cargo test` 与真实 FFmpeg GIF 导出抽帧验证
- [x] 更新文档状态为 `merged` 并合并推送至 `origin/main`
