# 实施：Windows sidecar 解压修复

- 状态：designed
- 修改 scripts/download-ffmpeg-sidecar.mjs 的 Windows 解压分支；不修改下载源和产品运行时。
- 依赖 Windows GitHub runner 自带 tar，不引入依赖；兼容性以 Actions 实际运行确认。
- 检查 pnpm install、pnpm build、cargo check --locked、git diff --check，并通过 Actions 构建双平台。
- 回滚：还原脚本中 Windows 解压分支；没有数据迁移。
