# 任务拆解：全局滤镜与画面调色 (video-filters)

- 状态：merged
- Feature slug: video-filters
- 分支：feature/video-filters
- 负责人：Antigravity
- 日期：2026-10-22

## 任务执行清单

- [x] 0. 文档初始化与建立基线
  - [x] 创建 `feature/video-filters` 分支
  - [x] 撰写并在 `docs/features/video-filters/` 提交完整调研、需求、设计与实现计划
- [x] 1. Rust 端视频滤镜管线实现 (`src-tauri/src/lib.rs`)
  - [x] 1.1 `RenderPlan` 引入 `video_filter: Option<String>`
  - [x] 1.2 实现 `map_video_filter`（grayscale, contrast, vintage, invert, fast2x, slow05x, reverse）
  - [x] 1.3 在 `build_filter_graph` 中于 scale 之后、drawtext/overlay 之前注入滤镜
  - [x] 1.4 编写原生 Rust 单元测试：`builds_filter_graph_with_video_filters`
- [x] 2. 前端模型与类型库升级 (`src/lib/desktop.ts`, `src/lib/templates.ts`, `src/lib/queue.ts`)
  - [x] 2.1 在 `RenderPlan` 和 `StudioDraft` 中加入 `videoFilter?: string`
  - [x] 2.2 确保 `createQueueItem` 快照包含 `videoFilter`
  - [x] 2.3 更新 `scripts/test-queue.mjs` 测试包含滤镜的快照
- [x] 3. 前端界面与所见即所得交互 (`src/App.vue`, `src/styles.css`)
  - [x] 3.1 控制面板增加“画风特效”选择器（平铺微型胶囊网格）
  - [x] 3.2 画布背景层绑定实时 CSS filter 与倍速播放联动
  - [x] 3.3 草稿自动保存、恢复与历史记录还原支持 `videoFilter`
  - [x] 3.4 编写精致样式并确保 1280x820 零溢出
- [x] 4. 端到端 Smoke Test 与质量门槛
  - [x] 4.1 编写并运行 `scripts/smoke-test-filters.mjs`
  - [x] 4.2 运行 `node scripts/test-queue.mjs` 全绿通过
  - [x] 4.3 运行 `pnpm build`（vue-tsc + vite）检查通过
  - [x] 4.4 运行 `cargo check --locked` 与 `cargo test` 全绿通过（15 个测试通过）
  - [x] 4.5 检查 `git diff --check`
- [x] 5. 收尾与主分支合入
  - [x] 5.1 更新五件套文档状态为 `ready-to-merge` 并记录自测结果
  - [x] 5.2 非快进合并至 `main`
  - [x] 5.3 文档状态变更为 `merged`，同步根目录 `TODO.md`，推送到远程仓库
