# 实现计划与交付记录：可视化矩形裁切框与画布交互

- 状态：ready-to-merge
- Feature slug: visual-crop-box
- 分支：feature/visual-crop-box
- 负责人：待定
- 日期：2026-10-22

## 1. 文件级实现计划

| 文件 | 变更类型 | 说明 |
| --- | --- | --- |
| `src/App.vue` | 修改 | 在中央舞台增加可视化裁切层（九宫格、暗色遮罩、8 手柄、实时尺寸气泡），实现平移与 8 方向拉伸交互逻辑，增加快捷构图按钮（居中、1:1、铺满） |
| `src/styles.css` | 修改 | 新增裁切选框、遮罩、九宫格参考线、控制手柄、方向光标与尺寸气泡样式，强化视觉层次 |
| `docs/features/visual-crop-box/*` | 新增 | 全套 feature 文档、架构图与自测记录 |

## 2. 兼容性与性能策略

1. **事件处理安全性**：
   - 使用 Pointer Events（`pointerdown`, `pointermove`, `pointerup`），并在 window 统一捕获，防止鼠标移出画布边界导致丢帧或卡死；
   - 适当阻止冒泡与默认行为，防止与底层舞台文字/贴图拖拽产生手势冲突。
2. **纯前端计算**：
   - 本次功能完全处于前端视图交互层，FFmpeg 底层转码接口协议 `crop: CropArea | null` 无需破坏性变更，100% 保持向前兼容；
   - 遮罩采用 CSS Box-Shadow 硬件加速，无论缩放还是平移均零重排，保障 60 FPS 响应。

## 3. 测试与自测方案

### 3.1 自动化检查
1. 前端类型检查与构建：
   ```bash
   pnpm build
   ```
2. Rust 检查与单测（回归校验）：
   ```bash
   PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked
   PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml
   ```
3. 代码风格与工作区检查：
   ```bash
   git diff --check
   ```

### 3.2 交互与冒烟验证项
- [x] 勾选「裁切」，画布立即出现半透明暗色遮罩与九宫格选框；
- [x] 按住选区中心可拖拽平移，边界限制在源视频内部，右侧 X/Y 输入框实时联动；
- [x] 拖拽 8 个手柄（NW, N, NE, E, SE, S, SW, W）可精准拉伸收缩，最小宽高保护生效；
- [x] 右侧控制面板手动修改 X/Y/W/H，画布选框即时响应重绘；
- [x] 点击「居中」、「1:1」、「铺满」快捷辅助按钮，选框形态瞬时更新；
- [x] 导出 GIF 验证真实 FFmpeg 裁切效果与所选区域一致；
- [x] 验证 `1280×820` 固定视口，无页面全局或局部异常滚动条。

## 4. 自测记录

- 执行时间：2026-10-22 11:46
- 平台环境：macOS (Darwin 24.3.0, Apple Silicon aarch64)
- 测试命令与输出：
  1. `pnpm build`:
     `$ vue-tsc --noEmit && vite build` -> `✓ built in 756ms`，前端类型检查与资源打包全部通过；
  2. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`:
     `Finished dev profile in 1.90s`，Rust 代码编译无警告；
  3. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml`:
     `8 passed; 0 failed; 0 ignored; finished in 0.00s`，全套 Rust 回归测试全部通过；
  4. `git diff --check`:
     无空白与代码格式违规。
- 验收结论：达到 ready-to-merge 门槛。
