# 全帧画质精修与色彩滤镜需求文档 (Requirements)

- **状态**: merged
- **分支**: `feature/frame-image-adjustments`
- **日期**: 2026-10-23

## 1. 目标与范围

为用户提供针对视频每一帧的画质增强与色彩调节能力（包括亮度、对比度、饱和度、锐化、高斯模糊、色相偏移、电影暗角及一键精修预设），实现画布实时预览与 FFmpeg 全格式导出一致。

## 2. 用户故事

1. **视频提亮与清晰度锐化**：作为用户，当导入的视频偏暗或有些模糊时，我可以一键点击「✨ 高清锐化」或拖动「亮度」「锐化」滑块，让导出的 GIF 每一帧都清晰明亮。
2. **多维色彩与特效组合**：作为用户，我可以自由组合亮度、对比度、饱和度、色相、模糊和暗角，并与黑白/复古/2x加速等画风特效叠加使用。
3. **实时预览与参数持久化**：作为用户，拖动任何画质滑块时画布立即响应，且参数自动随草稿、历史记录和批量队列保存恢复。

## 3. 功能需求

- **FR-1 (画质与色彩参数模型 `ImageAdjustments`)**：
  - `brightness`: `-100 .. 100`（默认 `0`）
  - `contrast`: `-100 .. 100`（默认 `0`）
  - `saturation`: `-100 .. 100`（默认 `0`）
  - `sharpen`: `0 .. 100`（默认 `0`）
  - `blur`: `0 .. 100`（默认 `0`）
  - `hue`: `-180 .. 180`（默认 `0`）
  - `vignette`: `boolean`（默认 `false`）
- **FR-2 (FFmpeg 全帧滤镜链构建)**：
  - 当 `brightness`、`contrast` 或 `saturation` 非 0 时，生成 `eq=brightness=...:contrast=...:saturation=...`。
  - 当 `hue` 非 0 时，生成 `hue=h=...`。
  - 当 `sharpen > 0` 时，生成 `unsharp=5:5:{amount}:5:5:0.0`（`amount` 映射至 `0.05 .. 2.50`）。
  - 当 `blur > 0` 时，生成 `gblur=sigma={sigma}`（`sigma` 映射至 `0.20 .. 10.00`）。
  - 当 `vignette == true` 时，生成 `vignette=PI/4`。
- **FR-3 (一键精修预设)**：
  - 内置「默认原画」「高清锐化」「提亮去暗」「浓郁鲜艳」「柔焦模糊」「赛博冷调」快捷预设，并支持「重置」及单项双击归零。
- **FR-4 (画布实时预览)**：
  - 结合 CSS `filter`、动态 SVG `<feConvolveMatrix>` 锐化滤镜与径向暗角层，在画布上实时呈现所有调节效果。

## 4. 验收标准 (Acceptance Criteria)

- **AC-1**：设置亮度、对比度、饱和度、锐化、模糊、色相、暗角后导出 GIF / WebP / AVIF / MP4，FFmpeg 滤镜图正确包含对应的 `eq`、`unsharp`、`gblur`、`hue`、`vignette` 节点。
- **AC-2**：使用真实 FFmpeg 导出开启「亮度 + 锐化」的 GIF 并抽帧对比，输出帧亮度与边缘锐度显著提升。
- **AC-3**：非法参数范围（如超出 `-100..100` 或 `0..100`）能被 `validate_render_plan` 拦截并返回明确错误码。
- **AC-4**：`pnpm build`、`cargo check --locked`、`cargo test` 与 `git diff --check` 全部通过。
