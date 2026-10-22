# 实现计划与交付记录：同名导出自动顺延序号不覆盖 (auto-numbered-export)

- 状态：merged
- Feature slug：auto-numbered-export
- 分支：feature/auto-numbered-export
- 负责人：Meme Studio Team
- 日期：2026-10-22

## 1. 文件级实现计划

| 文件 | 变更类型 | 说明 |
| --- | --- | --- |
| `src-tauri/src/lib.rs` | 修改 | 实现 `parse_numbered_stem` 与 `resolve_unique_output_path`；在 `render_gif` 中自动解析唯一文件路径；移除 `validate_render_plan` 中的 `OUTPUT_EXISTS` 硬拦截；增加对应单测 |
| `src/lib/queue.ts` | 修改 | 将 `generateQueueOutputPath` 序号格式对齐为系统规范 ` (<N>)` |
| `scripts/test-queue.mjs` | 修改 | 同步断言预期结果 |
| `docs/features/auto-numbered-export/*` | 新增 | 全套 feature 文档、技术设计与自测记录 |

## 2. 迁移与兼容策略

- 本次变更属于无破坏性、向后兼容的体验提升；
- 原本报错中断的同名导出场景变为自动成功导出并顺延，不影响已有历史记录与已导出文件；
- 临时文件或已存在文件 100% 得到保护，无数据丢失风险。

## 3. 测试与自测方案

### 3.1 自动化检查
1. Rust 单元测试：
   ```bash
   PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml
   ```
2. 前端类型检查与构建：
   ```bash
   pnpm build
   ```
3. 队列单测：
   ```bash
   node scripts/test-queue.mjs
   ```
4. 代码风格检查：
   ```bash
   git diff --check
   ```

### 3.2 交互与冒烟验证项
- [x] 导出一次生成 `foo.gif`；
- [x] 不修改输出路径，再次点击导出，成功生成 `foo (1).gif`，验证磁盘上 `foo.gif` 和 `foo (1).gif` 两个文件同时存在且大小有效；
- [x] 连续再次点击导出，成功生成 `foo (2).gif`；
- [x] 前端历史记录抽屉中依次展示 `foo.gif`、`foo (1).gif`、`foo (2).gif`；
- [x] 点击历史记录项中的「访达定位」可分别正确定位到每个物理文件。

## 4. 自测记录

- 执行时间：2026-10-22 18:39
- 平台环境：macOS (Darwin 24.3.0, Apple Silicon aarch64)
- 测试命令与输出：
  1. `cargo test --manifest-path src-tauri/Cargo.toml`:
     `36 passed; 0 failed; 0 ignored; finished in 0.14s`，新增测试 `parses_numbered_stem_correctly`、`resolves_unique_output_path_without_overwrite` 全部通过；
  2. `pnpm build`:
     `$ vue-tsc --noEmit && vite build` -> `✓ built in 1.39s`，前端类型与打包零错误；
  3. `cargo check --manifest-path src-tauri/Cargo.toml --locked`:
     通过，零警告；
  4. `node scripts/test-queue.mjs`:
     `ALL QUEUE TESTS PASSED SUCCESSFULLY!`；
  5. `node scripts/smoke-test-queue.mjs`:
     批量队列 smoke test 全部通过；
  6. `git diff --check`:
     无任何空白与格式异常。
- 验收结论：达到 ready-to-merge 门槛。
