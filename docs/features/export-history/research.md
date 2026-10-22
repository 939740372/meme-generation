# 需求调研：最近导出历史与快捷操作

- 状态：designed
- Feature slug: export-history
- 分支：feature/export-history
- 负责人：待定
- 日期：2026-10-22

## 1. 背景和问题

用户在使用表情工坊（Meme Studio）完成 GIF 导出后，当前仅在底部导出栏显示静态文字与输出路径。当用户批量制作多张表情包、或需要将导出的 GIF 发送到社交平台（微信、Slack、QQ、Discord）时，面临以下痛点：

1. **查找路径繁琐**：用户必须频繁手动切换到 macOS 访达（Finder）或 Windows 资源管理器，一层层找到导出的目标文件夹；
2. **复制分享不便**：聊天软件往往支持直接粘贴文件或图片，但目前无法在应用内一键复制文件到剪贴板；
3. **缺乏历史概览与溯源**：制作了 3~5 张不同台词或时段的 GIF 后，无法快速回顾之前导出的体积大小与效果；
4. **无法复用历史参数**：如果想基于上一版稍作修改（如字体颜色改红、裁剪区左移 20px），一旦重新打开或重置，之前的参数全部丢失，需要重新手工配置。

## 2. 用户和场景

- **场景 1：导出后秒发微信（一键定位与复制）**
  用户导出了一张 “你过来啊” 表情包，底部栏直接出现「在访达中显示」和「复制文件」按钮。点击「复制文件」，系统直接将该 GIF 文件写入系统剪贴板，切换到微信窗口按下 `Cmd+V` 即可直接发送，无需打开访达。
- **场景 2：回顾与对比制作结果（侧边栏与底部抽屉）**
  用户连续导出了 3 个版本的表情包，在侧边栏可以看到最近导出的微型卡片与体积（如 420 KB、890 KB），点击展开底部抽屉可以清晰对比缩略图、分辨率与体积。
- **场景 3：参数一键回溯（载入此历史参数）**
  用户对 20 分钟前导出的某个 GIF 效果非常满意，希望针对新视频沿用这套参数，或重新微调该 GIF 的文字内容。点击「载入参数」，应用自动恢复源视频路径、时间范围、裁切尺寸、叠加文字位置与预设质量。

## 3. 现有技术调研与方案对比

### 3.1 访达/资源管理器定位 (Reveal Item in Dir)
- Tauri 2 官方提供了 `@tauri-apps/plugin-opener` 插件。
- 调研发现该插件在 `index.js` 中已直接导出 `revealItemInDir(path: string | string[])`，并在默认权限 `opener:default`（`allow-reveal-item-in-dir`）中已全量授权。
- 无需引入额外外部依赖，即可原生支持在 macOS Finder / Windows Explorer 中定位并高亮选中文件。

### 3.2 复制文件到系统剪贴板 (Copy File to Clipboard)
- Web 标准 `navigator.clipboard.writeText(path)` 仅能复制文本路径。
- 微信、Slack 等现代桌面聊天客户端在按 `Cmd+V` 时，优先识别剪贴板中的文件对象（`NSFilenamesPboardType` / `NSPasteboardURL`）而非纯文本路径。
- 在 macOS 上，通过执行 `osascript -e 'set the clipboard to (POSIX file "...")'` 可安全、纯原生将真实文件对象挂载到剪贴板；同时前端同步调用 `navigator.clipboard.writeText(path)` 兼容纯文本粘贴。双重保障兼顾体验。

### 3.3 缩略图渲染
- 导出完成后，前端需要呈现 GIF 动图或静态缩略图。
- 直接通过 Rust 提供原子指令 `read_file_as_data_url(path: String)`，以 Base64 Data URL 读取图片。不仅无需配置复杂的静态资产服务器 scope，而且跨平台离线 100% 可用，对单张 < 5MB 的表情包读取仅耗时数毫秒。

### 3.4 持久化与存储开销
- 历史记录轻量（仅元数据与参数，Base64 缩略图按需读取或缓存），存储在浏览器的 `localStorage` 中（键名：`meme_studio_export_history`）。
- 限制最大容量为 8~10 条，遵循 LRU 淘汰机制，占用存储小于 20KB，启动零负担。

## 4. 调研结论和决策

| 结论/决策 | 理由 | 影响 |
| --- | --- | --- |
| 使用 `revealItemInDir` 原生能力 | 官方插件开箱即用，定位精准 | 用户无需手动寻径 |
| 引入 Rust 原生复制文件与 Base64 读取 | 真正支持向微信/聊天软件 Cmd+V 粘贴文件 | 极致桌面端体验 |
| 侧边栏常驻微卡片 + 底部收折抽屉 | 充分利用侧边栏 500px 闲置空间，严守 1280×820 固定视口 | 零滚动条，UI 精巧优雅 |
| 完整存储 RenderPlan 参数并支持一键还原 | 赋能历史参数回溯与微调 | 极大提升二次创作效率 |

## 5. 参考资料

- Tauri 2 Opener 插件文档：`revealItemInDir`
- AppleScript 官方文档：NSPasteboard POSIX file 剪贴板绑定
- Web API `navigator.clipboard`
