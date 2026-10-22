# 设计说明书：全局滤镜与画面调色 (video-filters)

- 状态：proposed
- Feature slug: video-filters
- 分支：feature/video-filters
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 滤镜规范与映射表

| 滤镜 Key | 中文标签 | FFmpeg 视频滤镜表达式 | 前端 CSS / 预览机制 |
|----------|----------|----------------------|-------------------|
| `none` | 原色 (默认) | *(无)* | `none`, `playbackRate = 1.0` |
| `grayscale` | 🖤 黑白致郁 | `hue=s=0` | `filter: grayscale(100%)` |
| `contrast` | ⚡ 高反差特写 | `eq=contrast=1.6:saturation=1.4` | `filter: contrast(160%) saturate(140%)` |
| `vintage` | 🎞️ 复古胶片 | `colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131` | `filter: sepia(60%) contrast(110%)` |
| `invert` | 🧪 反色负片 | `negate` | `filter: invert(100%)` |
| `fast2x` | 🚀 2x 鬼畜加速 | `setpts=0.5*PTS` | `playbackRate = 2.0` |
| `slow05x` | 🐢 0.5x 慢放 | `setpts=2.0*PTS` | `playbackRate = 0.5` |
| `reverse` | ⏪ 物理倒放 | `reverse` | 播放条提示倒放模式 |

## 2. 处理流与分层架构

```
源视频 [0:v]
  │
  ▼
[裁切 (crop)] (可选)
  │
  ▼
[缩放 (scale)]
  │
  ▼
【画面滤镜 / 调色 / 变速】 (video_filter)  <--- 新增节点 (在字幕与贴纸之前!)
  │
  ▼
[多文字图层 (drawtext...)] (保持前景原有色彩与描边)
  │
  ▼
[贴纸图案 (overlay...)] (保持贴纸原有透明度与色彩)
  │
  ▼
输出格式化 (GIF 调色板 / WebP / AVIF format=yuv420p)
```

## 3. 详细接口设计

### 3.1 Rust 后端 (`src-tauri/src/lib.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPlan {
    ...
    #[serde(default)]
    pub video_filter: Option<String>,
}
```

在 `build_filter_graph` 中：
```rust
fn map_video_filter(key: &str) -> Option<&'static str> {
    match key {
        "grayscale" => Some("hue=s=0"),
        "contrast" => Some("eq=contrast=1.6:saturation=1.4"),
        "vintage" => Some("colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131"),
        "invert" => Some("negate"),
        "fast2x" => Some("setpts=0.5*PTS"),
        "slow05x" => Some("setpts=2.0*PTS"),
        "reverse" => Some("reverse"),
        _ => None,
    }
}
```
在构建完 `scale` 之后追加：
```rust
if let Some(filter_key) = &plan.video_filter {
    if let Some(expr) = map_video_filter(filter_key) {
        source.push_str(&format!(",{}", expr));
    }
}
```

### 3.2 前端状态与预览绑定 (`src/App.vue`)

- 状态：`videoFilter = ref<VideoFilterKey>("none")`；
- 播放器绑定：
  - CSS 类名或内联样式：`:style="{ filter: currentVideoFilterCss }"`
  - 监听 `videoFilter`，当为 `fast2x` 时将 `<video>` 的 `playbackRate = 2.0`，`slow05x` 时为 `0.5`，其他还原为 `1.0`。
