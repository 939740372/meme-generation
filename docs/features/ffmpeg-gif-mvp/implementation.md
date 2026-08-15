# 详细实现文档：FFmpeg GIF MVP

- 状态：verifying
- Feature slug：ffmpeg-gif-mvp
- 分支：feature/ffmpeg-gif-mvp
- 负责人：待定
- 日期：2026-08-14

## 1. 文件和模块计划

| 路径 | 变更 | 目的 |
| --- | --- | --- |
| src/App.vue | 重写为单页 GIF 编辑器 | 导入、参数编辑、导出状态和结果展示 |
| src/styles.css | 增加 MVP 工作区样式 | 形成可操作的桌面编辑界面 |
| src/lib/desktop.ts | 增加媒体类型、Tauri command/event 封装 | 保持前端 API 可序列化并支持 web preview |
| src-tauri/src/lib.rs | 增加探测、RenderPlan 校验、FFmpeg runner 和取消 | 管理本机 FFmpeg 进程和事件 |
| src-tauri/Cargo.toml | 增加 `tauri-plugin-dialog` | 通过系统对话框取得真实文件路径 |
| src-tauri/capabilities/default.json | 增加 dialog 权限 | 允许最小的打开/保存对话框能力 |
| src-tauri/tauri.conf.json | 保持当前窗口/构建配置 | 本轮不宣称 sidecar 打包 |
| package.json / pnpm-lock.yaml | 增加 dialog JS 包 | 与 Rust dialog plugin 对接 |
| docs/features/ffmpeg-gif-mvp/ | 持续更新五份文档 | 记录需求、设计、实现和验证 |

本轮 UI refinement 追加：

| 路径 | 变更 | 目的 |
| --- | --- | --- |
| src/App.vue | 保留媒体交互模型，配合桌面工作区语义 | 不改变导入、参数、渲染和事件逻辑 |
| src/styles.css | 重写为固定视口的深色桌面工作台样式 | 消除网页式纵向扩张和滚动条，强化预览/控制/导出层级 |
| src-tauri/tauri.conf.json | 锁定窗口最小/最大尺寸并关闭缩放 | 保证客户端长宽高固定 |

### 新增依赖说明

- `@tauri-apps/plugin-dialog` / `tauri-plugin-dialog` `2.7.2`：调用系统打开/保存对话框，许可证为 MIT OR Apache-2.0；会增加少量前端包和原生 Rust 编译时间，不把文件内容上传到网络。
- `tauri-plugin-dialog` 的 Rust 依赖链包含 `tauri-plugin-fs` `2.5.1` 和 `rfd` `0.16.0`（均 Apache-2.0 OR MIT），用于跨平台原生对话框实现；会引入各平台窗口/文件对话框适配代码，Windows/Linux 打包仍需分别验证。
- 本轮没有新增 FFmpeg、字体或 PNG 运行时资源；FFmpeg 由开发机 PATH/环境变量提供，因此没有二进制体积或许可证分发承诺。

## 2. 实现步骤

1. 定义 RenderPlan、数值校验和安全路径策略。
2. 接入 `ffprobe`，完成媒体信息解析和基础错误映射。
3. 实现 trim、crop、scale、fps、drawtext 和 PNG overlay 参数构建。
4. 实现 palettegen/paletteuse、进度解析、取消、错误分类和输出验证。
5. 接入 Vue 编辑界面、桌面文件选择、任务状态和错误展示。
6. 用临时目录生成短视频/PNG 样例完成真实导出冒烟。
7. 记录本机 FFmpeg 版本、许可证边界、构建验证和未覆盖平台。

## 3. 迁移和兼容

当前没有既有媒体项目或输出配置需要迁移。RenderPlan 若进入持久化，必须先定义版本字段和迁移策略；MVP 阶段只作为单次任务输入，不把本地媒体复制进仓库。

## 4. 测试计划

| 场景 | 类型 | 操作/命令 | 预期结果 |
| --- | --- | --- | --- |
| 正常短视频 | 功能 | 导入并导出默认 GIF | 输出可打开、可播放 |
| 参数变更 | 功能 | 修改时间、裁切、宽度、帧率 | 输出属性和画面变化符合设置 |
| 文字/图案 | 功能 | 加入 drawtext 和 PNG overlay | 输出画面包含叠加内容 |
| 取消 | 边界 | 导出中点击取消 | 子进程退出、临时文件清理、可重新开始 |
| 无效输入 | 错误 | 使用损坏或不支持文件 | 显示失败，不生成假成功 |
| sidecar 缺失 | 错误 | 暂时移除或改名可执行文件 | 提示安装/打包问题 |
| 输出不可写 | 错误 | 选择无写权限目录 | 显示可操作错误 |
| 产物检查 | 回归 | 检查输出尺寸、帧率、时长和可打开性 | 与 RenderPlan 一致 |

## 5. 自测记录

- 环境：macOS arm64；Rust/Cargo 1.94.1；FFmpeg/FFprobe 9.0（Homebrew `/opt/homebrew/bin`）
- 日期：2026-08-14
- `git diff --check`：通过
- `CI=true pnpm install --store-dir /Volumes/M2SSD/.pnpm-store` / `CI=true pnpm build`：通过；构建包含 `vue-tsc --noEmit` 和 Vite production build
- `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`：通过
- `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml --locked`：通过，3 个 Rust 单测通过
- 桌面冒烟：`pnpm tauri:dev` 在补充 Cargo PATH 并获得端口权限后启动 Vite、编译并运行 Tauri binary；Computer Use 未发现可访问的开发窗口，因此文件选择和 UI 交互未完成验证
- 未覆盖平台：Windows/Linux sidecar 与打包未验证；macOS 安装包、签名和发布流程未执行

### 真实媒体冒烟

测试资产写入 `/private/tmp/meme-gif-mvp/`，没有进入仓库：

| 场景 | 结果 |
| --- | --- |
| `lavfi testsrc2` 生成 640×360、30 FPS、2 秒 MP4 | 通过；`ffprobe` 读取到 H.264、2.000 秒 |
| 0.2–1.4 秒、裁切 480×270、缩放 320 宽、12 FPS、palettegen/paletteuse | 通过；GIF 可被 `ffprobe` 读取，320×180、1.160 秒、150,568 bytes |
| 同一管线叠加 PNG 图案 | 通过；GIF 可被 `ffprobe` 读取，320×180、1.160 秒、125,646 bytes |
| 文字 `drawtext` | 本机 FFmpeg 9.0 不包含该 filter；已加入渲染前能力检查并返回 `DRAW_TEXT_UNAVAILABLE`，需要带 libfreetype 的构建 |

### UI refinement 验证

- `CI=true pnpm build`：通过；Vue 类型检查和 Vite production build 均通过。
- `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`：通过。
- 浏览器固定视口检查：使用本地 Vite 和 `1280 × 820` 视口验证，默认状态以及同时展开裁切/文字字段时，`document`、`body`、`#app` 均为 `1280 × 820`，页面没有水平/垂直外层滚动；右侧控制面板 `scrollWidth/scrollHeight` 与 client 尺寸一致。
- `CI=true PATH=/Users/ryantorres/.cargo/bin:$PATH pnpm tauri:dev`：通过启动 Vite、编译 Rust 并运行 `target/debug/meme-studio`；Tauri 接受固定窗口配置。当前环境未能通过桌面窗口观察工具读取原生窗口像素，视觉布局以同尺寸浏览器渲染结果复核。
- 视觉结果：深色石墨工作台、酸性黄主操作色、左侧导航、中央预览、右侧参数和底部导出状态均在同一固定视口内可见；现有媒体交互 DOM 和 script setup 未改动。
- 字体可读性 refinement：导航、标题、正文、参数输入和状态信息整体放大 1–2px；同步压缩面板留白和控件高度，避免放大后破坏固定视口布局。
- 放大字体后的固定视口复核：默认状态以及裁切/文字展开状态的 `document`、`body` 均为 `1280 × 820`，控制面板 `scrollHeight === clientHeight`，无可见横向/纵向滚动条。

验证环境：macOS arm64；Rust/Cargo 1.94.1；Node/pnpm；2026-08-15。

## 6. 已知限制

- 当前实现使用本机 PATH 中的 FFmpeg，不包含可分发 sidecar。
- 浏览器预览模式可以查看界面，但不能替代 Tauri 的真实文件选择和进程调用。
- 当前 Homebrew FFmpeg 没有 `drawtext`，文字叠加代码路径已实现但本机无法完成文字输出验收。
- 没有通过 Computer Use 完成文件选择、取消和成功提示的实际桌面交互；这部分仍需在可访问的 Tauri 窗口中补测。
- 本轮通过同尺寸浏览器渲染验证固定布局；原生 Tauri 窗口的实际像素截图和拖拽禁用状态未由桌面观察工具独立读取，但配置和开发启动均已通过。
- GIF 的颜色、尺寸和文件大小之间需要后续通过样例调参。
- 跨平台打包、签名和许可证审核尚未完成。

## 7. 风险和回滚

- 风险：本机 FFmpeg 版本差异、sidecar 体积、许可证或签名要求可能影响发布方式。
- 风险：不同输入的进度解析不稳定，可能需要阶段级而非精确百分比。
- 风险：字体和复杂滤镜在不同平台表现不同。
- 监控信号：渲染失败率、取消后残留进程、输出验证失败率和平均耗时。
- 回滚方式：关闭 GIF 导出入口，保留输入和编辑 UI；移除 dialog 依赖、渲染 command 和 FFmpeg 适配，不修改用户源文件。

## 8. 与设计的差异

- MVP 按设计使用单 filter graph 传递调色板，没有单独 palette 临时文件；FFmpeg 不是 sidecar，而是由 PATH/环境变量提供。设计文档已同步这一边界。
