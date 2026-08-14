# 设计文档：FFmpeg GIF MVP

- 状态：designed
- Feature slug：ffmpeg-gif-mvp
- 分支：feature/ffmpeg-gif-mvp
- 负责人：待定
- 日期：2026-08-14

## 1. 方案摘要

前端维护一个可序列化的 RenderPlan，负责编辑参数、展示预览状态和呈现任务事件。Tauri command 接收 RenderPlan，Rust 校验路径和数值，构建结构化 FFmpeg 参数数组，启动受控的 sidecar 进程，并将阶段、进度、日志摘要和结果事件发回前端。

不让 Vue 直接拼接命令，不让 Rust 接收一段完整 shell 字符串。FFmpeg 只负责媒体转换，Tauri 负责权限边界、临时目录、进程生命周期和结果验证。

## 2. 用户流程

1. 用户选择本地视频。
2. 前端请求媒体信息并展示时长、尺寸、帧率等可用信息。
3. 用户设置时间范围、裁切、宽度、帧率、文字和 PNG 图案。
4. 前端保存 RenderPlan，并显示可执行的预览状态。
5. 用户开始导出，Rust 创建临时目录和 FFmpeg 任务。
6. FFmpeg 分阶段执行处理，Tauri 推送进度和错误摘要。
7. 用户可以取消；取消后终止子进程并清理临时文件。
8. 完成后验证输出存在且可被读取，将结果路径返回前端。

## 3. 模块边界

| 模块 | 职责 | 输入 | 输出 | 不负责 |
| --- | --- | --- | --- | --- |
| Vue editor | 参数编辑、预览状态、任务控制 | 用户操作、媒体信息、事件 | RenderPlan、界面状态 | 执行 FFmpeg |
| Tauri command | 权限入口、参数校验、任务管理 | RenderPlan、文件路径 | 结果、事件、错误 | 复杂 UI 逻辑 |
| FFmpeg runner | sidecar 启动、参数构建、进度解析、退出处理 | 已校验参数 | 进度、退出码、输出路径 | 决定产品默认值 |
| RenderPlan | 描述一次可重复渲染 | 输入、变换、叠加、输出 | 可序列化任务 | 保存用户媒体本身 |
| 临时文件管理 | 调色板、日志和失败清理 | 任务上下文 | 可清理目录 | 长期项目存储 |

## 4. 数据和状态

RenderPlan 至少包含：

- input：源文件路径和探测到的媒体信息；
- trim：起止时间；
- crop：x、y、width、height；
- output：宽度、帧率、目标路径和 GIF 选项；
- overlays：文字和 PNG 图案的内容、位置、尺寸与样式；
- task：任务 ID、状态和取消句柄，不写入用户项目文件。

任务状态为 idle → probing → ready → rendering → completed/failed/cancelled。状态转移由 Rust 任务事件驱动，前端不通过猜测进程状态来显示成功。

默认处理链路：

1. 输入探测；
2. trim、crop、scale、fps 和 overlay filter；
3. 生成调色板；
4. 使用调色板生成 GIF；
5. 验证输出并清理临时文件。

## 5. Command / API / Event 契约

| 名称 | 方向 | 输入 | 输出/事件 | 错误 |
| --- | --- | --- | --- | --- |
| media_probe | Vue → Tauri | 输入路径 | MediaInfo | 路径不可读、格式不支持 |
| render_gif | Vue → Tauri | RenderPlan | task_id | 参数非法、sidecar 不可启动 |
| cancel_render | Vue → Tauri | task_id | cancelled | 任务不存在或已结束 |
| render_progress | Tauri → Vue | task_id、阶段、百分比 | UI 进度状态 | 解析失败时保留阶段信息 |
| render_completed | Tauri → Vue | task_id、输出路径、摘要 | 成功状态 | 输出验证失败 |
| render_failed | Tauri → Vue | task_id、错误码、用户摘要 | 失败状态 | 不泄露完整本地敏感路径 |

## 6. 安全和权限

- 只允许用户选择的输入、输出和图案路径进入任务。
- 数值参数在 Rust 侧进行范围校验；文本、字体路径和文件名作为独立参数传递。
- 禁止通过 shell 解释器执行拼接后的命令。
- 日志只返回必要摘要，避免将完整本地路径、用户文本或媒体内容暴露到不必要的界面。
- 临时目录使用应用数据目录下的任务子目录，并在成功、失败和取消路径清理。

## 7. 跨平台差异

- macOS、Windows、Linux 使用不同的 sidecar target 和可执行文件命名。
- 进程终止、权限错误、输出目录选择和系统默认字体路径需要平台适配。
- 发布前分别验证 sidecar 签名/权限和应用打包结果。
- 没有可用字体时应提供明确错误或稳定的回退策略。

## 8. 性能和可观测性

- 优先使用短任务探测和阶段化进度，避免前端轮询完整日志。
- 大文件和长视频不能一次性读入前端内存。
- 记录 task_id、阶段、耗时、FFmpeg 退出码和错误分类，不记录媒体内容。
- 调色板和输出都写入临时/目标路径，不在 WebView 中承担完整编码。

## 9. 关键决策

| 决策 | 备选方案 | 选择理由 | 影响 |
| --- | --- | --- | --- |
| RenderPlan 作为唯一任务输入 | 在前端直接拼命令 | 可测试、可序列化、边界清晰 | 需要定义稳定的数据契约 |
| Rust 管理 sidecar | 前端直接调用本地进程 | 更适合权限和生命周期管理 | 需要 Tauri command/event |
| 两阶段 palettegen/paletteuse | 单阶段直接输出 GIF | 颜色表现更稳定 | 需要临时调色板和清理 |
