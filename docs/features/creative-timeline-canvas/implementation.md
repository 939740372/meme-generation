# 详细实现文档：可视化时间轴与画布拖拽创作工作区

- 状态：implementing
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
   - 在预览区与媒体摘要之间增加 `.timeline-card`；
   - 响应式计算 `startPercent` 和 `endPercent`；
   - 实现左手柄、右手柄与中间选区拖拽事件；
   - 保证与 `startSecs`、`endSecs` 数字输入框完全双向联动。
2. **画布拖拽图层**：
   - 为预览区（`preview-stage`）绑定 `ref="stageRef"`；
   - 将原预览文字（`.preview-text`）与贴图（`.preview-sticker`）升级为可拖拽元素；
   - 添加 `pointerdown` 事件监听，换算实际物理坐标系并限制边界；
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

*(待实现后更新)*
