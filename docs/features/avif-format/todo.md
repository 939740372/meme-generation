# 任务拆解：AVIF 动图格式支持 (avif-format)

- 状态：proposed
- Feature slug: avif-format
- 分支：feature/avif-format
- 负责人：Antigravity
- 日期：2026-10-22

## 任务执行清单

- [ ] 0. 文档初始化与建立基线
  - [x] 创建 `feature/avif-format` 分支
  - [x] 撰写并在 `docs/features/avif-format/` 提交完整调研、需求、设计与实现计划
- [ ] 1. Rust 端 AVIF 转码管线实现 (`src-tauri/src/lib.rs`)
  - [ ] 1.1 扩展 `RenderPlan` 方法 `is_avif()` 与格式校验
  - [ ] 1.2 在 `build_filter_graph` 中对 AVIF 追加 `format=yuv420p`
  - [ ] 1.3 在 `build_ffmpeg_args` 中构建 `libsvtav1` 编码参数与三档 CRF
  - [ ] 1.4 在 `read_file_as_data_url` 中识别 `image/avif` MIME
  - [ ] 1.5 增加原生 Rust 单元测试：`builds_avif_filter_graph_and_args` 与格式校验
- [ ] 2. 前端模型与核心库升级 (`src/lib/desktop.ts`, `src/lib/queue.ts`)
  - [ ] 2.1 扩展 `format` 支持 `"avif"` 联合类型
  - [ ] 2.2 更新 `generateQueueOutputPath` 与 `createQueueItem`
  - [ ] 2.3 更新 `scripts/test-queue.mjs` 测试 AVIF 项生成
- [ ] 3. 前端交互与视觉适配 (`src/App.vue`, `src/styles.css`)
  - [ ] 3.1 格式选择器添加 AVIF 选项（GIF / WebP / AVIF）
  - [ ] 3.2 动态更新导出主按钮文案（`导出 AVIF ↗`）
  - [ ] 3.3 历史记录与队列卡片适配 AVIF 徽标与预览
  - [ ] 3.4 添加 `.format-badge.avif` 现代紫样式
- [ ] 4. 端到端 Smoke Test 与质量门槛
  - [ ] 4.1 编写并运行 `scripts/smoke-test-avif.mjs` 验证动图文件生成与体积
  - [ ] 4.2 运行 `node scripts/test-queue.mjs` 全绿通过
  - [ ] 4.3 运行 `pnpm build`（vue-tsc + vite）检查通过
  - [ ] 4.4 运行 `cargo check --locked` 与 `cargo test` 全绿通过
  - [ ] 4.5 检查 `git diff --check`
- [ ] 5. 收尾与主分支合入
  - [ ] 5.1 更新五件套文档状态为 `ready-to-merge` 并记录自测结果
  - [ ] 5.2 非快进合并至 `main`
  - [ ] 5.3 文档状态变更为 `merged`，同步根目录 `TODO.md`，推送到远程仓库
