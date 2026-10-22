# 实现计划与自测：常用表情包模板与草稿暂存

- 状态：merged
- Feature slug: templates-and-drafts
- 分支：feature/templates-and-drafts
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 改动清单

| 模块 | 文件 | 变动说明 |
|------|------|----------|
| 数据与类型 | `src/lib/templates.ts` | 新建模板模块：定义 `MemeTemplate`、`StudioDraft`、3 款内置模板常量、自定义模板本地增删改查逻辑与草稿读写工具 |
| 前端 / 界面 | `src/App.vue` | 1. 自动草稿持久化监听（防抖 400ms）与启动自动水合恢复<br>2. 抽屉增加双 Tab（导出历史 / 表情包模板）<br>3. 文字控制区增加“存为模板”与“套用模板”交互<br>4. 增加“重置工作区/清空草稿”动作 |
| 前端 / 样式 | `src/styles.css` | 抽屉 Tab 切换、模板卡片布局、小药丸徽章与“存为模板”弹窗样式 |
| 自动化单测 | `scripts/test-templates.mjs` | Node 原生测试脚本：测试内置模板完整性、自定义模板 CRUD 与草稿读写还原 |
| 文档 | `docs/features/templates-and-drafts/*` | 调研、需求、设计、实现与任务清单维护 |

## 2. 迁移与兼容策略

- **草稿数据**：带 `version: 1` 标识，读取时若反序列化失败或缺少关键字段，优雅 catch 并静默忽略，不阻塞应用主渲染；
- **模板向后兼容**：内置模板为静态内存常量，永不丢失；自定义模板保存在 `localStorage`，独立存储键 `meme_studio_custom_templates`。

## 3. 自测验证方案

### 3.1 自动化检查
```bash
# 1. 前端类型检查与打包
pnpm build

# 2. Rust 编译与单元测试保持正常
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml

# 3. 模板与草稿自动化单元测试
node scripts/test-templates.mjs

# 4. 提交物与空白符检查
git diff --check
```

### 3.2 交互验证 (Smoke Test)
1. **草稿恢复测试**：
   - 验证防抖写入草稿到本地存储；
   - 启动时成功恢复草稿中保存的视频、时间裁切区间、文字内容与描边/底框样式；
   - 点击“重置工作区”，工作区所有参数重置为初始状态且草稿被清空。
2. **内置模板套用测试**：
   - 分别测试“经典影视字幕条”、“顶部大字吐槽”、“名场面大字报”；
   - 验证文字字号、描边、底框立即所见即所得生效，保留已有文字内容。
3. **自定义模板测试**：
   - 调整自定义样式，点击“存为模板”，输入名称“我的测试模板”保存；
   - 验证模板列表中出现该自定义模板；套用正常；支持从列表中删除。

## 4. 自测记录

- 平台环境：macOS (Darwin aarch64 - Apple Silicon M-series)
- 验证结果：
  1. `pnpm build`（vue-tsc + vite）构建成功，0 错误；
  2. `node scripts/test-templates.mjs` 运行并通过所有 3 组自动化单测（内置模板断言、自定义模板 CRUD、草稿持久化与清空）；
  3. `cargo check --locked` 与 `cargo test` 运行通过，11 项 Rust 单元测试保持 100% 通过；
  4. `git diff --check` 检查无错误。

