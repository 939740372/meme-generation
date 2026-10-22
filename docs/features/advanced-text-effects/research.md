# 需求调研：高级文字排版效果 (advanced-text-effects)

- 状态：merged
- Feature slug: advanced-text-effects
- 分支：feature/advanced-text-effects
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 背景与用户痛点

1. **表情包二创对排版表现力的高阶诉求**：
   - 现有的文字排版仅支持水平居中/对齐、字号、颜色、描边和底框；
   - 现实中海量经典表情包与沙雕梗图依赖极具张力的几何排版：
     - **自由旋转贴合场景**：如将文字沿 45° 倾斜贴在建筑物、斜向指示牌、倒地人物身体或对角线构图上；
     - **弧形弯曲环绕头顶**：如人物头顶上方呈现彩虹状弧形排布的“头顶高能”、“天下第一”、“神仙操作”等文字；
     - **动感斜体 (Italic / Slant)**：表达速度感、鬼畜冲刺感与夸张强调；
     - **立体投射阴影 (Drop Shadow)**：增强文字与多变背景的分离度与立体感。
2. **目前创作者的痛点**：
   - 用户无法在桌面端内一键完成倾斜与弯曲文字，往往需要打开 Photoshop 等庞大软件做好透明 PNG 再导回，创作流程严重割裂。

## 2. 调研结论与技术选型

| 调研项 | 方案对比 | 决策依据 |
|--------|----------|----------|
| **自由角度旋转** | 原生 `rotate` 滤镜 vs 导出图片覆盖层 | **选用 FFmpeg 原生 `color=c=black@0.0` + `drawtext` + `rotate=...:c=none` + `overlay=cx-w/2:cy-h/2`**：无需生成本地临时文件，内存极速渲染，支持任意 -180°~180° 旋转，中心对齐精准 |
| **文字立体阴影** | 绘制双层文字 vs 原生 `shadowcolor`/`shadowx`/`shadowy` | **选用 `drawtext` 原生 `shadowcolor` + `shadowx` + `shadowy`**：FFmpeg 内部高效单 pass 渲染，与前端 CSS `text-shadow` 像素级一致 |
| **文字斜体动感** | 依赖特定斜体字体文件 vs `shear=shx=-0.22:c=none` 错切 | **选用 FFmpeg 原生 `shear` 滤镜配合前端 CSS `skewX` / `font-style: italic`**：彻底解决中文字体（如苹方、微软雅黑）普遍缺少内置 Oblique/Italic 字形的通病，任何系统字体均可拥有标准 12° 倾角斜体效果 |
| **弧形弯曲 (Arc/Curve)** | 字符级坐标与角度三角分解 vs 图片烘焙 | **选用字符级几何分解算法**：根据弧度角 $\alpha$ 将字符串各字符按弧度半径 $R$ 分布到圆形拱道 $(x_k, y_k)$ 并施加相应切线旋转角 $\theta_k$；前端 Canvas 与 Rust FFmpeg 滤镜图共享同一套数学参数模型，所见即所得 |

## 3. 数学建模

### 3.1 旋转中心对齐模型
设文字层未旋转时中心为 $(cx, cy) = (x + w/2, y + h/2)$，旋转角为 $\theta$（弧度制）。
在 FFmpeg 中：
- 透明文字画板尺寸：$W_{pad} \times H_{pad}$，文字居中绘制 `x=(w-text_w)/2:y=(h-text_h)/2`；
- 旋转滤镜：`rotate=θ:c=none:ow=rotw(θ):oh=roth(θ)`；
- 叠加滤镜：`overlay=${cx}-w/2:${cy}-h/2:eof_action=pass`。
旋转完全围绕文字自身中心展开，坐标与用户在画布上直接拖拽的视觉中心严密重合。

### 3.2 弧形弯曲模型
设文本字符数为 $N$，总弯曲角度为 $\alpha$（度数，正数向上拱起凸弧，负数向下凹陷凹弧）。
对于索引 $k \in [0, N-1]$：
- 归一化位置：$t_k = \frac{k - (N-1)/2.0}{(N-1)/2.0}$
- 偏转角度：$\theta_k = t_k \times (\alpha / 2.0)$
- 弧线半径：$R = \frac{\text{text\_width}}{\alpha \cdot \pi / 180}$
- 径向偏移：$\Delta x_k = R \cdot \sin(\theta_k \cdot \pi / 180)$，$\Delta y_k = -R \cdot (1 - \cos(\theta_k \cdot \pi / 180))$
- 字符自身朝向角：$\text{angle}_k = \text{rotation} + \theta_k$。

## 4. 风险与未知项

- **性能开销**：当弯曲字符较多时，字符级 pad 分解会增加滤镜节点数。
  - 防御策略：当 `arc_radius == 0`（绝大多数场景）时，完全走单节点快速通道，零额外开销；即使弯曲字符也仅在导出阶段组装，实测 4~8 个字符转码时间仍在数十毫秒内。
- **向下兼容性**：
  - `rotation`, `shadow_offset`, `shadow_color`, `is_italic`, `arc_radius` 均设为 `Option<...>`，默认 `None` 时行为完全等价于现有逻辑。
