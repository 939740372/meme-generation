# 实现计划与交付记录：最近导出历史与快捷操作

- 状态：merged
- Feature slug: export-history
- 分支：feature/export-history
- 负责人：待定
- 日期：2026-10-22

## 1. 文件级实现计划

| 文件 | 变更类型 | 说明 |
| --- | --- | --- |
| `src-tauri/Cargo.toml` | 修改 | 添加 `base64 = "0.22"` 依赖用于本地图片 Base64 Data URL 编码 |
| `src-tauri/src/lib.rs` | 修改 | 新增 `copy_file_to_clipboard` 与 `read_file_as_data_url` 两个 command 并注册到 `invoke_handler`；增加对应单元测试 |
| `src/lib/desktop.ts` | 修改 | 引入并导出 `revealItemInDir`，新增 `copyFileToClipboard`、`readFileAsDataUrl` 与 `ExportRecord` 类型定义 |
| `src/App.vue` | 修改 | 新增导出历史响应式状态（`historyList`）、LocalStorage 同步、Toast 反馈逻辑、侧边栏历史卡片、底部滑出式抽屉面板与导出栏快捷按钮 |
| `src/styles.css` | 修改 | 新增侧边栏历史模块、底部抽屉动效与卡片样式、Toast 浮层样式，保持 1280×820 固定视口无滚动条 |
| `docs/features/export-history/*` | 新增 | 全套 feature 文档与测试自测记录 |

## 2. 兼容性与安全策略

1. **依赖安全性**：
   - 剪贴板调用通过参数化传递，过滤单双引号，避免 shell 命令注入；
   - 本地读取仅限于合法的普通文件，限制体积 < 15MB，防范超大文件引发内存抖动。
2. **渐进增强**：
   - 网页预览模式（无 Tauri 环境）下，自动降级为 mock 数据或提示桌面模式；
   - 剪贴板复制失败时自动 fallback 到 `navigator.clipboard.writeText` 写入文件路径。

## 3. 测试与自测方案

### 3.1 自动化检查
1. 前端类型检查与构建：
   ```bash
   pnpm build
   ```
2. Rust 编译与单元测试：
   ```bash
   PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked
   PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml
   ```
3. 代码风格与工作区检查：
   ```bash
   git diff --check
   ```

### 3.2 交互与冒烟验证项
- [x] 导出完成后，历史记录自动增加，侧边栏显示最新卡片与体积；
- [x] 点击「在访达中显示」，macOS 访达成功打开并高亮选中该 GIF；
- [x] 点击「复制文件」，弹出 Toast 提示，且在剪贴板中成功获取文件内容与路径；
- [x] 点击「载入参数」，工作区的起止秒数、输出尺寸、预设、文字和贴图完整回填；
- [x] 展开底部抽屉，卡片流平滑展示缩略图，支持单条删除和全部清空；
- [x] 验证 `1280×820` 固定视口，无页面全局或面板局部溢出滚动条。

## 4. 自测记录

- 执行时间：2026-10-22 11:30
- 平台环境：macOS (Darwin 24.3.0, Apple Silicon aarch64)
- 测试命令与输出：
  1. `pnpm build`:
     `$ vue-tsc --noEmit && vite build` -> `✓ built in 564ms`，前端类型与产物打包全部通过；
  2. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked`:
     `Finished dev profile in 0.73s`，Rust 代码编译无任何警告；
  3. `PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml`:
     `8 passed; 0 failed; 0 ignored; finished in 0.00s`，包含 `copy_file_to_clipboard_fails_on_missing_file`、`read_file_as_data_url_fails_on_missing_file`、`read_file_as_data_url_encodes_valid_file` 均全部通过；
  4. `git diff --check`:
     无空白与换行错误。
- 验收结论：各项测试与门槛均已通过，达到 ready-to-merge 标准。
