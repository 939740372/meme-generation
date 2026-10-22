# 任务拆解：最近导出历史与快捷操作

- 状态：implementing
- Feature slug: export-history
- 分支：feature/export-history
- 负责人：待定
- 日期：2026-10-22

## 阶段一：文档与分支准备
- [x] 创建 feature 分支 `feature/export-history`
- [x] 完成 `docs/features/export-history/` 五件套文档初稿
- [x] 提交初始文档，遵循常规提交规范

## 阶段二：Rust 原生命令实现与测试
- [ ] 在 `src-tauri/Cargo.toml` 引入 `base64 = "0.22"`
- [ ] 在 `src-tauri/src/lib.rs` 实现 `copy_file_to_clipboard(path: String) -> CommandResult<()>`
- [ ] 在 `src-tauri/src/lib.rs` 实现 `read_file_as_data_url(path: String) -> CommandResult<String>`
- [ ] 注册新 command 至 `tauri::generate_handler!`
- [ ] 编写 Rust 单元测试验证功能与错误处理
- [ ] 执行 `cargo test` 与 `cargo check` 验证通过

## 阶段三：前端类型定义与 IPC 封装
- [ ] 在 `src/lib/desktop.ts` 定义 `ExportRecord` 数据结构
- [ ] 在 `src/lib/desktop.ts` 导出 `revealItemInDir`（封装自 `@tauri-apps/plugin-opener`）
- [ ] 在 `src/lib/desktop.ts` 封装 `copyFileToClipboard`（带文本路径回退）
- [ ] 在 `src/lib/desktop.ts` 封装 `readFileAsDataUrl` 并处理异常

## 阶段四：界面与交互实现 (App.vue & styles.css)
- [ ] 实现 `historyList` 响应式状态管理及 `localStorage` 持久化逻辑（上限 10 条）
- [ ] 监听 `render_completed`，导出成功后自动构建记录并追加至历史队列
- [ ] 导出成功栏增加「在访达中显示」、「复制文件」与「历史 (N)」快捷按钮
- [ ] 在侧边栏新增「最近导出」常驻卡片，展示最新表情包与微型快捷动作
- [ ] 实现底部滑出式历史抽屉面板，支持缩略图流式浏览、单条删除与全部清空
- [ ] 实现「载入此历史参数」逻辑，完整回填至视频、时间轴、裁剪、预设与图层
- [ ] 实现轻量 Toast 浮层提示组件，提供即时状态反馈
- [ ] 调整样式与布局，严格守住 1280×820 固定视口，无多余滚动条

## 阶段五：自测、质量门槛与合并
- [ ] 执行 `pnpm build`（vue-tsc + vite build）
- [ ] 执行 `cargo check` 与 `cargo test`
- [ ] 执行 `git diff --check`
- [ ] 记录自测结果至 `implementation.md`
- [ ] 更新 feature 文档状态为 `ready-to-merge`
- [ ] 切回 `main` 分支执行非快进合并：`git merge --no-ff feature/export-history`
- [ ] 更新 `TODO.md` 并标记 feature 文档为 `merged`
- [ ] 推送远程仓库
