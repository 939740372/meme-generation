# 文字与贴纸叠加渲染修复设计文档 (Design)

- **状态**: merged
- **分支**: `fix/text-sticker-gif-render`
- **日期**: 2026-10-22

## 1. 模块边界与数据契约

### 1.1 `RenderPlan` 扩展 (`src-tauri/src/lib.rs` & `src/lib/desktop.ts`)

在 `RenderPlan` 中新增两个向后兼容的可选字段：

```rust
#[serde(default)]
source_width: Option<u32>,
#[serde(default)]
source_height: Option<u32>,
```

- 当 `source_width` 未提供（例如旧版草稿、历史记录或现有单元测试直接构造 `RenderPlan`）时，基准宽度回退为 `plan.crop.as_ref().map(|c| c.width).unwrap_or(plan.output_width)`，此时缩放因子 `scale_factor = 1.0`，完全保持向后兼容。
- 在 `render_gif` Tauri 命令中，调用 `probe_media` 获取真实视频宽高后，若 `plan.source_width.is_none()`，自动填入 `Some(media.width)` 与 `Some(media.height)`。

### 1.2 坐标与尺寸转换模型 (`build_filter_graph`)

设源视频宽度为 $W_{\text{src}}$，裁切区域为 $(X_{\text{crop}}, Y_{\text{crop}}, W_{\text{crop}}, H_{\text{crop}})$（未开启裁切时 $X_{\text{crop}} = 0, Y_{\text{crop}} = 0, W_{\text{crop}} = W_{\text{src}}$），目标导出宽度为 $W_{\text{out}}$。

则空间缩放因子为：

$$
s = \frac{W_{\text{out}}}{\max(1, W_{\text{crop}})}
$$

对于任意文字图层 $L$ 或贴纸图层 $I$：
- **转换后坐标**：
  $$
  x' = \text{round}\big((x - X_{\text{crop}}) \cdot s\big), \quad y' = \text{round}\big((y - Y_{\text{crop}}) \cdot s\big)
  $$
- **转换后文字字号、描边、阴影与底框边距**：
  $$
  \text{size}' = \max\big(8, \text{round}(\text{size} \cdot s)\big)
  $$
  $$
  \text{stroke}' = \begin{cases} \max\big(1, \text{round}(\text{stroke} \cdot s)\big) & \text{stroke} > 0 \\ 0 & \text{otherwise} \end{cases}
  $$
  $$
  \text{shadow}' = \begin{cases} \max\big(1, \text{round}(\text{shadow} \cdot s)\big) & \text{shadow} > 0 \\ 0 & \text{otherwise} \end{cases}
  $$
- **转换后贴纸宽度**：
  $$
  W_{\text{sticker}}' = \max\big(8, \text{round}(W_{\text{sticker}} \cdot s)\big)
  $$

### 1.3 系统字体探测 (`detect_system_font`)

在 macOS 上按以下优先级探测：
1. `/System/Library/Fonts/STHeiti Medium.ttc`
2. `/System/Library/Fonts/Hiragino Sans GB.ttc`
3. `/System/Library/Fonts/STHeiti Light.ttc`
4. `/System/Library/Fonts/PingFang.ttc`
5. `/System/Library/Fonts/Supplemental/Songti.ttc`
6. `/System/Library/Fonts/Supplemental/Arial Unicode.ttf`
7. `/Library/Fonts/Arial Unicode.ttf`
8. `/System/Library/Fonts/Helvetica.ttc`
9. `/System/Library/Fonts/Supplemental/Arial.ttf`
10. `/Library/Fonts/Arial.ttf`

### 1.4 前端自适应比例与交互加固 (`src/App.vue`)

- 定义辅助函数 `getMediaScaleFactor()`：当导入视频宽度大于 `480px` 时，返回 `media.width / 480`（限制在 `0.5 .. 6.0`），用于在导入视频、添加文字图层、套用模板、选择内置贴纸时将基准字号/贴纸宽度自适应映射到源视频像素空间。
- 在 `🔤 文字` 检查器面板中，当用户编辑台词、字号、描边、底框或高级变形参数时，若 `useText` 为 `false`，自动置为 `true`。
- 在 `🎨 贴纸·音效` 检查器面板中，内嵌内置贴纸快速点选网格；当用户打开 `useImage` 开关且当前未选择贴纸时，自动应用首款内置贴纸。
