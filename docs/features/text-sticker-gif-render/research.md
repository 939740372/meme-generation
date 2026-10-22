# 文字与贴纸叠加渲染修复调研 (Text & Sticker GIF Render Fix Research)

- **状态**: merged
- **分支**: `fix/text-sticker-gif-render`
- **日期**: 2026-10-22

## 1. 问题背景

用户反馈在添加文字字幕与贴纸素材后导出 GIF 动图时，文字和贴纸存在不会正常渲染到导出 GIF 中的问题（表现为文字变成空白方框或消失、贴纸和字幕在导出动图中完全不可见，或者切换至文字/贴纸面板后未生效）。

## 2. 已确认事实（根因定位）

通过代码审计与本机 FFmpeg 实测，确认存在以下三个核心缺陷：

### 2.1 根因一：macOS 系统字体探测顺序导致中文渲染为缺失方框 (`□□□□`)
- `src-tauri/src/lib.rs` 的 `detect_system_font()` 在 macOS 下按以下顺序查找字体：
  1. `/System/Library/Fonts/PingFang.ttc`
  2. `/System/Library/Fonts/Helvetica.ttc`
- 在现代 macOS（Ventura / Sonoma / Sequoia，Darwin 22+）中，`PingFang.ttc` 已迁移至 `/System/Library/AssetsV2/` 动态字体资产目录，`/System/Library/Fonts/PingFang.ttc` 路径不再存在。
- 因此 `detect_system_font()` 回退到了第 2 顺位的 `/System/Library/Fonts/Helvetica.ttc`。而 `Helvetica.ttc` 仅包含西文字形，**完全不包含中日韩（CJK）汉字字形**，导致所有中文字幕在 FFmpeg `drawtext` 中被渲染为空白方框 `□□□□`。
- 同时在现代 macOS 的 `/System/Library/Fonts/` 目录下，原生内置有完整中文字库：
  - `/System/Library/Fonts/STHeiti Medium.ttc`（华文黑体-中粗，55.7MB）
  - `/System/Library/Fonts/Hiragino Sans GB.ttc`（冬青黑体简体中文，23.5MB）
  - `/System/Library/Fonts/STHeiti Light.ttc`（华文黑体-细体，55.7MB）

### 2.2 根因二：画布预览坐标系（源视频分辨率）与 FFmpeg 滤镜坐标系（导出缩放后分辨率）未做空间映射
- 在前端 `src/App.vue` 中，所有文字图层与贴纸的坐标（`layer.x`, `layer.y`, `imageX`, `imageY`）及尺寸（`layer.size`, `imageWidth`）均以**源视频像素分辨率**（`media.width × media.height`，例如 `1920 × 1080`）为基准存储和拖拽：
  - 点击贴纸（`applySticker`）会将贴纸居中放置在 `x = (1920 - 120) / 2 = 900, y = (1080 - 120) / 2 = 480`。
  - 点击字幕模板（`applyTemplate`）会将文字放置在 `x = 1920 * 0.5 = 960, y = 1080 * 0.88 = 950`。
  - 在画布拖拽文字或贴纸时，坐标范围同样是 `0..1920` 与 `0..1080`。
- 然而在后端 `src-tauri/src/lib.rs` 的 `build_filter_graph()` 中，FFmpeg **先**执行了 `crop` 和 `scale={output_width}:-1`（例如缩放至 `320 × 180`），**后**直接将未经缩放转换的源坐标（如 `x=900:y=480`、`x=960:y=950`）传入 `drawtext` 和 `overlay`。
- 因为 `900 > 320` 且 `480 > 180`，贴纸和字幕被直接绘制到了 `320 × 180` 画布边界之外的虚空中，导致导出的 GIF 中完全看不到贴纸和拖拽后的文字！
- 此外，当开启画面裁切（`crop`）时，后端也未从文字和贴纸坐标中减去 `crop.x` 与 `crop.y` 偏移量。

### 2.3 根因三：前端文字与贴纸面板的启用交互陷阱
- 用户点击右侧检查器的 `🔤 文字` 标签页时，`useText` 默认是 `false`，编辑控件被完全隐藏，容易误以为没有文字编辑入口或已显示的默认文字已生效。
- 用户点击画布顶部的 `🎨 贴纸` 快捷开关或检查器中的贴纸开关时，若尚未打开抽屉选择贴纸（`imagePath` 为空），画布没有任何贴纸出现，且面板中缺少直接点选内置贴纸的快捷网格。
- 高分辨率视频（如 1080p / 4K）导入时，默认字号 `42` 与贴纸默认宽度 `120` 未按源视频宽度比例自适应放大，在 4K 画布上显得过小。

## 3. 决策与方案

1. **修复系统字体探测优先级**：在 macOS 优先探测 `/System/Library/Fonts/STHeiti Medium.ttc`、`/System/Library/Fonts/Hiragino Sans GB.ttc`、`/System/Library/Fonts/STHeiti Light.ttc`、`/System/Library/Fonts/PingFang.ttc` 等 CJK 字体；在 Linux 优先探测 `NotoSansCJK` 与 `wqy-microhei`。
2. **统一前后端坐标空间映射**：
   - 在 `RenderPlan` 中增加可选的 `source_width` 与 `source_height` 字段（并在 `render_gif` 执行时由 `probe_media` 自动兜底补齐）。
   - 在 `build_filter_graph()` 中计算 `base_width = crop.width.unwrap_or(source_width)` 与缩放比 `scale = output_width / base_width`，将文字与贴纸的 `(x - crop_x, y - crop_y)` 及字号、描边、阴影、贴纸宽度精确映射到输出帧像素空间。
3. **优化文字与贴纸启用交互及高分屏默认尺寸**：
   - 在 `🔤 文字` 面板中常驻展示文字编辑区，当用户输入文字、修改样式或新增图层时自动激活 `useText = true`。
   - 在 `🎨 贴纸·音效` 面板中内嵌内置贴纸快捷选择网格，并在开启 `useImage` 且未选贴纸时自动载入首款内置贴纸。
   - 导入高分辨率视频、套用模板或选择贴纸时，按源视频分辨率比例自适应计算合理的初始字号与贴纸宽度。
