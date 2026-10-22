# Meme Studio 产品方向 TODO

## 北极星

构建一个本地优先、跨平台、易上手的 Meme/GIF 创作桌面客户端：用户可以把各种视频快速变成可分享的 GIF，并在导出前完成裁切、压缩、抽帧、文字和图案叠加。

## 当前优先级

### P0：工程底座（已完成）

- [x] Tauri 2 + Vue 3 + TypeScript + Vite
- [x] 跨平台窗口和 Rust command 基础
- [x] Git 仓库和基础构建校验
- [x] 开发流程规范与 feature 文档模板

### P1：FFmpeg GIF MVP

详细文档：docs/features/ffmpeg-gif-mvp/

本次 `feature/ffmpeg-gif-mvp` 先交付本机 FFmpeg 验证版；sidecar 分发、跨平台打包和完整桌面交互仍需后续验收。

#### 本次最小闭环（已完成并合并到 main）

- [x] Tauri dialog 选择视频/PNG/输出路径，并通过 ffprobe 返回媒体信息
- [x] RenderPlan 参数校验、trim/crop/scale/fps 和 palettegen/paletteuse GIF 管线
- [x] Rust 任务进度、取消、失败分类、输出存在性/可读性检查
- [x] Vue 单页编辑器、参数面板、导出状态和错误反馈
- [x] 用临时短视频和 PNG 完成真实 GIF/overlay 冒烟
- [x] 建立 FFmpeg 探测、版本读取和错误归一化服务
- [x] 支持视频导入、媒体信息读取和基础预览
- [x] 支持开始时间、结束时间、裁切区域、输出宽度和 FPS
- [x] 支持文字和 PNG 图案叠加（含本机 drawtext 能力检查与降级处理）
- [x] 使用调色板流程导出 GIF
- [x] 显示进度、取消任务、失败原因和输出文件位置
- [x] 增加端到端冒烟验证用例
- [x] 确定跨平台 FFmpeg sidecar 的版本、构建来源与许可证分发策略（详见 docs/features/ffmpeg-sidecar-packaging/）
- [x] 引入包含 libfreetype (drawtext) 的 FFmpeg sidecar 自动化准备脚本及文字渲染验证
- [ ] 三平台安装包（DMG/MSI/AppImage）分发与自动更新验收

### P2：创作工作区

- [ ] 可视化时间轴和拖拽裁切
- [ ] 画布内拖拽文字/图案并实时预览
- [ ] 常用尺寸、FPS、压缩质量预设
- [ ] 草稿、历史记录和最近导出
- [ ] 本地项目配置和可复用模板

### P3：效率与扩展

- [ ] 批量处理和任务队列
- [ ] WebP/AVIF 等更高压缩比的动图输出
- [ ] 字幕、贴纸、滤镜和多图层
- [ ] 硬件加速探测与可选编码策略
- [ ] 跨平台安装包和自动更新

## 暂不纳入当前范围

- 云端上传和服务端转码
- 用户账号、在线素材市场
- 直接在主进程中嵌入未经评估的 FFmpeg 库
- 在没有许可证和分发策略的情况下打包第三方二进制

## 更新规则

- 大方向变化在本文件记录，并新增或更新对应 feature 文档。
- 可交付的功能不要只写一句 TODO，必须补齐 docs/features/<feature-slug>/ 五件套。
- 完成一项后保留 checkbox 和相关文档链接，方便追溯。
