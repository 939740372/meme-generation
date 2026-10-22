# 任务清单：同名导出自动顺延序号不覆盖 (auto-numbered-export)

- 状态：merged
- Feature slug：auto-numbered-export
- 分支：feature/auto-numbered-export
- 负责人：Meme Studio Team
- 日期：2026-10-22

## 1. 任务拆解

- [x] **1. Rust 路径算法实现 (`src-tauri/src/lib.rs`)**
  - [x] 编写 `parse_numbered_stem` 识别 ` (<N>)` 后缀
  - [x] 编写 `resolve_unique_output_path` 循环探测未占用的递增序号
  - [x] 编写 Rust 单元测试验证各种边界场景（不存在、已存在单文件、已存在多个连续文件、自带序号文件名）
- [x] **2. 接入转码流程 (`src-tauri/src/lib.rs`)**
  - [x] 移除 `validate_render_plan` 中的 `OUTPUT_EXISTS` 错误拦截
  - [x] 在 `render_gif` 启动时将 `plan.output_path` 规范化为 `resolve_unique_output_path`
- [x] **3. 前端与批量队列对齐 (`src/lib/queue.ts`, `scripts/test-queue.mjs`)**
  - [x] 更新 `generateQueueOutputPath` 命名规则为 ` (<N>)`
  - [x] 更新 `scripts/test-queue.mjs` 测试断言
- [x] **4. 自动化验证与质量门槛**
  - [x] 运行 `cargo test` 全部 36 项测试通过
  - [x] 运行 `node scripts/test-queue.mjs` 测试通过
  - [x] 运行 `pnpm build` 构建通过
  - [x] `git diff --check` 无违规
- [x] **5. 合并与交付**
  - [x] 更新 `implementation.md` 自测记录
  - [x] 提交代码并合并回 `main`
