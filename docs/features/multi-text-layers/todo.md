# 任务拆解：多文字图层支持

- 状态：ready-to-merge
- Feature slug: multi-text-layers
- 分支：feature/multi-text-layers
- 负责人：Antigravity
- 日期：2026-10-22

## 任务执行清单

- [x] 0. 文档初始化与建立基线
  - [x] 创建 `feature/multi-text-layers` 分支
  - [x] 撰写并在 `docs/features/multi-text-layers/` 提交完整需求、设计与实现计划文档
- [x] 1. Rust 核心能力扩展 (`src-tauri/src/lib.rs`)
  - [x] 1.1 `RenderPlan` 支持 `text_layers: Option<Vec<TextOverlay>>` 与 `active_text_layers(&self)`
  - [x] 1.2 `validate_render_plan` 增加多图层遍历校验
  - [x] 1.3 `build_filter_graph` 实现多图层链式拼接 `drawtext`
  - [x] 1.4 编写针对多文字图层链式构建的单元测试
- [x] 2. 前端接口与数据模型 (`src/lib/desktop.ts`, `src/lib/templates.ts`)
  - [x] 2.1 `desktop.ts` 更新 `TextOverlay` 与 `RenderPlan` 类型
  - [x] 2.2 `templates.ts` 更新 `StudioDraft` 支持 `textLayers` 与 `activeLayerId`
  - [x] 2.3 更新 `test-templates.mjs` 测试用例并测试通过
- [x] 3. 前端交互与画布多图层实现 (`src/App.vue`, `src/styles.css`)
  - [x] 3.1 引入 `textLayers` 响应式列表与 `activeLayerId`
  - [x] 3.2 实现控制面板图层标签栏（增加、删除、显隐开关、选中切换）
  - [x] 3.3 画布内遍历渲染各图层、点击选中指示框与独立拖拽排版
  - [x] 3.4 模板套用与草稿持久化/水合恢复多图层适配
  - [x] 3.5 编写紧凑精致样式，确保 1280x820 视口零多余滚动条
- [x] 4. 严格验证与质量门槛
  - [x] 4.1 运行 `pnpm build`（vue-tsc + vite）检查通过
  - [x] 4.2 运行 `cargo check --locked` 与 `cargo test`（12 passed）
  - [x] 4.3 执行多文字真实导出 Smoke Test（GIF 与 WebP 均转码成功）
  - [x] 4.4 检查 `git diff --check`
- [ ] 5. 收尾与主分支合入
  - [x] 5.1 更新五件套文档状态为 `ready-to-merge` 并记录自测结果
  - [ ] 5.2 非快进合并至 `main`
  - [ ] 5.3 文档状态变更为 `merged`，同步根目录 `TODO.md`，推送到远程仓库
