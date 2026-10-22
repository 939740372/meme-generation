# 视频导入与画面实时预览实现计划 (Implementation)

## 1. 文件级变更范围

| 文件 | 变更内容 |
|---|---|
| `src-tauri/tauri.conf.json` | 开启 `app.security.assetProtocol`，配置 scope 允许访问本地视频素材 |
| `src-tauri/src/lib.rs` | 新增 `extract_video_frame` 快速单帧抽取命令，注册到 invoke handler，并添加单元测试 |
| `src/lib/desktop.ts` | 导出 `extractVideoFrame` 和 `getVideoAssetUrl` 接口 |
| `src/App.vue` | 改造预览画布底图架构，集成双模 `<video>` 与 `extractedFrameUrl`，实现片段循环播放控件与防抖抽帧联动 |
| `src/styles.css` | 补充预览播放控制条、画面自适应容器、视频/静帧切换动效与降级提示样式 |
| `scripts/smoke-test-video-preview.mjs` | 端到端冒烟测试脚本：验证单帧极速提取、时间定位准确性与边界容错 |

---

## 2. 详细执行步骤

1. **Step 1: 配置 Tauri 2 `assetProtocol`**
   - 在 `tauri.conf.json` 开启 `assetProtocol` 权限。
2. **Step 2: Rust 端实现 `extract_video_frame`**
   - 提取参数并校验文件存在性；
   - 构建 `ffmpeg -ss <secs> -i <path> -frames:v 1 -vf scale='min(640,iw)':-2 -f image2pipe -vcodec mjpeg -`；
   - 捕获 stdout 并通过 base64 编码为 `data:image/jpeg;base64,...`；
   - 编写单元测试并注册到 `generate_handler!`。
3. **Step 3: 前端接口与响应式状态集成**
   - 在 `desktop.ts` 中封装 API；
   - 在 `App.vue` 中导入视频时，并行触发首帧抽取与播放源配置；
   - 绑定循环播放控制、时间轴拖动防抖抽帧更新。
4. **Step 4: 画布结构与样式优化**
   - 替换原有抽象光球为真画面；
   - 保证文字、贴纸、裁切框在视频画面上方准确显示；
   - 添加播放/暂停控制栏。
5. **Step 5: 编写测试脚本与全量门槛检查**
   - 运行冒烟测试、单元测试、前端构建检查。

---

## 3. 自测验证记录

- **前端构建与类型检查**：`pnpm build`（`vue-tsc --noEmit` + `vite build`）通过，0 警告 0 报错；
- **Rust 单元测试**：`cargo test` 34 项测试全部通过（含 `extract_video_frame` 缺失文件与非法时间测试）；
- **Rust 依赖锁定检查**：`cargo check --manifest-path src-tauri/Cargo.toml --locked` 正常通过；
- **端到端冒烟测试**：`node scripts/smoke-test-video-preview.mjs` 通过：
  - 验证了首帧 0.0s 极速提取（44ms 内存流读取）；
  - 验证了特定时间点 1.8s 精准寻道提取；
  - 验证了 MKV 非浏览器原生格式的 100% 抽帧降级兼容性；
  - 验证了 `tauri.conf.json` 中 `assetProtocol` 的生效配置。
- **样式与格式检查**：`git diff --check` 无残留空格或格式问题。
