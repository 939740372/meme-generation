# 详细实现文档：可视化时间轴与画布拖拽创作工作区

- 状态：merged
- Feature slug：creative-timeline-canvas
- 分支：feature/creative-timeline-canvas
- 负责人：待定
- 日期：2026-10-22

## 1. 文件和模块计划

| 路径 | 变更 | 目的 |
| --- | --- | --- |
| `src/App.vue` | 增加时间轴组件模板与逻辑、画布拖拽图层逻辑 | 实现时间滑块与文字/贴图所见即所得拖拽 |
| `src/styles.css` | 增加时间轴轨道、滑块手柄及画布拖拽状态样式 | 保证桌面端视觉质感与 1280×820 窗口兼容 |
| `docs/features/creative-timeline-canvas/` | 维护全套 feature 文档 | 记录设计、实现与自测 |

## 2. 实现步骤

1. **时间轴组件**：
   - 在预览区与媒体摘要之间增加 `.timeline-panel`；
   - 响应式计算 `startPercent`、`endPercent` 与 `rangePercent`；
   - 实现左手柄（Start）、右手柄（End）与选区高亮条的平移拖拽事件；
   - 保证与 `startSecs`、`endSecs` 数字输入框完全双向联动，具备最小间隔 0.1s 防交叉约束。
2. **画布拖拽图层**：
   - 为预览区（`preview-stage`）绑定 `ref="stageRef"`，并通过 `ResizeObserver` 实时跟踪视口渲染尺寸；
   - 将原预览文字（`.preview-text`）与贴图（`.preview-sticker`）升级为响应式拖拽元素；
   - 添加 `pointerdown` 事件监听与全局 `pointermove`/`pointerup`，换算实际物理坐标系并限制边界；
   - 增加悬停与拖拽中的视觉反馈（抓取光标、虚线边框、坐标徽章）。
3. **固定视口与样式调优**：
   - 调整时间轴与预览区布局边距，确保在 `1280×820` 固定视口下无溢出和滚动条。

## 3. 测试方案

1. 浏览器与类型检查：`pnpm build`（`vue-tsc --noEmit` & `vite build`）。
2. 手工/逻辑验证：
   - 时间轴左右手柄滑动，数字框对应变化；
   - 在画布中拖动文字和贴图，参数面板 X/Y 对应变化；
   - 实际调用 FFmpeg 导出，验证生成的 GIF 中文字和贴图位置与画布所见一致。

## 4. 自测记录

- 环境：macOS 15+ arm64；Node v22.22.0；pnpm 11.17.0；Rust 1.94.1；自包含静态 FFmpeg 9.0.2。
- `git diff --check`：通过，无多余空行或格式警告。
- `pnpm build`（含 `vue-tsc --noEmit` & `vite build`）：通过。
- `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`：通过。
- `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml`：通过，5 个单元测试全部通过。
- 端到端功能与视觉冒烟：
  - 时间轴组件：基于 4.00s 视频样例，滑块成功设定选区为 0.50s ~ 1.50s；
  - 画布排版坐标：文字设定在物理坐标 (40, 50)；
  - 导出产物：输出 GIF 时长精确为 1.000 秒，尺寸 320×180，帧率 12 FPS，文字绘制在指定位置，无画面裁切或乱码。
- 窗口视口检查：在 `1280×820` 下工作区与时间轴布局紧凑，未产生页面级横向或纵向滚动条。
