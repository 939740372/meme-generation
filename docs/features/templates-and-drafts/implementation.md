# 实现计划与自测：常用表情包模板与草稿暂存

- 状态：proposed
- Feature slug: templates-and-drafts
- 分支：feature/templates-and-drafts
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 改动清单

| 模块 | 文件 | 变动说明 |
|------|------|----------|
| 数据与类型 | `src/lib/templates.ts` | 新建模板模块：定义 `MemeTemplate`、3 款内置模板常量、自定义模板本地增删改查逻辑 |
| 前端 / API | `src/lib/desktop.ts` | 增加草稿数据类型 `StudioDraft` 定义与通用校验辅助方法 |
| 前端 / 界面 | `src/App.vue` | 1. 自动草稿持久化监听（防抖 400ms）与启动自动水合恢复<br>2. 抽屉增加双 Tab（导出历史 / 表情包模板）<br>3. 文字控制区增加“存为模板”与“套用模板”交互<br>4. 增加“重置工作区/清空草稿”动作 |
| 前端 / 样式 | `src/styles.css` | 抽屉 Tab 切换、模板卡片布局、小药丸徽章与弹窗样式 |
| 文档 | `docs/features/templates-and-drafts/*` | 调研、需求、设计、实现与任务清单维护 |

## 2. 迁移与兼容策略

- **草稿数据**：带 `version: 1` 标识，读取时若反序列化失败或缺少关键字段，优雅 catch 并静默忽略，不阻塞应用主渲染；
- **模板向后兼容**：内置模板为静态内存常量，永不丢失；自定义模板保存在 `localStorage`，独立存储键 `meme_studio_custom_templates`。

## 3. 自测验证方案

### 3.1 自动化检查
```bash
# 1. 前端类型检查与打包
pnpm build

# 2. Rust 编译与单元测试保持正常
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml

# 3. 提交物与空白符检查
git diff --check
```

### 3.2 交互验证 (Smoke Test)
1. **草稿恢复测试**：
   - 导入视频，裁剪时间 1.0s~3.0s，开启文字“测试台词”，开启黑底字幕条；
   - 刷新页面（或模拟重启），验证所有参数和视频媒体信息自动恢复，弹出“已恢复上次未完成的草稿”提示；
   - 点击“重置工作区”，验证参数重置且草稿被清空。
2. **内置模板套用测试**：
   - 分别点击“经典影视字幕条”、“顶部大字吐槽”、“名场面大字报”；
   - 检查预览画布所见即所得更新（文字字号、描边、底框）。
3. **自定义模板测试**：
   - 调整一套自定义参数，点击“存为模板”，输入名称“我的测试模板”；
   - 验证模板列表中出现该模板；
   - 导入其他视频后点击套用，验证参数恢复；
   - 删除该自定义模板，验证列表更新。

## 4. 自测记录

- 平台环境：macOS (Darwin aarch64 - Apple Silicon M-series)
- 执行记录：待实现完成后填充
