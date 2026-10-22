# 需求调研：硬件加速编码策略探测 (hardware-acceleration)

- 状态：implementing
- Feature slug: hardware-acceleration
- 分支：feature/hardware-acceleration
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 背景与痛点

1. **CPU 密集型软解与软编的性能瓶颈**：
   - 当前 Meme Studio 在视频导入（解封装/解码）与短视频导出（MP4 H.264 编码）阶段，均默认使用纯 CPU 软件运算（`libx264`、默认解码器）；
   - 在现代高分辨率（1080P/4K）源视频截取，或批量任务队列（例如连续导出 10+ 个表情片段）时，多核 CPU 持续高负荷运转，导致笔记本风扇狂转、发热降频、电池耗电剧烈，且编码耗时明显拉长。
2. **现代操作系统的专属硬件编码器**：
   - **macOS (Apple Silicon M系列及 Intel 架构)**：内置专用的媒体引擎（Media Engine），提供原生的 `h264_videotoolbox` 与 `hevc_videotoolbox` 硬件编解码器，编码能效极高，耗时仅需 CPU 软编的 1/5 到 1/10，且几乎不占用 CPU 计算核心；
   - **Windows / Linux**：广泛配备 NVIDIA NVENC (`h264_nvenc`)、Intel QuickSync Video (`h264_qsv`) 或 AMD AMF (`h264_amf`) 独立硬件编码单元；
   - 目前内置的 FFmpeg sidecar 二进制已完整编译并集成了 `h264_videotoolbox` 等硬件接口，但应用层尚未主动探测并应用该加速能力。

## 2. 调研结论与技术选型

| 调研项 | 方案对比 | 决策依据 |
|--------|----------|----------|
| 硬件编码器探测方式 | 静态系统判断 vs FFmpeg 编码器列表动态探测 + 1 帧冒烟验证 | **动态探测 + 冒烟验证**：通过执行 `ffmpeg -encoders` 与 1 帧超轻微缓存测试，确认当前宿主机硬件是否真正可用，防止无 GPU 权限的虚拟化环境抛出异常 |
| 跨平台硬件加速器映射 | 优先匹配宿主系统硬件 | macOS 首选 `h264_videotoolbox`；Windows/Linux 首选 `h264_nvenc`，次选 `h264_qsv`；均不可用时自动回退 `libx264` |
| 解码端硬件加速 | `-hwaccel auto` | 在源视频解封装时，声明 `-hwaccel auto`，由 FFmpeg 依据平台自动选配可用硬件解码器，无法硬件解码时自动无损降级为软解 |
| 质量参数对应关系 | libx264 `crf` vs videotoolbox `q:v` | videotoolbox 接受 `-q:v`（0~100，推荐 high=75, medium=65, low=50）；NVENC 接受 `-cq`；自动根据激活的编码器动态注入适配参数 |
| 用户控制策略 | 强制开启 vs 自动探测 / 用户可切换 | 提供 **“⚡ 自动加速 (推荐)”** 与 **“⚙ 纯 CPU 软件压制”** 策略切换，并实时展示检测到的 GPU 硬件加速设备状态 |

## 3. 风险与未知项

- **部分极端分辨率与色深限制**：
  - 部分旧版硬件编码器对奇数分辨率或特殊色深敏感；
  - 应对方案：管线中统一保持 `scale=w:-2` 强制偶数对齐，且格式标准化为 `nv12` 或 `yuv420p`；并在 Rust 进程启动失败时具备自动降级至 CPU 软编的兜底保护。
