# 详细实现文档：导出预设与压缩质量控制

- 状态：implementing
- Feature slug: export-presets
- 分支：feature/export-presets
- 负责人：待定
- 日期：2026-10-22

## 1. 文件和模块计划

| 路径 | 变更 | 目的 |
| --- | --- | --- |
| `src/lib/desktop.ts` | `RenderPlan` 类型增加 `quality?: "low" \| "medium" \| "high"` | 拓展前端调用类型契约 |
| `src-tauri/src/lib.rs` | `RenderPlan` 结构体增加 `quality: Option<String>`，并在 `build_filter_graph` 中匹配 `dither` | 后端调色板抖动算法拓展与兼容 |
| `src/App.vue` | 增加预设列表数据、切换函数、手动修改监听与分段选择器 UI | 提供直观易懂的一键场景预设交互 |
| `src/styles.css` | 增加 `.preset-chips` / `.preset-pill` 样式并精调控制面板间距 | 保证 1280×820 固定视口无滚动条 |
| `docs/features/export-presets/` | 维护全套 feature 文档 | 记录设计、实现与自测 |

## 2. 预设参数表

| 预设 ID | 预设名称 | 宽度 | 帧率 | 质量 (Quality / Dither) | 目标体积 | 典型用途 |
| --- | --- | --- | --- | --- | --- | --- |
| `wechat-small` | 微信表情 | 240px | 10 FPS | `low` (`bayer:bayer_scale=3`) | < 1MB | 微信聊天窗口、群表情 |
| `wechat-large` | 微信大表情 | 320px | 12 FPS | `medium` (`sierra2_4a`) | < 5MB | 自定义大表情、贴纸 |
| `hd-social` | 高清动图 | 480px | 15 FPS | `high` (`sierra2_4a`) | 5MB~10MB | Discord、Twitter、微博高清分享 |
| `lite` | 极速轻量 | 180px | 8 FPS | `low` (`bayer:bayer_scale=5`) | < 500KB | 论坛头像、快速预览、低网速场景 |
| `custom` | 自定义 | 用户自定 | 用户自定 | 用户自定 | 自定 | 精细化调整 |

## 3. 测试方案

1. 语法与类型检查：`pnpm build`（`vue-tsc` 与 `vite build`）；
2. Rust 编译与单元测试：`cargo check --locked` 与 `cargo test`；
3. 真实媒体体积对比验证：
   - 使用相同的 3 秒短视频素材；
   - 场景 A：使用 `wechat-small` 预设导出，验证体积是否 < 1MB；
   - 场景 B：使用 `hd-social` 预设导出，验证画质和体积差异；
4. 视口检查：验证 1280×820 下整个工作区无外层滚动条。

## 4. 自测记录

*(待实现后更新)*
