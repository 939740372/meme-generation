# 任务清单：自定义背景音乐与音效混流 (bgm-audio-mixing)

- 状态：merged
- Feature slug: bgm-audio-mixing
- 分支：feature/bgm-audio-mixing
- 负责人：Antigravity
- 日期：2026-10-22

## 任务拆解与进度

### 阶段一：文档与素材基建

- [x] 完成 `docs/features/bgm-audio-mixing/research.md` 需求调研
- [x] 完成 `docs/features/bgm-audio-mixing/requirements.md` 需求说明
- [x] 完成 `docs/features/bgm-audio-mixing/design.md` 方案设计
- [x] 完成 `docs/features/bgm-audio-mixing/implementation.md` 实现计划
- [x] 完成 `docs/features/bgm-audio-mixing/todo.md` 任务清单
- [x] 编写 `scripts/generate-sfx.mjs`，在 `src-tauri/sfx/` 生成 8 款内置音效文件
- [x] 在 `src-tauri/tauri.conf.json` 中配置 `sfx` 资源打包
- [x] 提交规范 commit 建立 feature 基线

### 阶段二：Rust 后端混流与素材加载

- [x] 在 `src-tauri/src/lib.rs` 中定义 `BgmOverlay` 与 `BuiltinSfx`
- [x] 扩展 `MediaInfo`，返回 `has_audio: bool`
- [x] 扩展 `RenderPlan`，支持 `has_audio` 与 `bgm_overlay: Option<BgmOverlay>`
- [x] 扩展 `read_file_as_data_url` 支持音频 mime（mp3/wav/ogg/aac/m4a）
- [x] 实现 `resolve_sfx_dir` 与 `get_builtin_sfx` tauri command
- [x] 更新 `build_ffmpeg_args_with_hw`：
  - [x] 动态计算 BGM 输入流索引（无贴纸=1，有贴纸=2）
  - [x] 构建原声与 BGM 的双路 `amix` 混音滤镜链
  - [x] 兼容源视频无声/原声音量为 0 的纯 BGM 模式（避免引用 `[0:a]`）
  - [x] 兼容变速滤镜（atempo）与时长裁切（atrim）
- [x] 编写 Rust 单元测试并全部跑通

### 阶段三：前端数据模型与界面交互

- [x] 在 `src/lib/desktop.ts` 导出 `BgmOverlay`, `BuiltinSfx`, `getBuiltinSfx`, `pickAudioFile`
- [x] 在 `src/lib/templates.ts` 与 `src/lib/queue.ts` 中同步支持 `bgmOverlay`
- [x] 在 `src/App.vue` 抽屉中新增“🎵 音效库”Tab，支持 8 款音效卡片网格浏览与点击试听
- [x] 在 `src/App.vue` 控制面板中新增 BGM / 音效配置区块：
  - [x] 展示当前已选音频信息与清除按钮
  - [x] 本地音频文件选择按钮
  - [x] 原声音量滑块（0% ~ 200%）与 BGM 音量滑块（0% ~ 200%）
- [x] 在 `src/styles.css` 中添加音效卡片、试听状态指示器和音量滑块样式

### 阶段四：验证、自测与质量门槛

- [x] 编写端到端冒烟测试脚本 `scripts/smoke-test-bgm-mixing.mjs`
- [x] 运行冒烟测试，通过 ffprobe 严格检验视频音轨与混音时长
- [x] 执行前端类型检查与打包：`pnpm build`
- [x] 执行 Rust 测试与检查：`cargo test` 与 `cargo check --locked`
- [x] 执行格式与代码差异检查：`git diff --check`
- [x] 更新 `implementation.md` 自测记录与文档状态为 `ready-to-merge`

### 阶段五：合并与主分支同步

- [x] 非快进合并至 `main` 分支
- [x] 更新根目录 `TODO.md`
- [x] 推送 `main` 至远程 `origin main`
- [x] 清理本地分支 `feature/bgm-audio-mixing`
- [ ] 清理本地分支 `feature/bgm-audio-mixing`
