# 全帧画质精修与色彩滤镜设计文档 (Design)

- **状态**: ready-to-merge
- **分支**: `feature/frame-image-adjustments`
- **日期**: 2026-10-23

## 1. 数据结构与接口契约

### 1.1 Rust 后端 (`src-tauri/src/lib.rs`)

```rust
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ImageAdjustments {
    #[serde(default)]
    brightness: Option<i32>, // -100..=100
    #[serde(default)]
    contrast: Option<i32>,   // -100..=100
    #[serde(default)]
    saturation: Option<i32>, // -100..=100
    #[serde(default)]
    sharpen: Option<u32>,    // 0..=100
    #[serde(default)]
    blur: Option<u32>,       // 0..=100
    #[serde(default)]
    hue: Option<i32>,        // -180..=180
    #[serde(default)]
    vignette: Option<bool>,
}
```

在 `RenderPlan` 中新增字段：
```rust
#[serde(default)]
image_adjustments: Option<ImageAdjustments>,
```

### 1.2 FFmpeg 滤镜映射算法 (`build_image_adjustments_filter`)

对于 `ImageAdjustments`：
1. **`eq` 色彩增强**：
   - 亮度：$b = \text{brightness} \times 0.005 \in [-0.50, +0.50]$
   - 对比度：$c = 1.0 + \text{contrast} \times 0.01 \in [0.00, 2.00]$
   - 饱和度：$s = 1.0 + \text{saturation} \times 0.015 \in [0.00, 2.50]$（当 $\text{saturation} < 0$ 时按 $1.0 + \text{saturation} \times 0.01$ 映射至 $[0.0, 1.0]$）
   - 仅当存在非零调节项时生成 `eq=brightness={b:.3}:contrast={c:.2}:saturation={s:.2}`。
2. **`hue` 色相旋转**：
   - 当 $\text{hue} \neq 0$ 时，生成 `hue=h={hue}`。
3. **`unsharp` 全帧锐化**：
   - 当 $\text{sharpen} > 0$ 时，$a = (\text{sharpen} / 100.0) \times 2.50 \in [0.03, 2.50]$，生成 `unsharp=5:5:{a:.2}:5:5:0.0`。
4. **`gblur` 高斯模糊**：
   - 当 $\text{blur} > 0$ 时，$\sigma = (\text{blur} / 100.0) \times 8.0 \in [0.08, 8.00]$，生成 `gblur=sigma={sigma:.2}`。
5. **`vignette` 暗角**：
   - 当 $\text{vignette} = \text{true}$ 时，生成 `vignette=PI/4`。

这些滤镜节点紧随 `scale`、`fps` 与 `video_filter` 之后、文字与贴纸叠加之前执行，确保对视频的**每一帧画面**生效，同时不会模糊或破坏叠加在顶层的文字与贴纸。

### 1.3 前端实时预览 (`src/App.vue`)

- 组合 `currentVideoFilterCss` 与 `imageAdjustments` 生成统一的 `combinedMediaFilterCss`：
  - `brightness(${100 + brightness * 0.6}%)`
  - `contrast(${100 + contrast}%)`
  - `saturate(${saturation >= 0 ? 100 + saturation * 1.5 : 100 + saturation}%)`
  - `hue-rotate(${hue}deg)`
  - `blur(${(blur * 0.08).toFixed(2)}px)`
  - 当 `sharpen > 0` 时追加 `url(#studio-sharpen-filter)`，其中 `<feConvolveMatrix>` 核矩阵为 `[0, -k, 0, -k, 1 + 4k, -k, 0, -k, 0]`，$k = (\text{sharpen} / 100) \times 0.85$。
- 当 `vignette` 开启时，在 `.preview-media-container` 内渲染 `.preview-vignette-overlay` 径向暗角层。
