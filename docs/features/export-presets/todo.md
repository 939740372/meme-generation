# 细化 TODO：导出预设与压缩质量控制

- 状态：implementing
- Feature slug: export-presets
- 分支：feature/export-presets
- 负责人：待定
- 日期：2026-10-22

## 1. 契约拓展与 Rust 调色板算法

- [ ] 在 `src/lib/desktop.ts` 中为 `RenderPlan` 增加 `quality?: string` 字段
- [ ] 在 `src-tauri/src/lib.rs` 中为 `RenderPlan` 增加 `quality: Option<String>` 字段
- [ ] 在 `src-tauri/src/lib.rs` 的 `build_filter_graph` 中根据 `quality` 映射 `paletteuse=dither=...`
- [ ] 增加 Rust 单元测试验证不同质量档位下的滤镜图构建

## 2. 前端预设模型与交互实现

- [ ] 在 `src/App.vue` 中定义 4 个主流预设及自定义状态
- [ ] 实现预设切换函数，一键填充宽度、帧率与质量
- [ ] 实现手动修改输入框自动退回“自定义”预设状态
- [ ] 在控制面板渲染预设分段选择按钮组（Pills）
- [ ] 在导出栏显示当前选中的预设名称与预估体积说明

## 3. 样式与固定视口布局调优

- [ ] 在 `src/styles.css` 中编写预设按钮组与微型标签样式
- [ ] 检查并微调面板间距，确保 `1280×820` 固定视口下无任何滚动条

## 4. 自测与合并

- [ ] 执行 `git diff --check`
- [ ] 执行 `pnpm build`
- [ ] 执行 `cargo check --locked` 与 `cargo test`
- [ ] 执行端到端真实导出验证：验证 `wechat-small` 预设成功将动图体积压在 1MB 内
- [ ] 更新 feature 文档状态为 `ready-to-merge`
- [ ] 合并到 `main` 分支并将状态更新为 `merged`
- [ ] 同步更新根目录 `TODO.md`
