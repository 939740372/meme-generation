# 文字与贴纸叠加渲染修复需求文档 (Requirements)

- **状态**: merged
- **分支**: `fix/text-sticker-gif-render`
- **日期**: 2026-10-22

## 1. 目标与范围

确保用户在画布中添加、拖拽、缩放的文字字幕（含中英文、描边、底框、阴影、旋转、斜体、弧形）和透明 PNG 贴纸，在导出为 GIF / WebP / AVIF / MP4 时能够 100% 准确渲染到输出画面中，且位置、比例与画布预览所见即所得。

## 2. 用户故事

1. **中文字幕正常渲染**：作为用户，我在视频上输入中文台词并导出 GIF 时，导出的 GIF 中能清晰显示中文文字，不会出现空白方框（`□□□□`）或文字丢失。
2. **所见即所得的坐标与比例**：作为用户，我在 1080p/4K 视频画布上将文字或贴纸拖到任意位置（包括居中、底部字幕带或局部裁切框内），导出为 240px / 320px / 480px 的 GIF 时，文字和贴纸在输出动图中的相对位置与相对大小与画布预览完全一致，不会飞出画面外。
3. **直观无感的图层启用体验**：作为用户，我切换到「🔤 文字」或「🎨 贴纸·音效」面板编辑内容或点击贴纸时，对应图层自动启用并实时呈现在画布与最终导出结果中。

## 3. 功能需求

- **FR-1 (CJK 字体优先探测)**：`detect_system_font()` 必须在 macOS / Windows / Linux 上优先匹配内置中文字体（macOS 优先 `STHeiti Medium.ttc`、`Hiragino Sans GB.ttc`、`STHeiti Light.ttc`、`PingFang.ttc`）。
- **FR-2 (源分辨率到输出分辨率的坐标与尺寸映射)**：
  - `RenderPlan` 支持携带 `sourceWidth` 与 `sourceHeight`（后端 `render_gif` 自动以 `probe_media` 结果兜底）。
  - `build_filter_graph()` 根据 `crop` 偏移量 `(crop.x, crop.y)` 和缩放系数 `output_width / effective_source_width`，对所有文字图层（普通 `drawtext` 与高级旋转/斜体/弧形图层）的 `x, y, size, stroke_width, shadow_offset` 以及贴纸图层的 `x, y, width` 进行等比转换。
- **FR-3 (慢放滤镜下的高级文字时长对齐)**：当启用 `slow05x`（0.5x 慢放）时，高级文字的透明画布源时长自动乘以 `2.0`，确保慢放动图后半段文字不会提前消失。
- **FR-4 (贴纸路径规范化)**：`resolve_stickers_dir()` 与 `resolve_sfx_dir()` 返回规范化路径，确保 FFmpeg 在任何工作目录下均可读取内置贴纸与音效。
- **FR-5 (前端文字与贴纸即点即用交互)**：
  - 「🔤 文字」面板直接展示文字输入与样式控件，修改台词、字号、颜色或新增图层时自动开启 `useText = true`。
  - 「🎨 贴纸·音效」面板直接提供内置贴纸快捷网格；点击任意贴纸或开启贴纸开关时自动激活 `useImage = true` 并居中放置贴纸。
  - 贴纸初始宽度、默认文字字号与模板字号根据源视频宽度（相对 480px 基准）自适应缩放，确保在 1080p / 4K 视频上预览清晰、比例协调。

## 4. 验收标准 (Acceptance Criteria)

- **AC-1**：在 macOS 环境下导出含中文台词与内置 PNG 贴纸的 `1920×1080 -> 320×180` GIF，输出 GIF 帧中中文文字字形完整清晰（无方框）、贴纸位于画面对应位置且尺寸比例一致。
- **AC-2**：开启画面裁切（`crop`）并在裁切框内放置文字与贴纸，导出的 GIF 中文字与贴纸相对裁切框的位置准确无偏移。
- **AC-3**：单元测试覆盖无 `source_width`（默认 1:1 兼容）与有 `source_width` / `crop` 时的坐标缩放转换。
- **AC-4**：`pnpm build`、`cargo check --locked`、`cargo test` 与 `git diff --check` 全部通过。
