# 设计文档：FFmpeg Sidecar 打包与 libfreetype 文字渲染

- 状态：merged
- Feature slug：ffmpeg-sidecar-packaging
- 分支：feature/ffmpeg-sidecar-packaging
- 负责人：待定
- 日期：2026-10-22

## 1. 方案摘要

本方案确立了 Meme Studio 的 FFmpeg Sidecar 打包与文字渲染链路：
1. **Sidecar 打包配置**：在 `tauri.conf.json` 中配置 `bundle.externalBin: ["binaries/ffmpeg", "binaries/ffprobe"]`。
2. **二进制资源管理**：
   - 二进制放置在 `src-tauri/binaries/`，遵循 `<name>-<target-triple>` 命名规范；
   - 纳入 `.gitignore`，编写 `scripts/download-ffmpeg-sidecar.mjs` 负责从高可用静态构建源下载包含 `libfreetype`、`libharfbuzz` 的独立二进制；
3. **Rust 多级二进制解析器**：
   - 提取 `resolve_binary(app: &AppHandle, name: &str) -> PathBuf`；
   - 级联顺序：
     1. 环境变量重写（`FFMPEG_BINARY` / `FFPROBE_BINARY`）；
     2. Tauri 打包资源目录（通过 `app.path().resource_dir()` 或当前进程可执行目录同级查找 sidecar）；
     3. 开发期相对路径 `src-tauri/binaries/<name>-<target-triple>`；
     4. 系统环境变量 PATH 回退；
4. **字体与 drawtext 渲染设计**：
   - 自动检测宿主平台字体文件路径：
     - macOS: `/System/Library/Fonts/PingFang.ttc`（首选）或 `/System/Library/Fonts/Helvetica.ttc`；
     - Windows: `C:\Windows\Fonts\msyh.ttc`（微软雅黑）或 `C:\Windows\Fonts\arial.ttf`；
     - Linux: `/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`；
   - 构建 `drawtext` 滤镜字符串：
     `drawtext=fontfile='...':text='...':x=...:y=...:fontsize=...:fontcolor=...`
     对路径、单引号及冒号等特殊字符进行安全转义，防止 FFmpeg 参数解析注入或断裂。

## 2. 模块边界与数据流

~~~mermaid
flowchart TD
    subgraph UI ["Vue 3 前端"]
        A["用户输入文本与颜色"] --> B["RenderPlan"]
    end

    subgraph Tauri ["Tauri / Rust 后端"]
        B --> C["render_gif command"]
        C --> D["resolve_binary 解析 ffmpeg 路径"]
        D -->|优先| E["Tauri Bundled Sidecar"]
        D -->|开发回退| F["src-tauri/binaries/"]
        D -->|最终回退| G["System PATH"]
        C --> H["detect_system_font 匹配平台字体"]
        H --> I["build_filter_graph 组装 drawtext + palettegen"]
        I --> J["Command::new(ffmpeg) 异步执行"]
    end

    subgraph FFmpeg ["FFmpeg 静态进程 (with libfreetype)"]
        J --> K["解码视频 -> drawtext 文本绘制 -> 调色板生成与应用 -> GIF 写入"]
    end
~~~

## 3. 错误处理与健壮性

1. **二进制未找到**：
   - 返回清晰的 `FFMPEG_NOT_FOUND` 错误，提示用户当前平台缺失 sidecar 二进制，或指引运行准备脚本。
2. **drawtext 能力缺失**：
   - 执行前通过 `ffmpeg -filters` 或启动探测；若确实缺失则提示 `DRAW_TEXT_UNAVAILABLE`，保证无文字时仍可正常导出。
3. **字体文件不存在**：
   - 若系统默认字体文件不存在，尝试回退到 fontconfig 默认无 fontfile 模式，并在日志中输出警告，不直接让整个渲染流程崩溃。

## 4. 跨平台目标架构对照表

| 平台 | Target Triple | 二进制文件名 | 字体回退路径 |
| --- | --- | --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` | `ffmpeg-aarch64-apple-darwin` | `/System/Library/Fonts/PingFang.ttc` |
| macOS Intel | `x86_64-apple-darwin` | `ffmpeg-x86_64-apple-darwin` | `/System/Library/Fonts/PingFang.ttc` |
| Windows x64 | `x86_64-pc-windows-msvc` | `ffmpeg-x86_64-pc-windows-msvc.exe` | `C:\Windows\Fonts\msyh.ttc` |
| Linux x64 | `x86_64-unknown-linux-gnu` | `ffmpeg-x86_64-unknown-linux-gnu` | `/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf` |
