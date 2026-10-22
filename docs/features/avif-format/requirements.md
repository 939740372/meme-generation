# 需求说明书：AVIF 动图格式支持 (avif-format)

- 状态：merged
- Feature slug: avif-format
- 分支：feature/avif-format
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 范围与目标

- 核心目标：在客户端导出能力中新增 AVIF 动图格式支持，实现极致高压缩比与高保真动图输出；
- 不在范围内：暂不引入 AVIF 动画向 GIF 的反向导入解码。

## 2. 用户故事

- 作为一个表情包创作者，我希望在导出表情包时能选择 AVIF 格式，使得表情包在获得高清无失真画质的同时，体积缩减到极致（数百 KB），极速上传到 Discord、Telegram 等平台。
- 作为一个高频表情包创作者，我希望在批量处理队列中也能自由添加 AVIF 格式的任务，一键混合批量导出。

## 3. 功能需求

1. **导出参数扩展**：
   - 格式选择支持三种：`GIF (经典通用)`、`WebP (高效轻便)`、`AVIF (超高压缩比)`；
   - 切换为 AVIF 格式时，默认文件扩展名自动变更为 `.avif`；
   - 导出按钮文案自动切换为 `导出 AVIF ↗`。
2. **Rust FFmpeg AVIF 转码管线**：
   - 校验输出路径扩展名必须匹配 `.avif`；
   - 滤镜链输出规整为 `format=yuv420p` 适配 AV1 编码器；
   - 使用 `libsvtav1` 进行极速多核硬件感知编码，配置高质量平衡预设 `-preset 8`；
   - 联动质量等级：
     - `low` -> CRF 38
     - `medium` -> CRF 32
     - `high` -> CRF 26
3. **数据预览与剪贴板**：
   - Rust 端 `read_file_as_data_url` 识别 `.avif` 文件的 MIME 为 `image/avif`；
   - 导出历史记录、批量队列卡片、侧边栏能够正常展示 AVIF 动图缩略图与文件体积；
   - 支持在访达/资源管理器中一键定位 AVIF 文件。
4. **批量队列兼容**：
   - `createQueueItem` 与 `generateQueueOutputPath` 完整支持 `"avif"` 格式输出。

## 4. 验收标准

- 单元测试与真实 sidecar 能成功输出以 `.avif` 为后缀的动图序列文件；
- 输出文件被操作系统识别为 `ISO Media, AVIF Image Sequence` 且可以在现代浏览器中循环播放；
- 相同视频片段下，AVIF 相比 GIF 体积减少 70% 以上，比 WebP 小 15%~30%；
- 批量队列与历史记录中可以正常展示紫色 AVIF 标签；
- `pnpm build`, `cargo check --locked`, `cargo test` 全部绿灯通过。
