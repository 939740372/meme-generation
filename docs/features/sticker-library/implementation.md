# 实现计划与自测：内置透明表情贴纸库 (sticker-library)

- 状态：ready-to-merge
- Feature slug: sticker-library
- 分支：feature/sticker-library
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 改动清单

| 模块 | 文件 | 变动说明 |
|------|------|----------|
| 贴纸素材资源 | `src-tauri/stickers/*.png` | 18 款标准 RGBA 透明 PNG 贴纸矢量绘制与生成脚本 `scripts/generate-stickers.mjs` |
| 打包配置 | `src-tauri/tauri.conf.json` | `bundle.resources` 注册 `"stickers/*"` |
| Rust / Tauri | `src-tauri/src/lib.rs` | 1. 定义 `BuiltinSticker` 数据模型<br>2. 实现 `resolve_stickers_dir` 跨平台路径解析服务<br>3. 实现 `get_builtin_stickers` Tauri command<br>4. 新增 Rust 单元测试 `resolves_stickers_dir_and_loads_builtin_stickers` |
| 前端 / API | `src/lib/desktop.ts` | 增加 `BuiltinSticker` 接口与 `getBuiltinStickers()` 封装调用 |
| 前端 / 界面 | `src/App.vue` | 1. 抽屉集成 `stickers` 标签页与分类筛选网格<br>2. 点击贴纸一键置入画布中心<br>3. 画布内贴纸真图预览展示与右下角手柄缩放交互 (`startResizeImage`)<br>4. 侧边栏贴纸快速操作区与尺寸比例调节 |
| 前端 / 样式 | `src/styles.css` | 贴纸抽屉网格、透明棋盘格背景、缩放手柄 `⤡` 与指示标签样式 |
| 测试与脚本 | `scripts/test-stickers.mjs` | 自动化校验 18 款贴纸文件的存在性、PNG 魔数、尺寸与透明度通道 |
| 文档 | `docs/features/sticker-library/*` | 维护需求、设计、实现与任务清单 |

## 2. 迁移与兼容策略

- 向后兼容：保留用户自行选择外部 PNG 的能力，外部 PNG 与内置贴纸走同一套 `ImageOverlay` FFmpeg 合成通道；
- 草稿升级：草稿继续保存 `imagePath`，当其为内置贴纸路径时，启动水合时自动恢复其缩略图预览。

## 3. 自测验证方案

### 3.1 自动化检查
```bash
# 1. 贴纸完整性校验
node scripts/test-stickers.mjs

# 2. 前端类型检查与打包
pnpm build

# 3. Rust 编译与单元测试
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml

# 4. 代码规范检查
git diff --check
```

### 3.2 真实功能验收 (Smoke Test)
1. **抽屉打开与分类浏览**：
   - 点击侧边栏“🎨 贴纸素材库”，抽屉平滑展开到贴纸选项卡；
   - 切换“全部”、“经典梗图”、“情绪表态”、“对话气泡”分类，卡片筛选正常；
2. **画布置入与移动**：
   - 点击“暴徒墨镜”，画布中央立即显示高清透明墨镜图像；
   - 鼠标按住墨镜中心拖动，墨镜平滑跟随移动；
3. **右下角缩放交互**：
   - 按住墨镜右下角缩放手柄拖拽，墨镜等比例平滑放大/缩小；
   - 检查侧边栏 `Width` 数字同步变化；
4. **真实动图合成测试**：
   - 导出为 GIF 与 WebP，确认输出文件中墨镜完整合成且背景完全透明；
   - 关闭应用后重新打开，检查上次选用的贴纸与位置尺寸完整水合还原。

## 4. 自测记录

- 平台环境：macOS (Darwin aarch64 - Apple Silicon M-series)
- 执行记录：
  1. `node scripts/test-stickers.mjs`: 18 款贴纸全部校验通过（PNG 签名合法，IHDR 256×256，ColorType=6 RGBA 透明通道完全具备）；
  2. `pnpm build`: 通过，`vue-tsc` 类型检查无报错，Vite 打包耗时 841ms；
  3. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`: 检查通过，lockfile 锁定；
  4. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml`: 13 个单测全绿通过（含 `resolves_stickers_dir_and_loads_builtin_stickers`）；
  5. 真实 FFmpeg 贴纸动图合成 Smoke Test：
     - GIF: overlay `deal_with_it.png` 生成 `/tmp/test_sticker_deal_with_it.gif` (60KB, 18.3x 转码速度)；
     - WebP: overlay `doge_face.png` 生成 `/tmp/test_sticker_doge.webp` (52KB, 14.9x 转码速度)；
  6. `git diff --check`: 0 告警，无空白符与格式问题。
