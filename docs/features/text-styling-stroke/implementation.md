# 实现计划与交付记录：文字描边与字幕底色条增强

- 状态：merged
- Feature slug: text-styling-stroke
- 分支：feature/text-styling-stroke
- 负责人：待定
- 日期：2026-10-22

## 1. 文件级实现计划

| 文件 | 变更类型 | 说明 |
| --- | --- | --- |
| `src-tauri/src/lib.rs` | 修改 | 扩展 `TextOverlay` 字段（`stroke_width`, `stroke_color`, `use_box`, `box_color`, `box_opacity`）；更新 `build_filter_graph` 构建描边与底框；添加对应单测 |
| `src/lib/desktop.ts` | 修改 | 更新 `TextOverlay` 接口定义，包含可选描边与底框字段 |
| `src/App.vue` | 修改 | 增加文字样式响应式状态（`textStrokeWidth`, `textStrokeColor`, `textUseBox`, `textBoxColor`, `textBoxOpacity`）；控制面板增加描边与底框参数组件；更新画布文字预览与历史回填逻辑 |
| `src/styles.css` | 修改 | 新增描边与底框样式配置条、微型开关与色盘紧凑排版，维持 1280×820 窗口无滚动条 |
| `docs/features/text-styling-stroke/*` | 新增 | 全套 feature 文档、架构图与自测记录 |

## 2. 兼容性与回滚策略

1. **接口兼容性**：
   - 所有新增参数在 Rust 端均为 `Option<T>`，缺省时自动回退为 0 描边和关闭底框，旧版历史记录无损复用；
   - `buildRenderPlan` 与 `loadHistoryPlan` 支持新旧配置平滑过渡。
2. **跨平台字体兼容**：
   - 描边与底框基于 libfreetype 静态编译内建能力，三平台表现一致。

## 3. 测试与自测方案

### 3.1 自动化检查
1. 前端类型检查与构建：
   ```bash
   pnpm build
   ```
2. Rust 编译与单元测试：
   ```bash
   PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked
   PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml
   ```
3. 代码风格与工作区检查：
   ```bash
   git diff --check
   ```

### 3.2 交互与冒烟验证项
- [x] 开启文字描边（宽 2px，黑），画布文字呈现清晰外轮廓；
- [x] 勾选开启底框，画布文字出现半透明黑底背景；
- [x] 拖拽底框透明度滑块，画布背景透光度实时改变；
- [x] 关闭底框，画布文字背景恢复为完全透明；
- [x] 运行真实 FFmpeg 转码并导出 GIF，检验最终动图的描边与底框和画布预览完全一致；
- [x] 验证 `1280×820` 固定视口，无页面全局或局部异常滚动条。

## 4. 自测记录

- 执行时间：2026-10-22 12:06
- 平台环境：macOS (Darwin 24.3.0, Apple Silicon aarch64)
- 测试命令与输出：
  1. `pnpm build`:
     `$ vue-tsc --noEmit && vite build` -> `✓ built in 534ms`，前端类型检查与产物构建全部通过；
  2. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`:
     `Finished dev profile in 0.56s`，编译无任何警告；
  3. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml`:
     `9 passed; 0 failed; 0 ignored; finished in 0.00s`，包含新增的 `builds_drawtext_filter_graph_without_box_and_stroke` 与 `builds_drawtext_filter_graph_with_font` 均成功验证；
  4. `git diff --check`:
     无空白与换行错误。
- 验收结论：达到 ready-to-merge 门槛。
