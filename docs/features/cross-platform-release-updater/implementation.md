# 跨平台自动化构建、Windows 加固与远程更新检查实现与测试文档 (Implementation)

- **状态**: merged
- **分支**: `feature/cross-platform-release-updater`
- **日期**: 2026-10-23

## 1. 变更文件清单

- `.github/workflows/build-release.yml`：新增 Windows (`.exe` / `.msi`) 与 macOS (`.dmg`) 双端自动化构建与 GitHub Release 发布工作流。
- `scripts/download-ffmpeg-sidecar.mjs`：放开全平台（Windows / macOS / Linux）同架构下的 FFmpeg 二进制版本与 `drawtext` 滤镜校验。
- `src-tauri/src/lib.rs`：
  - 新增 `new_hidden_command()`，在 Windows 下自动注入 `CREATE_NO_WINDOW (0x08000000)` 防止黑框闪现。
  - 升级 `detect_system_font()` 的 Windows 中文字体候选列表并统一为 `/` 分隔符，在 `path_to_string()` 中剥离 Windows `\\?\` 扩展路径前缀，新增 `windows_font_path_and_unc_prefix_compatibility` 单元测试。
- `src/lib/desktop.ts`：
  - 新增 `UpdateInfo`、`compareSemver()`、`checkGitHubUpdate()` 与 `openExternalUrl()`。
- `src/App.vue` & `src/styles.css`：
  - 顶栏新增版本与更新检查按钮（支持红点提示新版本），新增更新详情模态框与一键下载跳转。

## 2. 迁移与回滚策略

- 所有改动向后兼容；若网络离线或 GitHub API 尚未创建 Release，后台静默检查不会打断用户操作，手动检查会给出清晰的当前版本状态提示。

## 3. 自测记录

- **平台信息**：macOS (aarch64-apple-darwin)
- **测试命令与结果**：
  1. `pnpm install && pnpm build`
     - **预期结果**：`vue-tsc --noEmit` 类型检查通过，Vite 构建成功。
     - **实际结果**：通过（`dist/assets/index-BTLjsSlL.js 189.33 kB`）。
  2. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`
     - **预期结果**：Rust 编译检查通过且零警告。
     - **实际结果**：通过（`Finished dev profile in 2.11s`）。
  3. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml`
     - **预期结果**：包含新增 `windows_font_path_and_unc_prefix_compatibility` 在内的全部单元测试通过。
     - **实际结果**：通过（`41 passed; 0 failed`）。
- **已知限制**：
  - GitHub Actions 首次运行需推送到远程仓库后在 GitHub 页面点击 `Run workflow` 或推送 `v*` 标签触发。
