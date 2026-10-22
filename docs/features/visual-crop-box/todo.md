# 任务拆解：可视化矩形裁切框与画布交互

- 状态：implementing
- Feature slug: visual-crop-box
- 分支：feature/visual-crop-box
- 负责人：待定
- 日期：2026-10-22

## 阶段一：文档与分支准备
- [x] 创建 feature 分支 `feature/visual-crop-box`
- [x] 完成 `docs/features/visual-crop-box/` 五件套文档初稿
- [x] 提交初始文档，遵循常规提交规范

## 阶段二：坐标映射与核心交互逻辑 (App.vue)
- [ ] 计算裁切选框在预览画布上的展示坐标与宽高（`cropDisplayLeft`, `cropDisplayTop`, `cropDisplayWidth`, `cropDisplayHeight`）
- [ ] 实现选框中心区域自由平移拖拽交互（带源视频边界钳制）
- [ ] 实现 8 个手柄（NW, N, NE, E, SE, S, SW, W）的指向性拉伸缩放交互（带最小尺寸保护与边界钳制）
- [ ] 增加拖拽活动状态响应（`isDraggingCrop`, `activeCropHandle`）及实时尺寸坐标胶囊气泡
- [ ] 实现快捷构图动作辅助（居中对齐 `cropCenter`、1:1 正方形 `cropSquare`、重置铺满 `cropReset`）

## 阶段三：视觉与样式实现 (styles.css & App.vue)
- [ ] 在中央预览舞台 `.preview-stage` 内渲染裁切交互层（条件：`media && useCrop`）
- [ ] 实现 CSS Box-Shadow 暗色半透明外圈遮罩
- [ ] 实现九宫格三分参考线（Rule-of-Thirds Grid）
- [ ] 绘制 8 个方向调节手柄（角手柄与边手柄微观视觉区分，悬浮高亮）
- [ ] 在控制面板裁切区增加快捷构图按钮组（居中、1:1、铺满）
- [ ] 严格审查样式与排版，严防 1280×820 固定视口出现滚动条

## 阶段四：自测、质量门槛与合并
- [ ] 执行 `pnpm build`（vue-tsc + vite build）
- [ ] 执行 `cargo check --locked` 与 `cargo test`
- [ ] 执行 `git diff --check`
- [ ] 记录自测结果至 `implementation.md`
- [ ] 更新 feature 文档状态为 `ready-to-merge`
- [ ] 切回 `main` 分支执行非快进合并：`git merge --no-ff feature/visual-crop-box`
- [ ] 更新 `TODO.md` 并标记 feature 文档为 `merged`
- [ ] 推送远程仓库
