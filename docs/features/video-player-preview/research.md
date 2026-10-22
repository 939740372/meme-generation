# 视频导入与画面实时预览调研 (Research)

## 1. 需求背景与痛点

在之前的版本中，用户导入视频后，工作区画布（Preview Stage）底层使用的是动态渐变网格与光球（`preview-grid`, `preview-orb`）作为示意背景，并未呈现真实的视频画面。

### 创作者痛点：
1. **排版“盲狙”**：用户在画布上放置文字、表情贴纸或调整位置大小时，看不到视频中人物的动作、面部位置或原视频已有的字幕，经常导致文字挡脸、贴纸位置错位；
2. **裁切无法感知**：调整九宫格矩形裁切框（Crop box）时，由于看不到画面边缘，用户无法确知是否把关键主体裁出了画面；
3. **区间无法动态预览**：用户拉动截取时间段 `[startSecs, endSecs]` 后，无法实时直观预览这个 2~3 秒动图在连续播放时的动态效果。

---

## 2. 技术方案调研

### 方案 A：Webview `<video>` 纯浏览器原生解码
- **机制**：开启 Tauri 2 的 `assetProtocol`，通过 `convertFileSrc(path)` 将本机路径转换为 `asset://localhost/...` 传给 `<video :src="...">`。
- **优点**：支持硬件解码流畅播放，浏览器内置 seek/play/pause 控制。
- **致命限制**：
  - Webview（尤其 macOS WKWebView）只支持原生容器格式（MP4、MOV、WebM）和有限编码（H.264、HEVC、VP8/VP9）。
  - 对于二创剪辑中极其常见的 **MKV、AVI、FLV、WMV** 或特殊编码视频，浏览器内核直接报错无法解码，画面黑屏。

### 方案 B：FFmpeg 极速精准单帧抽帧 (Frame Extract Pipe)
- **机制**：通过内置 FFmpeg sidecar 执行快速寻道抽帧：
  `ffmpeg -ss <time> -i <path> -frames:v 1 -vf scale='min(640,iw)':-2 -f image2pipe -vcodec mjpeg -`
  直接从标准输出 pipe 抓取 JPEG 字节流，不产生磁盘临时文件，内存极速转码（通常 15~35ms）。
- **优点**：
  - **100% 全格式支持**：所有 FFmpeg 能够解码的格式（MKV, MP4, AVI, FLV, MOV, TS, WebM 等）均可瞬间获得指定时间的高清帧；
  - **时间轴拖拽强联动**：当调整开始时间或拖动预览指针时，背景帧即刻刷新。
- **缺点**：连续快速播放需要频繁抽帧，对高帧率动态连续回放有轻微延迟。

---

## 3. 最佳架构决策：双模全兼容驱动 (Dual-Mode Universal Preview)

采用 **双模无缝协作 + 自动降级策略**：

1. **针对支持的常用格式 (MP4 / MOV / WebM)**：
   - 启用 Tauri 2 `assetProtocol`；
   - 在画布底层嵌入 `<video>` 播放元素，实现所选片段 `[startSecs, endSecs]` 的**原生硬件加速循环播放与暂停控制**；
   - 支持实时速度调节（与 2x 鬼畜加速或 0.5x 慢速特效同步倍速）以及静音试听。

2. **针对所有格式的单帧高清捕获与通用降级 (MKV / AVI / FLV 等)**：
   - Rust 端新增高效的 `extract_video_frame(path: String, time_secs: f64, max_width: Option<u32>) -> CommandResult<String>` 命令；
   - 导入任何视频时，立刻抽取 `startSecs` 处的首帧作为初始背景底图；
   - 当用户拖拽时间轴、调整开始时间时，防抖触发抽帧刷新底图；
   - 当 `<video>` 遇到不支持格式抛出 `error` 时，自动平滑无缝降级为“高清抽帧时间轴预览模式”，完全不报错、不黑屏。

3. **视觉层与交互完全统一**：
   - 无论是播放中的 `<video>` 还是抽取的静帧，顶部的多图层文字、贴纸、矩形裁切框与全局调色滤镜（CSS filter）均以绝对定位完整覆盖在画面上，达成真正的**所见即所得 (WYSIWYG)**。
