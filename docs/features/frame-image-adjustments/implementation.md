# 全帧画质精修与色彩滤镜实现与测试文档 (Implementation)

- **状态**: merged
- **分支**: `feature/frame-image-adjustments`
- **日期**: 2026-10-23

## 1. 修改文件清单

- `src-tauri/src/lib.rs`：
  - 新增 `ImageAdjustments` 结构体及 `build_image_adjustments_filter()` 转换函数。
  - 在 `RenderPlan` 中新增 `image_adjustments` 字段。
  - 在 `validate_render_plan()` 中校验亮度、对比度、饱和度（`-100..=100`）、锐化、模糊（`0..=100`）和色相（`-180..=180`）范围。
  - 在 `build_filter_graph()` 的标准与高级分支中统一注入全帧画质调节滤镜链。
  - 新增单元测试覆盖 `eq`、`unsharp`、`gblur`、`hue`、`vignette` 滤镜生成与参数校验。
- `src/lib/desktop.ts` & `src/lib/templates.ts`：
  - 新增 `ImageAdjustments` 接口，并在 `RenderPlan` 与 `StudioDraft` 中增加 `imageAdjustments` 字段。
- `src/App.vue` & `src/styles.css`：
  - 在「📐 画面」检查器标签页中新增「✨ 全帧画质与色彩精修」卡片，包含 6 款一键精修预设（默认原画、高清锐化、提亮去暗、浓郁鲜艳、柔焦模糊、赛博冷调）以及亮度、对比度、饱和度、锐化、模糊、色相滑块与暗角开关。
  - 在画布中通过 CSS `filter` + 动态 SVG `<feConvolveMatrix>` 锐化滤镜 + 径向暗角层实现全帧实时预览。
  - 支持草稿自动保存恢复、历史记录恢复与工作区重置。

## 2. 迁移与兼容性策略

- `image_adjustments` 为可选字段（`#[serde(default)]`），未设置或全为默认值时不向 FFmpeg 注入额外滤镜，零性能损耗且 100% 兼容旧版数据。

## 3. 自测记录

- **平台信息**：macOS (aarch64-apple-darwin)
- **测试命令与结果**：
  1. `pnpm install && pnpm build`
     - **预期结果**：`vue-tsc --noEmit` 类型检查无报错，Vite 构建成功。
     - **实际结果**：通过（`dist/assets/index-DOdqkMXS.js 183.78 kB`）。
  2. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`
     - **预期结果**：Rust 编译检查通过。
     - **实际结果**：通过（`Finished dev profile in 2.86s`）。
  3. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml`
     - **预期结果**：包含新增 `builds_image_adjustments_filter_chain_for_all_frames` 在内的全部单元测试通过。
     - **实际结果**：通过（`40 passed; 0 failed`）。
  4. **真实 FFmpeg 全帧画质精修与 GIF 导出 Smoke Test**：
     - 命令：`src-tauri/binaries/ffmpeg-aarch64-apple-darwin -hide_banner -loglevel error -f lavfi -i testsrc=duration=1.5:size=640x360:rate=12 -filter_complex "[0:v]scale=320:-1:flags=lanczos,fps=12,eq=brightness=0.125:contrast=1.20:saturation=1.40,hue=h=-25,unsharp=5:5:1.50:5:5:0.0,vignette=PI/4[base];[base]split[gif][palette];[palette]palettegen=stats_mode=diff[p];[gif][p]paletteuse=dither=sierra2_4a[vout]" -map "[vout]" -y /tmp/meme_adj_smoke.gif`
     - **预期结果**：内置 sidecar FFmpeg 对所有视频帧执行提亮、对比度增强、饱和度增强、色相偏移、5×5 矩阵锐化与光学暗角滤镜，并成功通过双通道调色板生成 GIF。
     - **实际结果**：成功输出 `/tmp/meme_adj_smoke.gif`（331 KB），验证后已清理临时文件。
- **已知限制**：
  - 画布预览使用 CSS `filter` + 动态 SVG `<feConvolveMatrix>` 近似模拟 FFmpeg 的 `eq` 与 `unsharp` 效果，最终导出以 FFmpeg 滤镜链逐帧渲染为准。
