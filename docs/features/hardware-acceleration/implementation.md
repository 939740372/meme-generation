# 实现与迁移计划：硬件加速编码策略探测 (hardware-acceleration)

- 状态：merged
- Feature slug: hardware-acceleration
- 分支：feature/hardware-acceleration
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 文件变更清单

| 文件路径 | 变更类型 | 说明 |
|----------|----------|------|
| `docs/features/hardware-acceleration/*` | 新增 | 全套五件套设计开发文档 |
| `src-tauri/src/lib.rs` | 修改 | `HardwareAccelerationInfo` 数据结构；`detect_hardware_acceleration` 探测逻辑与测试；`RenderPlan` 增加 `hw_accel`；`build_ffmpeg_args` 与 `build_ffmpeg_args_with_hw` 硬件编解码分支；单元测试 |
| `src/lib/desktop.ts` | 修改 | 导出 `HardwareAccelerationInfo`, `HwAccelPolicy`, `getHardwareAccelerationInfo` |
| `src/lib/templates.ts` | 修改 | `StudioDraft` 支持 `hwAccel?: HwAccelPolicy` |
| `src/App.vue` | 修改 | 启动探测硬件加速；提供硬件加速模式切换器（⚡ 自动加速 / ⚙ 纯 CPU）；RenderPlan 组装传递 `hwAccel`；操作栏展示加速指示徽章 |
| `src/styles.css` | 修改 | 增加硬件加速卡片 (`.hw-accel-row`)、开关按钮 (`.hw-toggle-btn`) 与状态徽章 (`.hw-accel-badge`) 样式 |
| `scripts/smoke-test-hardware-acceleration.mjs` | 新增 | 端到端冒烟测试（硬件加速编码与纯软件编码对比） |

## 2. 兼容性与降级策略

1. **环境兼容与不可用降级**：
   - 虚拟机、CI/CD 容器或不支持硬件加速的旧系统，`detect_hardware_acceleration` 自动探测返回 `supported: false, encoder: "libx264"`，无感自动回退，不抛出致命异常；
2. **用户自主控制**：
   - 用户如遇到特定显卡驱动异常，可一键切换为“纯 CPU 软件压制”，立即恢复标准 libx264 稳定执行。

## 3. 自测与验收记录

- [x] 单元测试：`PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml`
  - 22 个 Rust 单元测试全部通过（新增 `detects_hardware_acceleration_info`, `builds_mp4_args_with_hardware_acceleration_when_available`, `builds_mp4_args_with_software_when_hw_disabled` 等）。
- [x] 代码检查：`PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`（0 warning, 0 error）
- [x] 前端构建与类型检查：`pnpm build`（`vue-tsc --noEmit` + `vite build` 绿灯）
- [x] 端到端冒烟测试：`node scripts/smoke-test-hardware-acceleration.mjs`
  - 成功探测并执行 `h264_videotoolbox` 硬件编码（macOS Apple Silicon）；
  - 成功验证纯 CPU 软件编码（`libx264`）回退；
  - 验证生成文件符合标准 H.264 规格。
- [x] 全套回归测试：包含音频短视频、批处理队列、AVIF 动图、滤镜调色、模板与贴纸库全部 100% 绿灯。
- [x] 格式与规范：`git diff --check`（无异常）。
