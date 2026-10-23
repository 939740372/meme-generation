# 跨平台自动化构建、Windows 加固与远程更新检查任务清单 (TODO)

- **状态**: merged
- **分支**: `feature/cross-platform-release-updater`

## 任务拆解

- [x] 完成 `docs/features/cross-platform-release-updater/` 五件套文档设计
- [x] 在 `src-tauri/src/lib.rs` 中实现 `new_hidden_command`（Windows `CREATE_NO_WINDOW`）及 Windows 字体路径正斜杠化与单测
- [x] 优化 `scripts/download-ffmpeg-sidecar.mjs` 支持 Windows / macOS / Linux 本机架构自动校验 `drawtext`
- [x] 新增 `.github/workflows/build-release.yml` 支持 Windows `.exe`/`.msi` 与 macOS `.dmg` 自动构建、产物上传与 GitHub Release 发布
- [x] 在 `src/lib/desktop.ts`、`src/App.vue` 与 `src/styles.css` 中实现 GitHub Releases 远程更新检查与一键下载弹窗
- [x] 执行 `pnpm build`、`cargo check`、`cargo test` 自测并更新 `TODO.md`，合并推送至 `origin/main`
