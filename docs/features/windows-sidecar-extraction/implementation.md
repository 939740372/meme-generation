# 实施：Windows sidecar 解压修复

- 状态：verifying
- 修改 scripts/download-ffmpeg-sidecar.mjs 的 Windows 解压分支；不修改下载源和产品运行时。
- 依赖 Windows GitHub runner 自带 tar，不引入依赖；兼容性以 Actions 实际运行确认。
- 检查 pnpm install、pnpm build、cargo check --locked、git diff --check，并通过 Actions 构建双平台。
- 回滚：还原脚本中 Windows 解压分支；没有数据迁移。

## 自测记录

- macOS arm64：`pnpm install --frozen-lockfile`、`pnpm build`、`cargo check --locked`、`node --check scripts/download-ffmpeg-sidecar.mjs` 与 `git diff --check` 通过。安装后的 pnpm 版本提示联网失败，不影响安装退出码。
- GitHub Actions 运行 45094346421（提交 ce5593d）：Windows x64 成功下载、解压并校验 sidecar，NSIS 安装包已上传；macOS arm64 DMG 构建并上传成功。
- Windows 安装与 GUI 冒烟未执行：当前环境没有 Windows 桌面。macOS GUI 冒烟此前在 2026-11-16 对同一应用代码的本机正式包完成进程启动检查，未覆盖本次 Actions DMG 的安装流程。
