# 需求调研：AVIF 动图格式支持 (avif-format)

- 状态：proposed
- Feature slug: avif-format
- 分支：feature/avif-format
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 背景与痛点

1. **动图压缩比天花板**：
   - GIF 是 1987 年的 256 色调色板老格式，体积膨胀严重且有锯齿；
   - WebP 虽然大幅减小了体积（减少 50%~70%），但在高动态、渐变复杂场景中仍有 block 效应；
   - **AVIF (AV1 Image File Format)** 基于开放免版税的下一代 AV1 视频编码标准，相同画质下相比 WebP 可进一步缩减 20%~30% 体积，相比传统 GIF 体积仅为其 1/5 至 1/10，且支持全彩高动态范围与平滑渐变。
2. **现代浏览器与操作系统生态普及**：
   - Chrome、Safari (iOS 16+ / macOS Ventura+)、Firefox、Edge 现已全面原生支持 AVIF 动图序列（`image/avif`，ISO Media AVIF Image Sequence）；
   - Telegram、Discord 等现代社交软件已支持直接发送与播放 AVIF 动图。
3. **本地 FFmpeg 编码器成熟**：
   - 本项目自带的 FFmpeg 9.0.2 sidecar 已同时内置了目前业内转码速度最快、多核扩展性极佳的 `libsvtav1` 以及经典 `libaom-av1` 编码器，并原生支持 `avif` muxer。

## 2. 调研结论与关键决策

| 调研项 | 现状 / 发现 | 方案决策 |
|--------|-------------|----------|
| 编码器选型 | sidecar 包含 `libsvtav1` 和 `libaom-av1` | 优先使用 `libsvtav1`，采用多核多线程优化与预设 `-preset 8`，转码速度快 10 倍以上，秒级输出表情包 |
| 动图格式标准 | AVIF 支持单图与动图序列（Image Sequence） | 使用 ffmpeg 输出 `.avif`，自动打包为标准 ISO Media AVIF 动图文件，支持循环播放 |
| 像素格式限制 | SVT-AV1 要求严格的 YUV420P 像素格式 | 滤镜链统一追加 `format=yuv420p` 保证兼容所有带透明度或 RGB 源输入 |
| 质量梯队 (CRF) | low / medium / high 三档与 AV1 CRF 映射 | low: CRF 38 (极致小巧)；medium: CRF 32 (平衡推荐)；high: CRF 26 (无损级细节) |
| 工作区与队列融合 | 原有系统已有 GIF 与 WebP 支持 | 扩展为三格式通用支持 (`"gif" \| "webp" \| "avif"`)，历史记录与批量队列无缝兼容 |

## 3. 风险与未知项

- **部分陈旧聊天软件兼容性**：微信电脑版旧版本可能无法直接粘贴原生 `.avif`。
  - 对策：在格式选择栏清晰标明“AVIF (超小体积/新一代格式)”，保留 GIF 和 WebP 作为经典与兼容之选。
