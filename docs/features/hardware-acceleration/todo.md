# 细化开发任务清单：硬件加速编码策略探测 (hardware-acceleration)

- 状态：merged
- Feature slug: hardware-acceleration
- 分支：feature/hardware-acceleration
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 文档与规范准备
- [x] 探测本地 FFmpeg sidecar 硬件编码器（确认 `h264_videotoolbox` 可用）
- [x] 编写 `research.md`、`requirements.md`、`design.md`、`implementation.md`、`todo.md`
- [x] 提交初始五件套规范文档

## 2. Rust 后端实现 (`src-tauri/src/lib.rs`)
- [x] 定义 `HardwareAccelerationInfo` 结构体
- [x] 实现 `detect_hardware_acceleration` 探测逻辑（支持 VideoToolbox / NVENC / QSV / 软编降级）
- [x] 注册 Tauri 命令 `get_hardware_acceleration_info`
- [x] `RenderPlan` 新增 `hw_accel: Option<String>` 字段
- [x] `build_ffmpeg_args` 与 `build_ffmpeg_args_with_hw` 实现硬件解码与编码分支（`-hwaccel auto` 与硬件编码器注入）
- [x] 编写原生单元测试：
  - [x] `detects_hardware_acceleration_info`
  - [x] `builds_mp4_args_with_hardware_acceleration_when_available`
  - [x] `builds_mp4_args_with_software_when_hw_disabled`
- [x] 运行并通过 `cargo test`

## 3. 前端模型与类型扩展 (`src/lib/*`)
- [x] `src/lib/desktop.ts`: 增加 `HardwareAccelerationInfo`, `HwAccelPolicy`, `getHardwareAccelerationInfo`
- [x] `src/lib/templates.ts`: `StudioDraft` 支持 `hwAccel`
- [x] `src/lib/queue.ts`: `QueueItem` 支持 `hwAccel`

## 4. 前端界面与样式适配 (`src/App.vue`, `src/styles.css`)
- [x] `src/App.vue`:
  - [x] 初始化时调用 `getHardwareAccelerationInfo` 获取系统硬件加速信息
  - [x] 界面展示硬件加速状态与模式切换（自动 GPU 加速 / 纯 CPU 软件）
  - [x] 组装 RenderPlan 时传递 `hwAccel`
  - [x] 操作栏展示加速徽章
- [x] `src/styles.css`:
  - [x] 增加硬件加速状态卡片、切换按钮与状态徽标样式

## 5. 质量验证与门槛检查
- [x] 编写端到端冒烟测试 `scripts/smoke-test-hardware-acceleration.mjs`
- [x] 执行全套自动化验证：`cargo test`, `pnpm build`, `cargo check --locked`
- [x] 验证硬件加速与纯软解软编两种模式
- [x] 执行 `git diff --check`
- [x] 更新文档状态为 ready-to-merge 并提交

## 6. 合并至主分支
- [x] 切换到 `main` 分支执行非快进合并
- [x] 更新 `TODO.md` 产品大方向
- [x] 推送到远程仓库并清理本地分支
