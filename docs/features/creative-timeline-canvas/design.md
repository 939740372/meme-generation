# 设计文档：可视化时间轴与画布拖拽创作工作区

- 状态：ready-to-merge
- Feature slug：creative-timeline-canvas
- 分支：feature/creative-timeline-canvas
- 负责人：待定
- 日期：2026-10-22

## 1. 方案摘要

本方案在现有单页编辑器架构上，对两个高频创作环节进行交互升级：
1. **时间轴组件（TimelineSlider）**：
   - 放置在预览舞台与媒体摘要之间，利用条形轨道表达时间；
   - 包含背景轨道（灰）、选中高亮区（品牌酸性黄）、左手柄（Start）、右手柄（End）；
   - 基于 `media.durationSecs` 计算百分比：
     - `leftPercent = (startSecs / duration) * 100%`
     - `rightPercent = (endSecs / duration) * 100%`
   - 手柄支持 Pointer / Mouse 拖拽，并限制边界与最小间距（0.1s）。
2. **画布交互图层（CanvasOverlayLayer）**：
   - 预览舞台（`preview-stage`）使用相对定位 `position: relative`，并监听其当前的实际渲染宽高（`rect = stage.getBoundingClientRect()`）；
   - 缩放比 `scaleX = rect.width / media.width`，`scaleY = rect.height / media.height`；
   - 文字与贴图渲染为绝对定位的拖拽块：
     - `cssLeft = textX * scaleX`
     - `cssTop = textY * scaleY`
   - 拖拽过程：
     - `onPointerDown`：记录初始鼠标位置与元素当前物理坐标；
     - `window.onPointerMove`：计算移动增量 `deltaX = (e.clientX - startX) / scaleX`，实时更新 `textX = clamp(originX + deltaX, 0, media.width - elementWidth)`；
     - `window.onPointerUp`：注销全局监听器，完成位置锁定。

## 2. 界面与交互流图

~~~mermaid
flowchart TD
    A["导入视频完成"] --> B["激活可视化时间轴与画布拖拽图层"]
    B --> C["拖拽时间轴滑块手柄"]
    C --> D["双向更新 startSecs / endSecs"]
    B --> E["鼠标按住画布中的文字或贴图"]
    E --> F["PointerMove 实时计算像素坐标偏移"]
    F --> G["双向更新 textX, textY / imageX, imageY"]
    D & G --> H["点击导出 GIF"]
    H --> I["按精确参数构建 RenderPlan 并调用 FFmpeg"]
~~~

## 3. 坐标映射算法

设视频原始宽高为 $W_{orig}, H_{orig}$，画布 DOM 容器尺寸为 $W_{view}, H_{view}$。
- 比例因子：
  $$S_x = \frac{W_{view}}{W_{orig}}, \quad S_y = \frac{H_{view}}{H_{orig}}$$
- 拖拽偏移量计算：
  $$\Delta X_{orig} = \frac{e.clientX - startClientX}{S_x}$$
  $$\Delta Y_{orig} = \frac{e.clientY - startClientY}{S_y}$$
- 边界约束：
  $$X_{new} = \max(0, \min(W_{orig} - W_{elem}, X_{init} + \Delta X_{orig}))$$
  $$Y_{new} = \max(0, \min(H_{orig} - H_{elem}, Y_{init} + \Delta Y_{orig}))$$

## 4. 视觉规范

- **时间轴轨道**：高度 28px，深色胶囊轨道，手柄采用酸性黄主强调色圆角矩形，内嵌抓取竖线；高亮选区采用带微透明度的品牌强调色。
- **拖拽选中态**：在元素外围渲染 1.5px 虚线边框（`outline: 1.5px dashed var(--accent)`），鼠标悬浮为 `cursor: grab`，按住为 `cursor: grabbing`。
- **零额外体积**：纯 CSS 与 Vue 3 组合式 API，无需引入庞大的外部 Canvas 依赖。
