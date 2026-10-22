# 实现计划与交付记录：极简创作工作台布局重构 (clean-studio-layout)

- 状态：merged
- Feature slug：clean-studio-layout
- 分支：feature/clean-studio-layout
- 负责人：Meme Studio Team
- 日期：2026-10-22

## 1. 文件级实现计划

| 文件 | 变更类型 | 说明 |
| --- | --- | --- |
| `src/App.vue` | 修改 | 重构页面宏观骨架为三段式工作台（一体化顶栏 + 左画布时间轴/右四分栏属性检查器 + 底部导出栏）；新增 `activeInspectorTab`、`toggleDrawerTab`、折叠面板状态及画布拖拽自动切换 Tab 联动 |
| `src/styles.css` | 修改 | 升级为极简工作台样式体系，优化顶栏胶囊、一体化时间轴内联输入、右侧分段 Tab 导航、分组卡片及折叠面板样式 |
| `docs/features/clean-studio-layout/*` | 新增 | 补齐五件套文档与自测记录 |
| `TODO.md` | 修改 | 同步更新产品路线图记录 |

## 2. 迁移与兼容策略

- 纯前端视图布局重构，不改变任何 Tauri IPC 接口、Rust 结构体或 `localStorage` 存储键名；
- 回滚方式：可通过 Git 直接回退 `feature/clean-studio-layout` 合并提交，无任何数据迁移副作用。

## 3. 测试方案

1. 前端类型检查与生产构建：`pnpm build`
2. Rust 编译与单元测试：`cargo check --manifest-path src-tauri/Cargo.toml --locked` 与 `cargo test --manifest-path src-tauri/Cargo.toml`
3. 自动化功能测试脚本回归：`node scripts/test-templates.mjs`、`node scripts/test-stickers.mjs`、`node scripts/test-queue.mjs`
4. 格式与空白检查：`git diff --check`

## 4. 自测记录

- 执行时间：2026-10-22 18:58
- 平台环境：macOS (Darwin 24.3.0, Apple Silicon aarch64)
- 测试命令与实际结果：
  1. `pnpm build` (`vue-tsc --noEmit && vite build`)：
     `✓ 19 modules transformed. ✓ built in 2.26s`，零 TypeScript 错误；
  2. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`：
     `Finished dev profile in 2.36s`，零编译警告；
  3. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml`：
     `test result: ok. 36 passed; 0 failed`；
  4. `node scripts/test-templates.mjs && node scripts/test-stickers.mjs && node scripts/test-queue.mjs`：
     模板与草稿持久化、18 款透明贴纸完整性、批量队列调度测试全部通过；
  5. `git diff --check`：
     零格式与空白错误。
- 已知限制或未覆盖项：无，所有既有功能 100% 保留并收纳进对应的分栏与折叠面板。
