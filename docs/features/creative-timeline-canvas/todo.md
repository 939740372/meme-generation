# 细化 TODO：可视化时间轴与画布拖拽创作工作区

- 状态：implementing
- Feature slug：creative-timeline-canvas
- 分支：feature/creative-timeline-canvas
- 负责人：待定
- 日期：2026-10-22

## 1. 可视化时间轴组件

- [ ] 在 `src/App.vue` 增加时间轴组件模板与双滑块手柄
- [ ] 实现 `startSecs` 与 `endSecs` 的滑块拖拽逻辑与边界计算
- [ ] 实现时间轴与表单输入框的双向同步
- [ ] 增加选区时间长度与提示文字展示

## 2. 画布内文字与贴图拖拽

- [ ] 在 `preview-stage` 挂载 `ref` 并计算舞台真实显示尺寸与视频物理分辨率的比例
- [ ] 将 `.preview-text` 改造为可拖拽组件，支持指针拖拽并实时更新 `textX, textY`
- [ ] 将 `.preview-sticker` 改造为可拖拽组件，支持指针拖拽并实时更新 `imageX, imageY`
- [ ] 限制元素拖拽不能超出视频有效边界
- [ ] 增加拖拽悬停反馈（抓取光标、高亮虚线框）

## 3. 样式与固定视口兼容

- [ ] 在 `src/styles.css` 中编写时间轴与画布拖拽控制层样式
- [ ] 检查 `1280×820` 固定视口，确保无横向/纵向滚动条

## 4. 自测与合并

- [ ] 执行 `git diff --check`
- [ ] 执行 `pnpm build`
- [ ] 执行 `cargo check --locked` 与 `cargo test`
- [ ] 执行真实媒体导出验证：确认拖拽位置与导出的 GIF 完全一致
- [ ] 更新 feature 文档状态为 `ready-to-merge`
- [ ] 合并到 `main` 分支并将状态更新为 `merged`
- [ ] 更新根目录 `TODO.md`
