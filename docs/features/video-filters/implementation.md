# 实现计划与自测：全局滤镜与画面调色 (video-filters)

- 状态：ready-to-merge
- Feature slug: video-filters
- 分支：feature/video-filters
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 改动清单

| 模块 | 文件 | 变动说明 |
|------|------|----------|
| Rust 后端 | `src-tauri/src/lib.rs` | 1. `RenderPlan` 新增 `video_filter: Option<String>` 字段<br>2. 新增 `map_video_filter` 映射内置滤镜表达式<br>3. 在 `build_filter_graph` 中于 scale 之后、drawtext 之前注入滤镜链<br>4. 新增 Rust 原生单元测试：`builds_filter_graph_with_video_filters` 验证各滤镜表达式拼接与前后层级隔离 |
| 桌面接口 | `src/lib/desktop.ts` | 扩展 `RenderPlan` 新增 `videoFilter?: string` |
| 模板/草稿 | `src/lib/templates.ts` | 扩展 `StudioDraft` 新增 `videoFilter?: string` |
| 队列核心 | `src/lib/queue.ts` | 确保 `createQueueItem` 完整捕获 `videoFilter` |
| 单元测试 | `scripts/test-queue.mjs` | 验证包含 `videoFilter` 的快照创建与传递 |
| 端到端 Smoke | `scripts/smoke-test-filters.mjs` | 真实调用 sidecar 转码黑白、反色、加速、倒放动图，验证各滤镜有效性 |
| 前端界面 | `src/App.vue` | 1. 控制面板第 02 节“输出预设”中增加“画风特效”选择网格<br>2. 画布背景层绑定实时 CSS filter 与倍速播放联动<br>3. 联动草稿存储、恢复与队列快照 |
| 前端样式 | `src/styles.css` | 画面特效选择器胶囊网格、动态高亮光晕与视口自适应样式 |
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

- 平台环境：macOS (Darwin aarch64 - Apple Silicon M-series)
- 执行记录：
  - `node scripts/smoke-test-filters.mjs`:
    - 黑白致郁 (`hue=s=0`): 真实生成 80KB GIF，画风转换成功
    - 反色负片 (`negate`): 真实生成 28KB WebP，鬼畜反相成功
    - 2x 鬼畜加速 (`setpts=0.5*PTS`): 真实生成 17KB GIF，魔性加速成功
    - 物理倒放 (`reverse`): 真实生成 5KB AVIF，倒放成功
  - `node scripts/test-queue.mjs`: 队列不可变快照对 `videoFilter` 字段捕获与恢复断言通过
  - `pnpm build`: vue-tsc 0 错误，vite 生产构建耗时 751ms
  - `cargo test`: 15 个原生 Rust 测试全部绿灯通过（通过率 100%）
  - `cargo check --locked`: 无报错通过
  - `git diff --check`: 零空白或格式错误
- 结论：各项技术指标与用户体验均达到合并标准。
