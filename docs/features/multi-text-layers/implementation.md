# 实现计划与自测：多文字图层支持

- 状态：ready-to-merge
- Feature slug: multi-text-layers
- 分支：feature/multi-text-layers
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 改动清单

| 模块 | 文件 | 变动说明 |
|------|------|----------|
| Rust / Tauri | `src-tauri/src/lib.rs` | 1. `RenderPlan` 增加 `text_layers: Option<Vec<TextOverlay>>` 与 `active_text_layers(&self)`<br>2. `validate_render_plan` 增加多图层遍历校验（上限 6，校验字符/字号/颜色）<br>3. `build_filter_graph` 链式拼接多个 `drawtext` 滤镜<br>4. 新增多图层单测 `builds_multi_text_layers_filter_graph` |
| 前端 / API | `src/lib/desktop.ts` | 1. `TextOverlay` 增加 `id?`, `visible?`<br>2. `RenderPlan` 增加 `textLayers?: TextOverlay[]` |
| 模板与草稿 | `src/lib/templates.ts` | 1. 导出 `CanvasTextLayer`<br>2. `StudioDraft` 支持 `textLayers` 与 `activeLayerId`<br>3. 向后兼容旧单文字草稿 |
| 测试脚本 | `scripts/test-templates.mjs` | 新增针对多图层草稿存储与恢复的断言测试 |
| 前端 / 界面 | `src/App.vue` | 1. 引入 `textLayers` 响应式列表与 `activeLayerId`<br>2. 控制面板实现图层标签栏（增删、显隐、切换）<br>3. 画布实现多文字遍历渲染、选中高亮与独立拖拽<br>4. 模板套用与草稿持久化/水合恢复兼容多图层 |
| 前端 / 样式 | `src/styles.css` | 图层标签栏（高度 23px）、显隐按钮、删除按钮、活跃图层高亮指示虚线框样式 |
| 文档 | `docs/features/multi-text-layers/*` | 调研、需求、设计、实现与任务清单维护 |

## 2. 迁移与兼容策略

- 向后兼容：旧调用方未传 `textLayers` 时自动使用原 `textOverlay`，反之亦然；
- 草稿升级：自动兼容旧草稿中未包含 `textLayers` 的情况，自动升级为首个图层。

## 3. 自测验证方案

### 3.1 自动化检查
```bash
# 1. 前端类型检查与打包
pnpm build

# 2. Rust 编译与单元测试
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml

# 3. 模板与草稿单元测试
node scripts/test-templates.mjs

# 4. 提交物与空白符检查
git diff --check
```

### 3.2 真实功能验收 (Smoke Test)
1. **多图层增删与切换**：
   - 添加“文字 2”与“文字 3”，分别设置不同文字、颜色、字号与描边；
   - 切换活跃图层，参数面板联动响应；
   - 删除“文字 3”，确认数量与状态正确。
2. **画布交互**：
   - 在画布中点击“文字 2”，确认高亮框切换到“文字 2”；
   - 拖拽“文字 2”至顶部，拖拽“文字 1”至底部，互不干扰。
3. **导出真实动图**：
   - 导出为 GIF 与 WebP，确认导出的文件中上下两段文字均清晰显示；
   - 隐藏“文字 2”，再次导出，确认输出文件中仅包含“文字 1”。

## 4. 自测记录

- 平台环境：macOS (Darwin aarch64 - Apple Silicon M-series)
- 执行记录：
  1. `pnpm build`: 通过，`vue-tsc` 类型检查无报错，Vite 打包耗时 1.08s；
  2. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`: 通过，锁文件无冲突；
  3. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml`: 12 个单测全量通过（含 `builds_multi_text_layers_filter_graph`）；
  4. `node scripts/test-templates.mjs`: 通过，多图层草稿持久化与恢复测试通过；
  5. 真实 FFmpeg 链式 drawtext 导出测试：
     - GIF: `scale=320:-1,fps=10,drawtext=顶部吐槽...,drawtext=底部反转...;split[gif][palette]...` 成功生成 `/tmp/test_multi_text.gif` (49KB, 14.4x 速度)；
     - WebP: `scale=320:-1,fps=10,drawtext=顶部吐槽...,drawtext=底部反转...` 成功生成 `/tmp/test_multi_text.webp` (58KB, 12.8x 速度)；
  6. `git diff --check`: 0 告警，无多余空白与换行错误。
