# 需求说明书：硬件加速编码策略探测 (hardware-acceleration)

- 状态：merged
- Feature slug: hardware-acceleration
- 分支：feature/hardware-acceleration
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 范围与目标

- 核心目标：
  1. 在 Rust 端实现宿主机硬件加速能力的动态探测（支持 macOS VideoToolbox、Windows/Linux NVENC/QSV/VAAPI 等）；
  2. 在 MP4 转码与视频解码中，根据硬件策略优先启用原生 GPU 硬件加速；
  3. 控制面板呈现当前检测到的硬件加速引擎徽标，并提供“自动加速 / 纯软件编码”切换选项；
  4. 具备严格的不可用降级策略，确保任何环境都能 100% 成功导出。
- 不在范围：暂不强制替换 GIF/WebP 的图像调色板阶段（GIF 本身标准不支持硬件编码）。

## 2. 用户故事

- 作为一个在 MacBook / iMac 上制作表情包的创作者，我希望在导出短视频时调用 Apple Silicon 的 Media Engine 硬件加速，导出速度达到 5~10 倍且风扇不发出噪音。
- 作为一个 Windows / PC 用户，我希望如果电脑有 NVIDIA 独显或 Intel 核显，导出能调用 NVENC 或 QSV 硬件加速。
- 作为一个对兼容性有极致要求的用户，如果我的虚拟化环境无法调用 GPU，我希望软件无感且稳定地回退到 CPU 软件压制，而不是直接报错崩溃。

## 3. 功能需求

1. **Rust 端硬件能力探测指令 (`get_hardware_acceleration_info`)**：
   - 返回结构体包含：
     - `supported: bool`（当前系统与 FFmpeg 是否支持硬件加速）
     - `engine: String`（例如 `"VideoToolbox"`, `"NVENC"`, `"QuickSync"`, `"None"`）
     - `encoder: String`（例如 `"h264_videotoolbox"`, `"h264_nvenc"`, `"h264_qsv"`, `"libx264"`）
     - `description: String`（展示给用户的硬件说明）
2. **转码管线策略适配**：
   - `RenderPlan` 新增 `hw_accel: Option<String>`（`"auto"` | `"enabled"` | `"disabled"`）；
   - 当 `hw_accel != "disabled"` 且探测到支持硬件加速时：
     - 解码参数追加 `-hwaccel auto`；
     - MP4 编码器切换为对应硬件编码器（如 `h264_videotoolbox`），并应用对应质量参数映射；
   - 当 `hw_accel == "disabled"` 或硬件编码器未就绪时：
     - 保持现有稳定标准的 `libx264` 软编流程。
3. **前端状态展示与控制**：
   - 控制面板第 04 节或顶部状态栏展示硬件加速徽标（如 `⚡ GPU 加速: VideoToolbox`）；
   - 提供加速模式切换选项（默认“⚡ 自动 GPU 加速”，可手动切换为“⚙ 纯 CPU 软件编码”）；
   - 在批量导出队列中同步保留硬件加速选项。

## 4. 验收标准

- Rust 单元测试验证不同系统硬件探测逻辑与参数构建规则；
- 在具备硬件编码器（如 macOS aarch64）的机器上，导出的 MP4 文件实际由硬件编码器生成，并通过 `ffprobe` 验证编码器标识；
- 手动切换为“纯 CPU 软件编码”后，正确回退并使用 `libx264`；
- 端到端冒烟测试实测通过；
- `pnpm build`, `cargo test`, `cargo check --locked` 绿灯通过。
