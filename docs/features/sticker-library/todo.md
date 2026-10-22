# 任务拆解：内置透明表情贴纸库 (sticker-library)

- 状态：proposed
- Feature slug: sticker-library
- 分支：feature/sticker-library
- 负责人：Antigravity
- 日期：2026-10-22

## 任务执行清单

- [ ] 0. 文档初始化与建立基线
  - [x] 创建 `feature/sticker-library` 分支
  - [x] 撰写并在 `docs/features/sticker-library/` 提交完整调研、需求、设计与实现计划
- [ ] 1. 贴纸素材制作与生成脚本 (`src-tauri/stickers/`, `scripts/generate-stickers.mjs`)
  - [ ] 1.1 编写 `generate-stickers.mjs`，绘制 18 款经典高质量透明矢量贴纸
  - [ ] 1.2 批量生成 18 张 256×256 标准透明 PNG 贴纸至 `src-tauri/stickers/`
  - [ ] 1.3 编写 `scripts/test-stickers.mjs` 校验贴纸文件与透明通道
- [ ] 2. Tauri 打包配置与 Rust 接口 (`src-tauri/tauri.conf.json`, `src-tauri/src/lib.rs`)
  - [ ] 2.1 `tauri.conf.json` 注册 `"stickers/*"` 至 `bundle.resources`
  - [ ] 2.2 定义 `BuiltinSticker` 结构体并实现 `resolve_stickers_dir` 跨环境解析
  - [ ] 2.3 实现 `get_builtin_stickers` Tauri command 并注册至 invoke_handler
  - [ ] 2.4 编写针对贴纸解析与元数据的 Rust 单元测试
- [ ] 3. 前端接口与贴纸库抽屉 (`src/lib/desktop.ts`, `src/App.vue`)
  - [ ] 3.1 `desktop.ts` 导出 `BuiltinSticker` 与 `getBuiltinStickers()`
  - [ ] 3.2 抽屉集成“贴纸素材 (🎨)”标签页，实现分类筛选与贴纸网格卡片
  - [ ] 3.3 侧边栏“03 叠加”增加“🎨 贴纸素材库”快捷入口与预设缩放比例
- [ ] 4. 画布真图预览与缩放交互 (`src/App.vue`, `src/styles.css`)
  - [ ] 4.1 画布贴纸容器升级为渲染真图 `<img>`
  - [ ] 4.2 贴纸外框右下角实现缩放手柄与 `startResizeImage` 交互
  - [ ] 4.3 编写贴纸库暗色棋盘格网格与手柄样式，维持 1280x820 零额外滚动条
- [ ] 5. 严格验证与质量门槛
  - [ ] 5.1 运行 `node scripts/test-stickers.mjs` 通过
  - [ ] 5.2 运行 `pnpm build`（vue-tsc + vite）检查通过
  - [ ] 5.3 运行 `cargo check --locked` 与 `cargo test` 全绿通过
  - [ ] 5.4 执行真实 FFmpeg 贴纸动图合成 Smoke Test（GIF 与 WebP）
  - [ ] 5.5 检查 `git diff --check`
- [ ] 6. 收尾与主分支合入
  - [ ] 6.1 更新五件套文档状态为 `ready-to-merge` 并记录自测结果
  - [ ] 6.2 非快进合并至 `main`
  - [ ] 6.3 文档状态变更为 `merged`，同步根目录 `TODO.md`，推送到远程仓库
