# 实现计划与自测：现代小体积动图 WebP 格式支持

- 状态：proposed
- Feature slug: modern-formats
- 分支：feature/modern-formats
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 改动清单

| 模块 | 文件 | 变动说明 |
|------|------|----------|
| Rust / Tauri | `src-tauri/src/lib.rs` | 1. `RenderPlan` 增加 `format: Option<String>` 与 `is_webp` 方法<br>2. `validate_render_plan` 放宽支持 `.gif` 与 `.webp`<br>3. `build_filter_graph` 根据格式分支（WebP 免除调色板）<br>4. `build_ffmpeg_args` 增加 `-c:v libwebp -loop 0 -q:v <quality>`<br>5. 补充单测用例 |
| 前端 / API | `src/lib/desktop.ts` | 1. `RenderPlan` 增加 `format?: "gif" | "webp"`<br>2. 增加 `pickMediaOutput` 支持动态文件过滤器 |
| 前端 / 界面 | `src/App.vue` | 1. 增加 `exportFormat = ref<"gif" | "webp">("gif")`<br>2. 增加格式切换分段胶囊按钮<br>3. 路径自动同步后缀逻辑<br>4. 历史卡片与抽屉增加格式 Badge |
| 前端 / 样式 | `src/styles.css` | 格式切换胶囊、高亮状态与格式 Badge 样式 |
| 文档 | `docs/features/modern-formats/*` | 调研、需求、设计、实现与任务清单维护 |

## 2. 迁移与兼容策略

- **向后兼容**：
  - `format` 为可选字段（`Option<String>` / `?: "gif" | "webp"`），默认为 `gif`；
  - 旧历史记录和旧调用端未提供 `format` 时，根据 `output_path` 的扩展名安全推导，完全不破坏现有调用；
- **回滚方式**：
  若 WebP 发生意外兼容性问题，只需将 `exportFormat` 默认锁定为 `gif`，且 Rust 具备完整的回退分支。

## 3. 自测验证方案

### 3.1 自动化检查
```bash
# 1. 前端类型检查与打包
pnpm build

# 2. Rust 编译与死锁/借用检查
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked

# 3. Rust 单元测试（包含 WebP 滤镜图与参数构建测试）
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml

# 4. 提交物与空白符检查
git diff --check
```

### 3.2 实际转码验证 (Smoke Test)
使用真实测试视频运行转码，对比 GIF 与 WebP：
1. 导出同一视频片段为 GIF（High 质量），记录耗时与文件大小；
2. 导出同一视频片段为 WebP（High 质量），记录耗时与文件大小；
3. 验证 WebP 动图的循环播放、文字描边/底框清晰度、贴图透明通道与历史缩略图展示。

## 4. 自测记录

- 平台环境：macOS (Darwin aarch64 - Apple Silicon)
- 执行记录：待实现完成后填充
