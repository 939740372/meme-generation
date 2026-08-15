# 设计文档：FFmpeg GIF MVP

- 状态：implementing
- Feature slug：ffmpeg-gif-mvp
- 分支：feature/ffmpeg-gif-mvp
- 负责人：待定
- 日期：2026-08-14

## 1. 方案摘要

前端维护一个可序列化的 RenderPlan，负责编辑参数和呈现任务事件。Tauri command 接收 RenderPlan，Rust 校验路径和数值，构建结构化 FFmpeg 参数数组，启动受控的本机 FFmpeg 进程，并将阶段、进度、日志摘要和结果事件发回前端。FFmpeg 路径由 `FFMPEG_BINARY`/`FFPROBE_BINARY` 或当前 `PATH` 提供；本轮不把二进制打进应用。

不让 Vue 直接拼接命令，不让 Rust 接收一段完整 shell 字符串。FFmpeg 只负责媒体转换，Tauri 负责权限边界、临时目录、进程生命周期和结果验证。

## 2. 用户流程

1. 用户选择本地视频。
2. 前端请求媒体信息并展示时长、尺寸、帧率等可用信息。
3. 用户设置时间范围、可选裁切、宽度、帧率、文字和 PNG 图案。
4. 前端保存 RenderPlan，并显示可执行的预览状态。
5. 用户开始导出，Rust 创建 FFmpeg 任务并保留可取消的子进程句柄。
6. FFmpeg 分阶段执行处理，Tauri 推送进度和错误摘要。
7. 用户可以取消；取消后终止子进程并清理临时文件。
8. 完成后验证输出存在且可被读取，将结果路径返回前端。

## 3. 模块边界

| 模块 | 职责 | 输入 | 输出 | 不负责 |
| --- | --- | --- | --- | --- |
| Vue editor | 参数编辑、预览状态、任务控制 | 用户操作、媒体信息、事件 | RenderPlan、界面状态 | 执行 FFmpeg |
| Tauri command | 权限入口、参数校验、任务管理 | RenderPlan、文件路径 | 结果、事件、错误 | 复杂 UI 逻辑 |
| FFmpeg runner | 本机进程启动、参数构建、进度解析、退出处理 | 已校验参数 | 进度、退出码、输出路径 | 决定产品默认值 |
| RenderPlan | 描述一次可重复渲染 | 输入、变换、叠加、输出 | 可序列化任务 | 保存用户媒体本身 |
| 任务管理 | 子进程句柄、取消和状态清理 | 任务上下文 | 可清理的内存状态 | 长期项目存储 |

## 4. 数据和状态

RenderPlan（MVP 版本）包含：

- `inputPath`：源文件路径；媒体信息由独立 `media_probe` 返回；
- `startSecs`、`endSecs`：非负且 `endSecs > startSecs`；
- `crop`：可选的 x、y、width、height，必须落在探测到的画面内；
- `outputWidth`、`fps`、`outputPath`：输出宽度和 FPS 在 Rust 再次校验；
- `textOverlay`：可选文本、x/y、字号和十六进制颜色；
- `imageOverlay`：可选 PNG 路径、x/y、宽度；
- `task` 不进入 RenderPlan，由 Rust 生成 task_id 和取消句柄。

任务状态为 idle → probing → ready → rendering → completed/failed/cancelled。状态转移由 Rust 任务事件驱动，前端不通过猜测进程状态来显示成功。

默认处理链路：

1. 输入探测（`ffprobe`）；
2. 使用 `-ss`/`-t` 和 filter graph 完成 crop、scale、fps、drawtext、PNG overlay；
3. 在同一 filter graph 内 `split`，由一支 `palettegen` 生成调色板；
4. 另一支通过 `paletteuse` 生成 GIF；
5. 验证输出存在且非空，清理任务状态。

## 5. Command / API / Event 契约

| 名称 | 方向 | 输入 | 输出/事件 | 错误 |
| --- | --- | --- | --- | --- |
| media_probe | Vue → Tauri | 输入路径 | MediaInfo | 路径不可读、格式不支持 |
| render_gif | Vue → Tauri | RenderPlan | task_id | 参数非法、FFmpeg 不可启动 |
| cancel_render | Vue → Tauri | task_id | cancelled | 任务不存在或已结束 |
| render_progress | Tauri → Vue | task_id、阶段、百分比 | UI 进度状态 | 解析失败时保留阶段信息 |
| render_completed | Tauri → Vue | task_id、输出路径、摘要 | 成功状态 | 输出验证失败 |
| render_failed | Tauri → Vue | task_id、错误码、用户摘要 | 失败状态 | 不泄露完整本地敏感路径 |

## 6. 安全和权限

- 只允许用户选择的输入、输出和图案路径进入任务。
- 数值参数在 Rust 侧进行范围校验；文本、字体路径和文件名作为独立参数传递。
- 禁止通过 shell 解释器执行拼接后的命令。
- 日志只返回必要摘要，避免将完整本地路径、用户文本或媒体内容暴露到不必要的界面。
- 不创建调色板临时文件：调色板在 filter graph 中传递；任务状态和子进程句柄只保存在内存中，成功、失败和取消后移除。
- 输出路径必须有父目录且不能已存在；不删除用户已有文件。

## 7. 跨平台差异

- macOS、Windows、Linux 使用不同的 sidecar target 和可执行文件命名；本轮只通过 PATH/环境变量解析本机可执行文件。
- 进程终止、权限错误、输出目录选择和系统默认字体路径需要平台适配。
- 发布前分别验证 sidecar 签名/权限和应用打包结果。
- 没有可用字体时应提供明确错误或稳定的回退策略。

## 8. 性能和可观测性

- 优先使用短任务探测和阶段化进度，避免前端轮询完整日志。
- 大文件和长视频不能一次性读入前端内存。
- 记录 task_id、阶段、耗时、FFmpeg 退出码和错误分类，不记录媒体内容。
- 调色板由 filter graph 在内存中传递，输出写入目标路径，不在 WebView 中承担完整编码。

## 9. 关键决策

| 决策 | 备选方案 | 选择理由 | 影响 |
| --- | --- | --- | --- |
| RenderPlan 作为唯一任务输入 | 在前端直接拼命令 | 可测试、可序列化、边界清晰 | 需要定义稳定的数据契约 |
| Rust 管理 sidecar | 前端直接调用本地进程 | 更适合权限和生命周期管理 | 需要 Tauri command/event |
| 两阶段 palettegen/paletteuse | 单阶段直接输出 GIF | 颜色表现更稳定 | 需要临时调色板和清理 |
| 单 filter graph 传递 palette | 写入单独 palette 临时文件 | 让 MVP 的取消/清理更简单 | 复杂滤镜仍需后续扩展测试 |

## 10. 桌面工作区设计

- Tauri 窗口基准尺寸固定为 `1280 × 820`，`minWidth`/`maxWidth` 与 `minHeight`/`maxHeight` 均锁定到基准值，并关闭 `resizable`。
- 根视口使用固定高度布局，禁止页面级横向/纵向滚动；内部通过 `grid`/`flex` 的收缩约束把预览、控制和导出区域压入窗口。
- 视觉结构为左侧导航/运行时状态、顶部工作区栏、中央预览画布、右侧参数面板和底部导出状态栏。固定区域之间使用深色面板、细边框、等宽微标签和单一强调色建立桌面工具层级。
- 文字层级以可读性优先：导航约 18px，正文和控件约 14–16px，参数输入使用 17px，技术标签保留 10–12px；通过收紧面板间距保持固定窗口内的完整性。
- 本轮精简信息层级：侧栏只保留当前 GIF 制作，移除未实现的历史/快捷键入口；页头只保留工作区名称和运行状态；删除营销式副标题、重复本地处理说明和底部宣传 footer。
- 本轮字号目标：主要导航约 18px、面板标题约 24px、参数输入约 17px；技术标签可保留较小字号，但不承担主要阅读信息。通过缩短标题和减少固定区域高度保持 1280×820 工作区不变形。
- 第一版语言策略：界面、状态、错误和系统文件筛选名称使用简体中文；FFmpeg、GIF、PNG、Tauri、FPS、MP4/MOV/WEBM 等技术名词、单位和扩展名保留原文或可识别缩写。
- 浏览器预览仍保留，但只用于查看壳层；由于窗口固定策略，浏览器窗口缩放时可能看到内容被压缩，真实交互以 Tauri 固定窗口为准。
