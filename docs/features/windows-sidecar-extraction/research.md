# Windows sidecar 解压修复调研

- 状态：verifying
- 2026-11-23：Actions 运行 45094025239 在 Windows 准备 FFmpeg 时，Windows PowerShell 的 Expand-Archive 报 ExtractToDirectory 重载错误。下载已完成，解压未完成。现有 Windows runner 提供 bsdtar；改用 tar 解压 ZIP，保留原有下载来源与二进制校验。Windows 实际验证由 Actions 完成。
