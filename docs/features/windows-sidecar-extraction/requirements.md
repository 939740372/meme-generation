# 需求：Windows sidecar 解压修复

- 状态：designed
- 保持现有 FFmpeg 下载来源、命名、能力验证和临时目录清理；Windows 上能够从 ZIP 提取 ffmpeg 与 ffprobe，解压失败应立即报错。Windows 安装包和 macOS DMG 构建均成功并上传为 Actions artifacts。
