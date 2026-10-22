# 任务拆解：文字描边与字幕底色条增强

- 状态：ready-to-merge
- Feature slug: text-styling-stroke
- 分支：feature/text-styling-stroke
- 负责人：待定
- 日期：2026-10-22

## 阶段一：文档与分支准备
- [x] 创建 feature 分支 `feature/text-styling-stroke`
- [x] 完成 `docs/features/text-styling-stroke/` 五件套文档初稿
- [x] 提交初始文档，遵循常规提交规范

## 阶段二：Rust 数据模型与 FFmpeg 滤镜扩展 (lib.rs)
- [x] 在 `src-tauri/src/lib.rs` 的 `TextOverlay` 增加 `stroke_width`, `stroke_color`, `use_box`, `box_color`, `box_opacity`
- [x] 改进 `color_for_filter` 支持 Hex 与透明度转换
- [x] 更新 `build_filter_graph` 构建 `borderw`, `bordercolor`, `box`, `boxcolor` 动态参数
- [x] 编写并补充 Rust 单元测试验证描边与底框构建正确性
- [x] 运行 `cargo test` 与 `cargo check` 验证通过

## 阶段三：前端类型定义与状态管理 (desktop.ts & App.vue)
- [x] 在 `src/lib/desktop.ts` 扩展 `TextOverlay` 接口定义
- [x] 在 `src/App.vue` 增加文字样式响应式状态（`textStrokeWidth`, `textStrokeColor`, `textUseBox`, `textBoxColor`, `textBoxOpacity`）
- [x] 更新 `buildRenderPlan` 打包文字样式参数
- [x] 更新 `loadHistoryPlan` 支持回填文字描边与底框配置

## 阶段四：界面与画布所见即所得渲染 (App.vue & styles.css)
- [x] 在中央预览舞台 `.preview-text` 增加 `-webkit-text-stroke`、`paint-order` 与半透明 `background` 渲染
- [x] 在控制面板文字区域增加描边宽度选择器、描边色盘、底框开关、底色盘与透明度滑块
- [x] 优化排版与样式，严格守住 1280×820 窗口无多余滚动条

## 阶段五：自测、质量门槛与合并
- [x] 执行 `pnpm build`（vue-tsc + vite build）
- [x] 执行 `cargo check --locked` 与 `cargo test`
- [x] 执行 `git diff --check`
- [x] 记录自测结果至 `implementation.md`
- [x] 更新 feature 文档状态为 `ready-to-merge`
- [ ] 切回 `main` 分支执行非快进合并：`git merge --no-ff feature/text-styling-stroke`
- [ ] 更新 `TODO.md` 并标记 feature 文档为 `merged`
- [ ] 推送远程仓库
