# 任务拆解：常用表情包模板与草稿暂存

- 状态：ready-to-merge
- Feature slug: templates-and-drafts
- 分支：feature/templates-and-drafts
- 负责人：Antigravity
- 日期：2026-10-22

## 任务执行清单

- [x] 0. 文档初始化与建立基线
  - [x] 创建 `feature/templates-and-drafts` 分支
  - [x] 撰写并在 `docs/features/templates-and-drafts/` 提交完整需求、设计与实现计划文档
- [x] 1. 模板体系与草稿核心服务构建 (`src/lib/templates.ts`, `scripts/test-templates.mjs`)
  - [x] 1.1 定义 `StudioDraft` 与 `MemeTemplate` 类型接口
  - [x] 1.2 内置 3 款经典表情包排版（影视字幕条、顶部吐槽带、名场面大字报）
  - [x] 1.3 实现自定义模板的本地持久化、读取与删除工具方法
  - [x] 1.4 编写并跑通模板与草稿自动化单元测试
- [x] 2. 自动草稿暂存与启动恢复 (`src/App.vue`)
  - [x] 2.1 监听工作区状态，实现防抖 400ms 的草稿实时自动暂存
  - [x] 2.2 应用加载时自动检测并恢复草稿（包含视频 probe 与容错）
  - [x] 2.3 增加工作区“重置/清空草稿”动作
- [x] 3. 模板库管理与交互集成 (`src/App.vue`, `src/styles.css`)
  - [x] 3.1 抽屉面板升级为双 Tab：`导出历史` 与 `排版模板`
  - [x] 3.2 实现模板卡片列表渲染与“一键套用”动作
  - [x] 3.3 文字控制区增加“存为模板”弹窗交互与快捷套用入口
  - [x] 3.4 编写精致紧凑的样式，确保 1280x820 视口无溢出
- [x] 4. 严格验证与质量门槛
  - [x] 4.1 运行 `pnpm build`（vue-tsc + vite）检查通过
  - [x] 4.2 运行 `cargo check --locked` 与 `cargo test`
  - [x] 4.3 执行草稿恢复、模板套用与自定义模板增删 Smoke Test
  - [x] 4.4 检查 `git diff --check`
- [x] 5. 收尾与主分支合入
  - [x] 5.1 更新五件套文档状态为 `ready-to-merge` 并记录自测结果
  - [ ] 5.2 非快进合并至 `main`
  - [ ] 5.3 文档状态变更为 `merged`，同步根目录 `TODO.md`，推送到远程仓库
