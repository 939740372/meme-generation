# 实现方案：高级文字排版效果 (advanced-text-effects)

- 状态：implementing
- Feature slug: advanced-text-effects
- 分支：feature/advanced-text-effects
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 文件级实现计划

| 文件 | 变更类型 | 说明 |
|------|----------|------|
| `src-tauri/src/lib.rs` | 修改 | `TextOverlay` 新增 `rotation`, `arc_radius`, `is_italic`, `shadow_offset`, `shadow_color`；升级 `build_filter_graph` 算法支持旋转、斜切、弧形与立体阴影滤镜链；编写原生单元测试 |
| `src/lib/desktop.ts` | 修改 | `TextOverlay` 接口同步补充高级排版字段 |
| `src/lib/templates.ts` | 修改 | `CanvasTextLayer`, `StudioDraft` 同步扩展高级排版字段与初始默认值 |
| `src/App.vue` | 修改 | 图层控制面板新增旋转角度、弧形弯曲度、斜体切换与阴影控件；画布增强支持旋转/斜体/弯曲/阴影的高保真 Canvas 渲染与中心拖拽 |
| `src/styles.css` | 修改 | 旋转快捷预设胶囊、弧度滑块、阴影拾色与排版控件视觉样式 |
| `scripts/smoke-test-advanced-text.mjs` | 新增 | 端到端自动化冒烟测试（验证旋转、斜体、阴影与弧形弯曲的真实 FFmpeg 导出） |
| `TODO.md` | 修改 | 记录特性完成进度与文档关联 |

## 2. 兼容性与迁移策略

- **向下兼容**：所有新增字段均为可选类型（`Option<...>`），旧版草稿、历史记录与排版模板无缝反序列化，缺省值自动归零为标准横排无阴影文本。
- **全格式支持**：滤镜图全部作用于视频基础图层，因此生成的 GIF、WebP、AVIF 与 MP4 均 100% 具备完全一致的视觉输出。

## 3. 测试方案

1. **Rust 单元测试**：
   - `builds_drawtext_filter_graph_with_shadow`：验证阴影 `shadowx/shadowy/shadowcolor` 滤镜参数生成；
   - `builds_drawtext_filter_graph_with_rotation`：验证旋转透明画板 `rotate` 与居中 `overlay` 组装；
   - `builds_drawtext_filter_graph_with_italic_shear`：验证斜体 `shear=shx=-0.22` 参数生成；
   - `builds_drawtext_filter_graph_with_arc_curvature`：验证弧形弯曲字符级多路分解与圆弧角度定位；
   - `builds_drawtext_filter_graph_combined_advanced_effects`：验证旋转、斜体、阴影与描边复合生效。
2. **端到端冒烟测试 (`smoke-test-advanced-text.mjs`)**：
   - 场景 1：45° 倾斜文字叠加导出 MP4 与 GIF，ffprobe 验证有效性；
   - 场景 2：弧形弯曲文字（头顶高能）真实转码，验证各字符完整合成；
   - 场景 3：斜体 + 立体阴影文字真实转码，验证各滤镜图协同工作。
3. **质量门槛与构建检查**：
   - `pnpm build`（`vue-tsc` + `vite`）
   - `cargo test`
   - `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`
   - `git diff --check`

## 4. 自测记录

- 状态：待执行
- 测试平台：macOS 15.3 (Apple Silicon aarch64)
- 执行结果将同步记录在此。
