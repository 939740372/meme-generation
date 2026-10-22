# 实现计划与自测：AVIF 动图格式支持 (avif-format)

- 状态：merged
- Feature slug: avif-format
- 分支：feature/avif-format
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 改动清单

| 模块 | 文件 | 变动说明 |
|------|------|----------|
| Rust 后端 | `src-tauri/src/lib.rs` | 1. 扩展 `RenderPlan` 支持 `is_avif()`<br>2. 校验 `.avif` 文件扩展名及格式匹配<br>3. 在滤镜图中对 AVIF 追加 `,format=yuv420p`<br>4. 构建 `libsvtav1` SVT-AV1 编码参数 (`-preset 8`, `-crf`, `-pix_fmt yuv420p`)<br>5. 在 `read_file_as_data_url` 中支持 `image/avif`<br>6. 增加 AVIF 滤镜与参数构建原生单元测试 `builds_avif_filter_graph_and_args` |
| 桌面接口 | `src/lib/desktop.ts` | 1. 扩展 `exportFormat` 为 `"gif" \| "webp" \| "avif"`<br>2. 文件保存对话框过滤器增加 `.avif` 选项 |
| 队列核心 | `src/lib/queue.ts` | 扩展 `format` 支持 `"avif"`，并在 `generateQueueOutputPath` 与 `createQueueItem` 中支持 `.avif` |
| 模板类型 | `src/lib/templates.ts` | 扩展 `StudioDraft` 与 `MemeTemplate` 的 format 支持 `"avif"` |
| 单元测试 | `scripts/test-queue.mjs` | 增加 AVIF 格式队列项防冲突生成与序列化测试用例 |
| 端到端 Smoke | `scripts/smoke-test-avif.mjs` | 调用真实 sidecar 执行一段 AVIF 动图导出，验证其存在性、体积与文件头标识 |
| 前端界面 | `src/App.vue` | 1. 更新 `exportFormat` 状态与三段式胶囊切换按钮<br>2. 更新按钮文案 `导出 AVIF ↗`<br>3. 恢复历史与草稿支持识别恢复 `avif` |
| 前端样式 | `src/styles.css` | 增加 `.format-badge.avif` 和 `.format-pill.pill-avif` 紫色徽标样式 |
| 文档 | `docs/features/avif-format/*` | 维护五件套文档与任务清单 |

## 2. 迁移与兼容策略

- 向后兼容：旧历史记录与草稿若无 format 字段，依扩展名推断，默认回退为 `gif`；
- 无缝共存：同一个批量队列中可以自由混合 GIF、WebP 与 AVIF 三种任务，依序转码。

## 3. 自测验证方案

### 3.1 自动化检查
```bash
# 1. 队列纯逻辑测试
node scripts/test-queue.mjs

# 2. AVIF 端到端冒烟测试
node scripts/smoke-test-avif.mjs

# 3. 前端类型检查与打包
pnpm build

# 4. Rust 检查与单元测试
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml

# 5. 代码规范
git diff --check
```

## 4. 自测记录

- 平台环境：macOS (Darwin aarch64 - Apple Silicon M-series)
- 执行记录：
  - `node scripts/smoke-test-avif.mjs`:
    - GIF 大小: **110.4 KB**
    - WebP 大小: **68.0 KB**
    - AVIF 大小: **9.0 KB** (比 GIF 减少 **91.8%**，比 WebP 减少 **86.8%**)
    - 转码速度: **24.6x 极速**
    - 文件头标识: 确认包含 ISO Media `ftyp` box 与 `avif` brand。
  - `node scripts/test-queue.mjs`: 队列防冲突唯一路径生成断言通过（GIF, WebP, AVIF）。
  - `pnpm build`: vue-tsc 0 错误，vite 生产构建耗时 654ms。
  - `cargo test`: 14 个原生测试全部通过（通过率 100%）。
  - `cargo check --locked`: 无报错通过。
  - `git diff --check`: 零空白或格式错误。
- 结论：各项技术指标与用户体验均达到合并标准。
