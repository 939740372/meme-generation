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
- [x] macOS DMG 桌面安装包构建与分发验收（自包含 sidecar、免配置运行、产物体积 62.7MB，详见 docs/features/desktop-packaging/）

### P2：创作工作区

- [x] 可视化时间轴和拖拽裁切（详见 docs/features/creative-timeline-canvas/）
- [x] 画布内拖拽文字/图案并实时预览
- [x] 可视化矩形裁切框与九宫格画布交互（详见 docs/features/visual-crop-box/）
- [x] 文字描边与台词背景底框增强（详见 docs/features/text-styling-stroke/）
- [x] 常用尺寸、FPS、压缩质量预设（详见 docs/features/export-presets/）
- [x] 最近导出历史与快捷操作（访达定位/复制文件/载入参数，详见 docs/features/export-history/）
- [x] 常用表情包模板与草稿暂存（实时防抖暂存/启动恢复/内置经典排版/一键存为模板，详见 docs/features/templates-and-drafts/）
- [x] 多文字图层支持（多段独立文字叠加、图层列表管理、显隐/删除、链式 drawtext 渲染，详见 docs/features/multi-text-layers/）
- [x] 内置透明表情贴纸库（18 款经典免抠透明 PNG、抽屉素材库分类浏览、画布真图渲染与角手柄缩放交互，详见 docs/features/sticker-library/）
- [x] 视频导入与画面实时预览（双模架构：原生 MP4/MOV 片段循环播放与空格控制，MKV 等特殊格式 30ms 极速高清抽帧降级，排版所见即所得，详见 docs/features/video-player-preview/）

### P3：效率与扩展

- [x] WebP 动图输出（高画质小体积，支持极速单阶段编码，详见 docs/features/modern-formats/）
- [x] 批量处理和导出任务队列（FIFO 串行调度、防重名自动递增、进度联动、恢复参数，详见 docs/features/batch-queue/）
- [x] AVIF 等更高压缩比的动图输出（基于 libsvtav1，体积比 WebP 还小 20%~80%，秒级转码，详见 docs/features/avif-format/）
- [x] 全局画面滤镜与调色（黑白致郁、高反差、港风复古、反色负片、2x鬼畜加速、倒放，详见 docs/features/video-filters/）
- [x] 带音轨的短视频表情导出（MP4 格式，H.264+AAC，支持保留原声、变速音效同步与 0:a? 容错降级，详见 docs/features/audio-video-export/）
- [x] 硬件加速探测与可选编码策略（自动探测 VideoToolbox / NVENC / QSV，支持 GPU 极速压制与 CPU 软编降级，详见 docs/features/hardware-acceleration/）
- [x] 自定义背景音乐与音效混流（内置 8 款经典二创音效库、本地音频导入、原声与 BGM 独立音量平滑控制、amix 混流与 atrim 时长精确对齐，详见 docs/features/bgm-audio-mixing/）
- [x] 高级文字排版效果（自由角度旋转、圆弧弯曲环绕、动感斜体倾斜与立体阴影，全格式预览与导出一致，详见 docs/features/advanced-text-effects/）
- [x] 全面代码审计与健壮性加固（FFmpeg 单引号闭合重开转义、ffprobe 单次解析性能提升 50%、AVIF/MP4 像素偶数位对齐、本地音频跨域试听、草稿/模板高级字段同步持久化）
- [x] 同名导出自动顺延序号防覆盖（支持同名连续导出，按系统规则自动递增 (1)/(2)，保留全部历史文件，详见 docs/features/auto-numbered-export/）
- [x] 极简创作工作台布局重构（告别冗余侧边栏与纵向参数大杂烩，采用三段式工作台与四分栏按需属性面板，详见 docs/features/clean-studio-layout/）
- [x] 文字与贴纸叠加渲染全链路修复（修复现代 macOS 中文字体探测缺失导致的方框乱码、修复源视频坐标系到输出缩放帧的等比映射与裁切偏移计算、优化文字与贴纸自动启用交互，详见 docs/features/text-sticker-gif-render/）
- [x] 全帧画质精修与色彩调节（支持所有视频帧统一调节亮度、对比度、饱和度、5×5 矩阵锐化、高斯模糊、色相偏移与电影感暗角，内置 5 款一键精修预设并支持画布 SVG 卷积锐化实时预览，详见 docs/features/frame-image-adjustments/）
- [x] 跨平台安装包和自动更新（新增 GitHub Actions Windows .exe/.msi 与 macOS .dmg 自动构建及 Release 发布流水线、Windows 子进程 CREATE_NO_WINDOW 防黑框与字体路径兼容加固、内置 GitHub Releases 远程更新检查与一键下载引导，详见 docs/features/cross-platform-release-updater/）

## 暂不纳入当前范围

- 云端上传和服务端转码
- 用户账号、在线素材市场
- 直接在主进程中嵌入未经评估的 FFmpeg 库
- 在没有许可证和分发策略的情况下打包第三方二进制

## 更新规则

- 大方向变化在本文件记录，并新增或更新对应 feature 文档。
- 可交付的功能不要只写一句 TODO，必须补齐 docs/features/<feature-slug>/ 五件套。
- 完成一项后保留 checkbox 和相关文档链接，方便追溯。
