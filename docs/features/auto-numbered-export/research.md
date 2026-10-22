# 需求调研：同名导出自动顺延序号不覆盖 (auto-numbered-export)

- 状态：researching
- Feature slug：auto-numbered-export
- 分支：feature/auto-numbered-export
- 负责人：Meme Studio Team
- 日期：2026-10-22

## 1. 背景和问题

在表情包/GIF 创作过程中，用户经常针对同一个视频源片段连续微调参数（例如微调文案、更换滤镜、调整裁切区域、修改输出宽度或帧率）并多次导出对比效果。
在目前的实现中：
1. 首次导出生成如 `video-meme.gif`；
2. 用户微调后再次点击导出，Rust 后端在 `validate_render_plan` 中直接检测 `if output_path.exists()` 并硬性拦截报错 `OUTPUT_EXISTS: 输出文件已存在，请选择另一个路径`；
3. 用户被迫手动去输入框修改路径或重新打开保存弹窗命名，不仅打断创作流，还容易因为误覆盖丢失先前的优秀版本。

用户明确要求：**相同名称也可以导出，只是不覆盖，保留两个文件，按系统规则会在后面加上序号（如 `meme (1).gif`, `meme (2).gif` 等）**。

## 2. 用户和场景

- **核心场景 A（对比选优）**：用户做了一个表情包导出 `cat-meme.gif`，随后加了一句新吐槽文案再次点击导出，期望生成 `cat-meme (1).gif`，两个版本都保存在文件夹中以便发给朋友挑选；
- **核心场景 B（多次连续压制）**：用户从 `320px` 改为 `240px` 测试文件体积是否低于微信 1MB 限制，连续导出多次，自动顺延成 `(1)`, `(2)`, `(3)`，无需每次手动键入新名字；
- **核心场景 C（批量队列与预设切换）**：批量队列添加多个片段时，若目标文件名发生重名，按系统标准序号规则分配独立文件路径，防止转码互相覆盖或报错中断。

## 3. 现有方案与操作系统规则调研

主流操作系统与软件的文件防覆盖重命名规则：
- **macOS Finder / 浏览器下载**：当保存已有同名文件时，自动在主文件名末尾加上带空格的圆括号序号，如 `filename (1).ext`，再次冲突则递增为 `filename (2).ext`；
- **Windows 资源管理器**：复制冲突或下载重名时，标准格式为 `filename (1).ext`、`filename (2).ext`；
- **带序号文件的再重名**：若用户主动选定 `filename (1).ext` 且已存在，系统自动递增为 `filename (2).ext`，而非嵌套出现 `filename (1) (1).ext`。

## 4. 技术调研

1. **Rust 后端路径解析与检测**：
   - 使用 Rust 原生 `std::path::{Path, PathBuf}`，无需引入额外三方依赖；
   - 提取文件所在目录 `parent`、扩展名 `extension`、主文件名 `file_stem`；
   - 识别末尾是否已存在 ` (N)` 模式（通过纯文本切分识别开括号 `" ("` 和闭括号 `")"` 并解析正整数 N）；
   - 循环探测目标物理磁盘路径是否存在 `!candidate.exists()`，找到首个空闲序号（从 1 或 N+1 开始）；
   - 在 `render_gif` 启动时自动将 `plan.output_path` 修正为唯一路径，并将最终实际使用的路径在任务完成事件 `meme://render-completed` 中下发给前端。
2. **FFmpeg 安全保护**：
   - FFmpeg 命令行参数依然保留 `-n`（退出若文件存在），形成双重安全防线，保证 100% 绝对不静默覆盖已有文件。
3. **前端状态联动**：
   - 单任务导出完成后，`outputPath` 自动更新为当前生成的实际文件路径（如从 `.../out.gif` 更新为 `.../out (1).gif`），便于下次导出自然递增到 `(2)`，且右侧历史记录与访达打开均对应真实的物理文件；
   - 队列助手 `generateQueueOutputPath` 调整原有下划线命名 `_(${counter})` 为符合系统规范的空格圆括号 ` (${counter})`。

## 5. 已确认事实

- `render_gif` 接受 `plan: RenderPlan`，其 `output_path` 为 String；
- 渲染完成后发射事件 `meme://render-completed`，附带实际的 `output_path`；
- 前端 `App.vue` 在监听到 `meme://render-completed` 时已有 `outputPath.value = event.outputPath` 逻辑；
- 当前 `validate_render_plan` 中硬编码拦截了 `if output_path.exists()`，这是报错阻塞的根源。

## 6. 当前假设

- 绝大部分用户在同一目录下的同名导出冲突在 100 次以内；设置最大递增上限（如 10,000）防止极端死循环；
- 序号按标准 ` (1)`, ` (2)`, ` (3)` 递增，最符合 macOS 和 Windows 用户的自然直觉。

## 7. 调研结论和决策

| 结论/决策 | 理由 | 影响 |
| --- | --- | --- |
| 在 Rust 层实现 `resolve_unique_output_path` | Rust 具备权威的原子文件系统探测能力，跨平台原生无差错 | 彻底避免前端与后端不同步 |
| 移除 `validate_render_plan` 中的硬拦截 `OUTPUT_EXISTS` | 转由 `resolve_unique_output_path` 自动分配唯一序号 | 同名导出完全畅通，用户零阻碍 |
| 智能识别已有 ` (N)` 序号后缀 | 避免发生 `file (1) (1) (1).gif` 丑陋嵌套 | 符合 macOS / Windows 原生规则 |
| 前端队列同步更新为系统标准序号格式 | 保持全站文件命名体系一致性 | 队列与单片导出风格统一 |
