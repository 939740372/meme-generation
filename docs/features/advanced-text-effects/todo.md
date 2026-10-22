# 任务清单：高级文字排版效果 (advanced-text-effects)

- 状态：merged
- Feature slug: advanced-text-effects
- 分支：feature/advanced-text-effects
- 负责人：Antigravity
- 日期：2026-10-22

## 任务拆解与进度

### 阶段一：文档与技术基准

- [x] 完成 `docs/features/advanced-text-effects/research.md` 需求调研
- [x] 完成 `docs/features/advanced-text-effects/requirements.md` 需求说明
- [x] 完成 `docs/features/advanced-text-effects/design.md` 方案设计
- [x] 完成 `docs/features/advanced-text-effects/implementation.md` 实现计划
- [x] 完成 `docs/features/advanced-text-effects/todo.md` 任务清单
- [x] 提交规范 commit 建立 feature 基线

### 阶段二：Rust 后端滤镜算法实现

- [x] 在 `src-tauri/src/lib.rs` 中扩展 `TextOverlay`（新增 `rotation`, `arc_radius`, `is_italic`, `shadow_offset`, `shadow_color`）
- [x] 升级 `build_filter_graph`：
  - [x] 标准水平文字支持原生 `shadowcolor`, `shadowx`, `shadowy`
  - [x] 独立/多图层文字旋转支持透明画板 + `rotate` + 中心对齐 `overlay`
  - [x] 文字斜体支持 `shear=shx=-0.22:c=none`
  - [x] 弧形弯曲文字支持几何圆弧逐字分解与切向角定位
- [x] 编写 Rust 单元测试并验证全部跑通 (32/32 tests passed)

### 阶段三：前端数据模型与交互开发

- [x] 更新 `src/lib/desktop.ts` 中 `TextOverlay` 接口
- [x] 更新 `src/lib/templates.ts` 中 `CanvasTextLayer` 与 `StudioDraft`
- [x] 在 `src/App.vue` 图层控制面板中增加：
  - [x] 旋转角度滑块与快捷预设胶囊 (`0°`, `-45°`, `-15°`, `+15°`, `+45°`, `90°`)
  - [x] 弧形弯曲度滑块（-100 ~ 100，凹凸视觉标识）
  - [x] 斜体动感切换开关
  - [x] 阴影偏移距离滑块与阴影颜色拾色器
- [x] 在 `src/App.vue` 画布预览中增加旋转、弯曲、斜体与阴影渲染逻辑
- [x] 在 `src/styles.css` 中添加高级排版控件视觉样式

### 阶段四：验证、自测与质量门槛

- [x] 编写端到端冒烟测试脚本 `scripts/smoke-test-advanced-text.mjs`
- [x] 运行冒烟测试，通过真实 FFmpeg 导出验证旋转、弧度、斜体与阴影
- [x] 执行前端类型检查与打包：`pnpm build`
- [x] 执行 Rust 测试与检查：`cargo test` 与 `cargo check --locked`
- [x] 执行格式与代码差异检查：`git diff --check`
- [x] 更新 `implementation.md` 自测记录与文档状态为 `ready-to-merge`

### 阶段五：合并与主分支同步

- [x] 非快进合并至 `main` 分支
- [x] 更新根目录 `TODO.md`
- [x] 推送 `main` 至远程 `origin main`
- [x] 清理本地分支 `feature/advanced-text-effects`
