# 文字与贴纸叠加渲染修复任务清单 (TODO)

- **状态**: merged
- **分支**: `fix/text-sticker-gif-render`

## 任务拆解

- [x] 定位文字和贴纸未正确渲染进 GIF 的前后端根因（macOS 中文字体缺失、源视频坐标与输出分辨率未做缩放映射、面板开关未自动激活）
- [x] 创建 `docs/features/text-sticker-gif-render/` 五件套文档
- [x] 更新 `src-tauri/src/lib.rs`：修复 `detect_system_font()` 中文字体优先级、支持 `source_width`/`source_height` 坐标与尺寸等比映射、规范化贴纸目录路径并补充单元测试
- [x] 更新 `src/lib/desktop.ts` 与 `src/App.vue`：传递源视频尺寸、按源视频比例自适应初始化文字与贴纸尺寸、优化文字/贴纸自动启用与内嵌贴纸点选体验
- [x] 执行 `pnpm build`、`cargo check`、`cargo test` 与真实 FFmpeg GIF 导出抽帧验证
- [x] 更新文档状态为 `merged` 并合并推送至 `origin/main`
