# 实现计划与自测：全局滤镜与画面调色 (video-filters)

- 状态：proposed
- Feature slug: video-filters
- 分支：feature/video-filters
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 改动清单

| 模块 | 文件 | 变动说明 |
|------|------|----------|
| Rust 后端 | `src-tauri/src/lib.rs` | 1. `RenderPlan` 新增 `video_filter: Option<String>` 字段<br>2. 新增 `map_video_filter` 映射内置滤镜表达式<br>3. 在 `build_filter_graph` 中于 scale 之后、drawtext 之前注入滤镜链<br>4. 新增 Rust 原生单元测试：验证各滤镜表达式拼接与前后层级隔离 |
| 桌面接口 | `src/lib/desktop.ts` | 扩展 `RenderPlan` 新增 `videoFilter?: string` |
| 模板/草稿 | `src/lib/templates.ts` | 扩展 `StudioDraft` 新增 `videoFilter?: string` |
| 队列核心 | `src/lib/queue.ts` | 确保 `createQueueItem` 完整捕获 `videoFilter` |
| 单元测试 | `scripts/test-queue.mjs` | 验证包含 `videoFilter` 的快照创建与传递 |
| 端到端 Smoke | `scripts/smoke-test-filters.mjs` | 真实调用 sidecar 转码黑白、反色、加速动图，验证各滤镜有效性 |
| 前端界面 | `src/App.vue` | 1. 控制面板第 01 节“画面与裁切”中增加“画风特效”选择栏<br>2. 画布 `<video>` 元素绑定实时 CSS filter 与倍速播放<br>3. 联动草稿存储、恢复与队列快照 |
| 前端样式 | `src/styles.css` | 画面特效选择器胶囊按钮组与高亮样式 |
| 文档 | `docs/features/video-filters/*` | 维护五件套文档与任务清单 |

## 2. 迁移与兼容策略

- 向后兼容：旧工程历史记录与草稿若无 `videoFilter` 字段，默认解析为 `"none"`；
- 前后隔离：视频特效只作用于视频画面，前景文字字幕和透明贴图完全不受影响。

## 3. 自测验证方案

### 3.1 自动化检查
```bash
# 1. 滤镜与队列纯逻辑测试
node scripts/test-queue.mjs

# 2. 滤镜端到端转码冒烟测试
node scripts/smoke-test-filters.mjs

# 3. 前端类型检查与打包
pnpm build

# 4. Rust 检查与单元测试
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml

# 5. 代码规范
git diff --check
```

## 4. 自测记录

- 平台环境：macOS (Darwin aarch64 - Apple Silicon)
- 执行记录：待实现完成后填充
