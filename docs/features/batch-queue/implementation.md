# 实现计划与自测：批量处理与导出任务队列 (batch-queue)

- 状态：proposed
- Feature slug: batch-queue
- 分支：feature/batch-queue
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 改动清单

| 模块 | 文件 | 变动说明 |
|------|------|----------|
| 队列核心逻辑 | `src/lib/queue.ts` | 1. 定义 `QueueItem`, `QueueItemStatus` 等类型<br>2. 实现 `generateQueueOutputPath` 防冲突路径生成算法<br>3. 实现队列持久化（可选或内存保持）与纯函数计算辅助 |
| 测试脚本 | `scripts/test-queue.mjs` | 编写纯逻辑单元测试：路径防冲突、队列增删查改、状态转换断言 |
| 前端 / 界面 | `src/App.vue` | 1. 引入 `queueItems`, `isQueueRunning`, `activeQueueTaskId`<br>2. 实现“＋ 加入队列”快照捕获与防冲突命名<br>3. 实现队列调度器 `runQueueScheduler` 串行执行状态机<br>4. 抽屉集成第 4 个 Tab“📋 导出队列 (N)”与卡片组件<br>5. 底部操作栏适配队列快捷按钮 |
| 前端 / 样式 | `src/styles.css` | 队列状态徽标、动态微型进度条、操作卡片与抽屉工具条样式 |
| 文档 | `docs/features/batch-queue/*` | 维护五件套文档与任务清单 |

## 2. 迁移与兼容策略

- 向后兼容：完全不影响现有的“立即导出”单图工作流；
- 历史记录同步：队列任务每完成一项，无缝写入现有的 `ExportRecord` 历史库。

## 3. 自测验证方案

### 3.1 自动化检查
```bash
# 1. 队列纯逻辑单元测试
node scripts/test-queue.mjs

# 2. 前端类型检查与打包
pnpm build

# 3. Rust 编译与单元测试
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml

# 4. 代码规范检查
git diff --check
```

### 3.2 真实功能验收 (Smoke Test)
1. **加入多任务到队列**：
   - 载入测试视频，选择 0.0s~0.5s，点击“加入队列”，检查队列数为 1；
   - 切换到 0.5s~1.0s，修改文字为“第二幕”，格式切为 WebP，点击“加入队列”，检查队列数为 2；
   - 检查两项的输出文件名是否独立防冲突。
2. **批量启动与顺序调度**：
   - 打开抽屉“📋 导出队列”，点击“🚀 开始批量处理”；
   - 观察任务 1 变为“渲染中”，进度条平滑跑动，完成后变成“已完成”；
   - 任务 2 自动紧接着开始转码，完成后整体批量处理提示完成；
   - 检查本地是否生成了 2 个真实的动图文件且均可播放。
3. **容错与清空**：
   - 测试“清空已完成”按钮，已完成项被清除；
   - 测试单个任务“重试”。

## 4. 自测记录

- 平台环境：macOS (Darwin aarch64 - Apple Silicon M-series)
- 执行记录：待实现完成后填充
