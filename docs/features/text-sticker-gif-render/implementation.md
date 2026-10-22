# 文字与贴纸叠加渲染修复实现与测试文档 (Implementation)

- **状态**: merged
- **分支**: `fix/text-sticker-gif-render`
- **日期**: 2026-10-22

## 1. 修改文件清单

- `src-tauri/src/lib.rs`：
  - 更新 `detect_system_font()`，在 macOS 和 Linux 上优先选用包含完整 CJK 汉字字形的系统字体（macOS 优先 `/System/Library/Fonts/STHeiti Medium.ttc`、`/System/Library/Fonts/Hiragino Sans GB.ttc`、`/System/Library/Fonts/STHeiti Light.ttc`）。
  - 在 `RenderPlan` 中新增 `source_width` 与 `source_height` 字段，并在 `render_gif` 中使用 `probe_media` 结果自动补齐。
  - 在 `validate_render_plan()` 中将最大有效字号上限提升至 `512`，贴纸最大宽度上限提升至 `3840`，以支持 1080p/4K 源视频空间下的字幕与贴纸。
  - 在 `build_filter_graph()` 中引入 `scale_text_overlay()` 与 `scale_image_overlay()`，将源视频坐标系下的文字图层与贴纸图层精确映射到裁切及缩放后的输出帧坐标系；同时修正 `slow05x` 慢放滤镜下高级文字透明画布的持续时长。
  - 在 `resolve_stickers_dir()` 和 `resolve_sfx_dir()` 中对解析出的目录执行 `fs::canonicalize` 规范化。
  - 新增 3 项单元测试验证源分辨率缩放映射、裁切偏移扣除及中文字体探测。
- `src/lib/desktop.ts` & `src/lib/templates.ts`：
  - 在 `RenderPlan` 与 `StudioDraft` 接口中新增可选字段 `sourceWidth?: number` 与 `sourceHeight?: number`。
- `src/App.vue` & `src/styles.css`：
  - 在 `buildRenderPlan()` 中传入 `sourceWidth: media.value?.width` 与 `sourceHeight: media.value?.height`。
  - 在导入视频、选择内置贴纸、套用模板、新增文字图层时，基于源视频分辨率比例（`getMediaScaleFactor()`）自适应计算初始字号、描边宽度、贴纸宽度与居中位置。
  - 在「🔤 文字」与「🎨 贴纸·音效」面板中优化自动启用逻辑，并内嵌热门贴纸快捷点选网格。

## 2. 迁移与兼容性策略

- `source_width` 和 `source_height` 均标记为 `#[serde(default)]` 可选字段，旧版草稿、历史记录、批量队列以及不指定源尺寸的单元测试均保持 100% 兼容。
- 回滚方式：可通过 `git revert` 独立回滚本分支合并提交。

## 3. 自测记录

- **平台环境**：macOS (aarch64-apple-darwin / Apple Silicon)
- **前端类型检查与构建**：
  - 命令：`pnpm build`
  - 预期与实际结果：`vue-tsc --noEmit` 零错误通过，Vite 生产构建成功（759ms）。
- **Rust 编译与单元测试**：
  - 命令：`PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked` 与 `cargo test --manifest-path src-tauri/Cargo.toml`
  - 预期与实际结果：39 项单元测试全部通过（含新增的 `scales_text_and_sticker_from_source_resolution_to_output_width`、`scales_text_and_sticker_with_crop_offset_and_scale`、`detect_system_font_prefers_cjk_font_on_macos`）。
- **真实 FFmpeg 端到端导出冒烟验证**：
  - 步骤：使用 sidecar FFmpeg (`ffmpeg-aarch64-apple-darwin`) 将 `1920×1080` 测试视频叠加居中的 `doge_face.png` 贴纸（源坐标 `x=720, y=220, w=480` -> 映射后 `x=120, y=37, w=80`）与底部中文字幕「这波操作绝了」（源坐标 `x=528, y=810, size=144` -> 映射后 `x=88, y=135, fontsize=24`）导出为 `320×180` GIF，并抽取第 10 帧图像校验。
  - 实际结果：成功生成 193KB 的 `320×180` 动态 GIF，抽帧图像中「旺柴狗头」贴纸居中清晰、中文台词「这波操作绝了」字形完整（无方框）、描边与半透明底框位置与预览完全一致。
