# 设计文档：导出预设与压缩质量控制

- 状态：merged
- Feature slug: export-presets
- 分支：feature/export-presets
- 负责人：待定
- 日期：2026-10-22

## 1. 方案摘要

本方案在现有控制面板中引入预设模型，并在底层转码管线中增强调色板抖动算法选择机制：
1. **预设数据模型（Preset Model）**：
   ```ts
   interface ExportPreset {
     id: string;
     name: string;
     targetSizeLabel: string;
     width: number;
     fps: number;
     quality: "low" | "medium" | "high";
   }
   ```
2. **前后端接口契约（RenderPlan）**：
   在 `RenderPlan` 中追加可选字段：
   - TypeScript: `quality?: "low" | "medium" | "high"`
   - Rust: `pub quality: Option<String>`
3. **调色板算法映射**：
   - `"low"`：使用 `bayer:bayer_scale=3`，有序抖动不仅压缩效率极高，且能消除误差扩散带来的相邻帧杂色颗粒闪烁；
   - `"medium"` / 缺省：使用 `sierra2_4a`，画质与压缩率兼顾的误差扩散算法；
   - `"high"`：使用 `sierra2_4a`（大分辨率与高帧率），保证细腻渐变。

## 2. 状态联动流图

~~~mermaid
flowchart TD
    A["用户导入视频"] --> B["默认加载'微信大表情'预设(320px/12fps)"]
    B --> C{"用户交互操作"}
    C -->|点击预设Pill| D["批量更新 outputWidth, fps, quality"]
    D --> E["activePresetId 保持对应预设ID"]
    C -->|直接修改宽度/FPS/质量| F["activePresetId 自动切换为'custom'"]
    E & F --> G["构建包含 quality 的 RenderPlan"]
    G --> H["Rust: build_filter_graph 匹配 dither 算法"]
    H --> I["FFmpeg 执行调色板生成与导出"]
~~~

## 3. UI 布局与样式设计

- 在右侧控制面板的“02 输出”分区标题下方，加入紧凑平铺的分段式预设标签条：
  - 容器样式：`.preset-pills`，采用水平弹性网格（`grid-template-columns: repeat(4, 1fr)` + 自定义指示）；
  - 按钮样式：`.preset-pill`，深色底面，选中时为品牌酸性黄高亮背景或边框；
  - 提示文案：每个预设带有小型微标签（如 `<1MB`, `<5MB`, `HD`）；
- 紧凑布局：缩小各控件外边距 2px，保证即使新增一排预设按钮，右侧控制面板和整个页面高度依然完全锁定在 820px 内。

## 4. 容错与向后兼容

- 若调用方未传递 `quality`，Rust 端默认使用 `"medium"`（即保持原有的 `sierra2_4a` 行为不变）；
- 若用户手动输入异常值，校验规则依然生效（`output_width 32..=1920`, `fps 1..=60`）。
