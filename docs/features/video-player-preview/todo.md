# 视频导入与画面实时预览任务清单 (TODO)

## 状态：merged

- [x] Step 1: 在 `src-tauri/tauri.conf.json` 配置 `assetProtocol: { enable: true, scope: ["**"] }`
- [x] Step 2: 在 `src-tauri/src/lib.rs` 实现 `extract_video_frame` 命令并编写单元测试
- [x] Step 3: 在 `src/lib/desktop.ts` 暴露 `extractVideoFrame` 与 `getVideoAssetUrl` API
- [x] Step 4: 在 `src/App.vue` 实现画布双模视频与帧预览、片段循环播放控制条与时间轴联动
- [x] Step 5: 在 `src/styles.css` 完善播放控制浮条、视频画面贴合与优雅过渡样式
- [x] Step 6: 编写并执行 `scripts/smoke-test-video-preview.mjs`
- [x] Step 7: 执行 `pnpm build` 与 `cargo check --locked` 并通过所有质量门槛
- [x] Step 8: 合入 `main` 分支并推送到远程
