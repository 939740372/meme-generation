# 实现与迁移计划：硬件加速编码策略探测 (hardware-acceleration)

- 状态：implementing
- Feature slug: hardware-acceleration
- 分支：feature/hardware-acceleration
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 文件变更清单

| 文件路径 | 变更类型 | 说明 |
|----------|----------|------|
| `docs/features/hardware-acceleration/*` | 新增 | 全套五件套设计开发文档 |
| `src-tauri/src/lib.rs` | 修改 | `HardwareAccelerationInfo` 数据结构；`detect_hardware_acceleration` 探测指令与测试；`RenderPlan` 增加 `hw_accel`；`build_ffmpeg_args` 硬件编解码分支；单元测试 |
| `src/lib/desktop.ts` | 修改 | 导出 `HardwareAccelerationInfo`, `HwAccelPolicy`, `getHardwareAccelerationInfo` |
| `src/lib/templates.ts` | 修改 | `StudioDraft` 支持 `hwAccel?: HwAccelPolicy` |
| `src/lib/queue.ts` | 修改 | `QueueItem` 保持与传递 `hwAccel` 策略 |
| `src/App.vue` | 修改 | 启动时探测硬件加速并展示状态；提供硬件加速模式切换器；组装时传递 `hwAccel` |
| `src/styles.css` | 修改 | 增加硬件加速胶囊徽标、状态提示与开关切换样式 |
| `scripts/smoke-test-hardware-acceleration.mjs` | 新增 | 端到端冒烟测试（硬件加速编码与纯软件编码对比） |

## 2. 兼容性与降级策略

1. **环境兼容与不可用降级**：
   - 虚拟机、CI/CD 容器或不支持硬件加速的旧系统，`detect_hardware_acceleration` 自动探测返回 `supported: false, encoder: "libx264"`，无感自动回退，不抛出致命异常；
2. **用户自主控制**：
   - 用户如遇到特定显卡驱动异常，可一键切换为“纯 CPU 软件压制”，立即恢复标准 libx264 稳定执行。

## 3. 自测与验收记录

- [ ] 单元测试：`cargo test`
- [ ] 代码检查：`cargo check --manifest-path src-tauri/Cargo.toml --locked`
- [ ] 前端构建与类型：`pnpm build`
- [ ] 端到端冒烟测试：`node scripts/smoke-test-hardware-acceleration.mjs`
- [ ] 格式与规范：`git diff --check`
