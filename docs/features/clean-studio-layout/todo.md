# 任务清单：极简创作工作台布局重构 (clean-studio-layout)

- 状态：merged
- Feature slug：clean-studio-layout
- 分支：feature/clean-studio-layout
- 负责人：Meme Studio Team
- 日期：2026-10-22

## 1. 任务拆解

- [x] **1. 需求调研与方案设计 (`docs/features/clean-studio-layout/`)**
  - [x] 梳理现有界面冗余与功能混排痛点，完成 `research.md` 与 `requirements.md`
  - [x] 设计三段式工作台与右侧四分栏属性面板架构，完成 `design.md`
- [x] **2. 重构前端视图与交互逻辑 (`src/App.vue`)**
  - [x] 引入右侧四分栏状态 `activeInspectorTab`（`canvas` / `text` / `media` / `export`）及折叠面板开关
  - [x] 实现画布拖拽（文字、贴纸、裁切框）与素材套用时的 Tab 智能联动
  - [x] 重构顶栏为一体化极简 Header（集成品牌、当前视频信息与更换入口、5 大抽屉入口、重置按钮与状态指示）
  - [x] 重构预览区底部为一体化播控与时间裁切栏（合并播放/静音按钮、双滑块时间轴与起止秒数内联输入框）
  - [x] 重构右侧控制面板为 4 个独立 Tab，并将低频文字变形/坐标输入收纳为折叠面板
  - [x] 优化底部导出状态栏，展示清晰的规格摘要芯片与主次操作按钮
- [x] **3. 重构与精简样式表 (`src/styles.css`)**
  - [x] 适配新三段式工作台网格与响应式布局
  - [x] 新增顶栏工具按钮、分段 Tab 导航、折叠面板与内联时间输入样式
- [x] **4. 自测与合并交付**
  - [x] 运行 `pnpm build`、`cargo check`、`cargo test` 与单元回归脚本
  - [x] 检查 `git diff --check`，更新文档状态并合并至 `main`
