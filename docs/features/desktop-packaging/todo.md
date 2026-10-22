# 任务拆解：macOS DMG 桌面安装包构建与分发验收

- 状态：ready-to-merge
- Feature slug: desktop-packaging
- 分支：feature/desktop-packaging
- 负责人：Antigravity
- 日期：2026-10-22

## 任务执行清单

- [x] 0. 文档初始化与建立基线
  - [x] 创建 `feature/desktop-packaging` 分支
  - [x] 撰写并在 `docs/features/desktop-packaging/` 提交完整需求、设计与实现计划文档
- [x] 1. 运行时二进制解析增强与配置完善 (`src-tauri/src/lib.rs`, `src-tauri/tauri.conf.json`)
  - [x] 1.1 强化 `resolve_binary` 在生产打包 App Bundle 下的候选路径探测逻辑
  - [x] 1.2 编写针对 `resolve_binary` 生产与开发候选路径的单测
  - [x] 1.3 确认 `tauri.conf.json` 中 `bundle` 与 macOS 相关配置
- [x] 2. Release 构建与 DMG 打包
  - [x] 2.1 沉淀一键打包流水线 `scripts/build-dmg.mjs` 与 `pnpm build:dmg`
  - [x] 2.2 验证构建过程无报错，成功生成 `.dmg` 与 `.app` Bundle
- [x] 3. 产物完整性与离线可执行性自动化验收
  - [x] 3.1 编写 `scripts/verify-bundle.mjs` 校验脚本
  - [x] 3.2 验证 `.app/Contents/MacOS` 内 sidecar 二进制存在性、权限与版本参数
  - [x] 3.3 记录最终 DMG 文件体积（62.72 MB）与产物哈希
- [x] 4. 收尾与主分支合入
  - [x] 4.1 更新五件套文档状态为 `ready-to-merge` 并记录自测结果
  - [ ] 4.2 非快进合并至 `main`
  - [ ] 4.3 文档状态变更为 `merged`，同步根目录 `TODO.md`，推送到远程仓库
