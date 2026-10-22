# 设计说明书：AVIF 动图格式支持 (avif-format)

- 状态：proposed
- Feature slug: avif-format
- 分支：feature/avif-format
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 架构与数据流设计

```
Vue 前端 (App.vue)
  ├── 格式选择: exportFormat = 'avif' | 'webp' | 'gif'
  ├── 路径推导: name_start-end.avif
  └── RenderPlan: { format: 'avif', outputPath: '...avif', ... }
        │
        ▼ (Tauri IPC: render_gif)
Rust 后端 (src-tauri/src/lib.rs)
  ├── 格式校验: format == "avif" <==> ext == ".avif"
  ├── 滤镜图构建: [0:v]...format=yuv420p[vout] (无需二次调色板)
  └── 参数拼装:
        ffmpeg -ss <start> -t <duration> -i <src>
               -filter_complex "[0:v]...[vout]" -map "[vout]"
               -c:v libsvtav1 -preset 8 -crf <26/32/38> -pix_fmt yuv420p -an
               output.avif
```

## 2. 详细接口设计

### 2.1 Rust 后端 (`src-tauri/src/lib.rs`)

```rust
impl RenderPlan {
    fn is_webp(&self) -> bool { ... }
    fn is_avif(&self) -> bool {
        if let Some(fmt) = &self.format {
            if fmt.eq_ignore_ascii_case("avif") {
                return true;
            }
        }
        self.output_path.to_ascii_lowercase().ends_with(".avif")
    }
}
```

- 滤镜处理：
  ```rust
  if plan.is_webp() || plan.is_avif() {
      // 单阶段直接编码，末尾规整格式
      if plan.is_avif() {
          source.push_str(",format=yuv420p");
      }
      ...
  }
  ```

- 参数构建：
  ```rust
  if plan.is_avif() {
      let crf_val = match plan.quality.as_deref() {
          Some("low") => "38",
          Some("high") => "26",
          _ => "32",
      };
      args.extend([
          "-filter_complex".to_string(),
          filter_graph,
          "-map".to_string(),
          "[vout]".to_string(),
          "-c:v".to_string(),
          "libsvtav1".to_string(),
          "-preset".to_string(),
          "8".to_string(),
          "-crf".to_string(),
          crf_val.to_string(),
          "-pix_fmt".to_string(),
          "yuv420p".to_string(),
          "-an".to_string(),
          plan.output_path.clone(),
      ]);
  }
  ```

### 2.2 前端类型与纯函数模型 (`src/lib/desktop.ts`, `src/lib/queue.ts`)

- 导出格式联合类型：`"gif" | "webp" | "avif"`；
- 对话框扩展：文件选择器接受 `.avif`；
- MIME 识别：`.avif` => `image/avif`。

## 3. UI 与交互设计

- 控制面板“格式选择”升级为三段式胶囊切换或下拉选项：
  - `GIF`：经典通用，全平台兼容
  - `WebP`：小体积，主流社交平台首选
  - `AVIF`：超高压缩比，极客与新一代格式
- 卡片标签徽标 `.format-badge.avif` 采用紫色科技风设计（`#a855f7`），突出新一代感。
