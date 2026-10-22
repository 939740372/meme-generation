# 任务拆解：批量处理与导出任务队列 (batch-queue)

- 状态：ready-to-merge
- Feature slug: batch-queue
- 分支：feature/batch-queue
- 负责人：Antigravity
- 日期：2026-10-22

## 任务执行清单

- [x] 0. 文档初始化与建立基线
  - [x] 创建 `feature/batch-queue` 分支
  - [x] 撰写并在 `docs/features/batch-queue/` 提交完整调研、需求、设计与实现计划
- [x] 1. 队列核心模块设计与纯逻辑测试 (`src/lib/queue.ts`, `scripts/test-queue.mjs`)
  - [x] 1.1 编写 `src/lib/queue.ts`，定义 `QueueItem`, `QueueItemStatus` 等模型
  - [x] 1.2 实现 `generateQueueOutputPath` 唯一文件名生成函数与防冲突重命名
  - [x] 1.3 编写 `scripts/test-queue.mjs` 测试用例并运行通过
- [x] 2. 状态机调度器与前端逻辑 (`src/App.vue`)
  - [x] 2.1 引入 `queueItems`, `isQueueRunning`, `activeQueueTaskId`
  - [x] 2.2 实现“＋ 加入队列”：快照不可变捕获与防冲突生成
  - [x] 2.3 实现调度状态机 `runQueueScheduler`，顺序驱动单任务 FIFO 转码
  - [x] 2.4 实现队列控制：开始、暂停、单个重试、单个删除、清空已完成、恢复参数
- [x] 3. 队列抽屉与操作栏交互 (`src/App.vue`, `src/styles.css`)
  - [x] 3.1 抽屉集成第 4 个 Tab：“📋 导出队列 (N)”
  - [x] 3.2 渲染队列任务卡片（状态标签、进度条、时间区间、路径指示、操作按钮）
  - [x] 3.3 底部操作栏增加“＋ 加入队列”与“📋 队列 (N)”快捷入口
  - [x] 3.4 编写精致暗黑主题样式，确保 1280x820 视口零多余滚动条
- [x] 4. 严格验证与质量门槛
  - [x] 4.1 运行 `node scripts/test-queue.mjs` 通过
  - [x] 4.2 运行 `pnpm build`（vue-tsc + vite）检查通过
  - [x] 4.3 运行 `cargo check --locked` 与 `cargo test` 全绿通过（13 个测试通过）
  - [x] 4.4 执行真实多任务批量队列导出 Smoke Test（GIF 与 WebP）
  - [x] 4.5 检查 `git diff --check`
- [ ] 5. 收尾与主分支合入
  - [x] 5.1 更新五件套文档状态为 `ready-to-merge` 并记录自测结果
  - [ ] 5.2 非快进合并至 `main`
  - [ ] 5.3 文档状态变更为 `merged`，同步根目录 `TODO.md`，推送到远程仓库
