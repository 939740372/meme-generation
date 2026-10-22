# 任务拆解：现代小体积动图 WebP 格式支持

- 状态：proposed
- Feature slug: modern-formats
- 分支：feature/modern-formats
- 负责人：Antigravity
- 日期：2026-10-22

## 任务执行清单

- [ ] 0. 文档初始化与建立基线
  - [x] 创建 `feature/modern-formats` 分支
  - [x] 撰写并在 `docs/features/modern-formats/` 提交完整需求、设计与实现计划文档
- [ ] 1. Rust 核心能力扩展 (`src-tauri/src/lib.rs`)
  - [ ] 1.1 `RenderPlan` 支持 `format: Option<String>`，增加 `is_webp(&self) -> bool`
  - [ ] 1.2 `validate_render_plan` 适配 `.gif` 和 `.webp` 双扩展名校验
  - [ ] 1.3 `build_filter_graph` 重构：WebP 格式直接输出至 `[vout]`，跳过调色板
  - [ ] 1.4 `build_ffmpeg_args` 针对 WebP 注入 `-c:v libwebp -loop 0 -q:v <quality>`，移除 `-f gif`
  - [ ] 1.5 编写针对 WebP 滤镜图和 FFmpeg 参数构建的单元测试并执行通过
- [ ] 2. 前端接口与交互实现 (`src/lib/desktop.ts`, `src/App.vue`, `src/styles.css`)
  - [ ] 2.1 `desktop.ts` 扩展 `RenderPlan` 类型并增加 `pickMediaOutput`
  - [ ] 2.2 `App.vue` 引入 `exportFormat` 响应式状态与格式切换药丸控件
  - [ ] 2.3 实现格式切换时路径后缀自动同步（`.gif` <-> `.webp`）与保存对话框联动
  - [ ] 2.4 侧边栏最近卡片与历史抽屉新增 `GIF` / `WEBP` 格式徽章显示
  - [ ] 2.5 历史记录参数载入时恢复对应格式选择
  - [ ] 2.6 添加紧凑精致的样式，保证 1280x820 视口绝无溢出
- [ ] 3. 严格验证与质量门槛
  - [ ] 3.1 运行 `pnpm build`（vue-tsc + vite 构建检查）
  - [ ] 3.2 运行 `cargo check --locked` 与 `cargo test`
  - [ ] 3.3 运行真实 Smoke Test 测试 WebP 导出产物大小与画质
  - [ ] 3.4 检查 `git diff --check`
- [ ] 4. 收尾与主分支合入
  - [ ] 4.1 更新五件套文档状态为 `ready-to-merge` 并记录自测结果
  - [ ] 4.2 非快进合并至 `main`
  - [ ] 4.3 文档状态变更为 `merged`，同步根目录 `TODO.md`，推送到远程仓库
