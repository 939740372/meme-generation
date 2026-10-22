# 实现计划与自测：macOS DMG 桌面安装包构建与分发验收

- 状态：proposed
- Feature slug: desktop-packaging
- 分支：feature/desktop-packaging
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 改动清单

| 模块 | 文件 | 变动说明 |
|------|------|----------|
| Rust / Tauri | `src-tauri/src/lib.rs` | 增强 `resolve_binary`：支持生产打包 App Bundle 下针对 `externalBin` 的多级相对路径安全解析与单元测试 |
| Tauri 配置 | `src-tauri/tauri.conf.json` | 确认 `bundle.externalBin`、`bundle.targets` 与 macOS 配置 |
| 构建脚本 | `package.json` | 增加方便调用的打包命令（或确认 `pnpm tauri:build`） |
| 验收脚本 | `scripts/verify-bundle.mjs` | 自动化验收脚本：检查生成的 App Bundle 内部 sidecar 完整性与可执行权限 |
| 文档 | `docs/features/desktop-packaging/*` | 调研、需求、设计、实现与任务清单维护 |

## 2. 迁移与兼容策略

- 本特性不修改任何既有 API 协议与 UI 行为，仅增强运行时二进制寻址鲁棒性与发布构建流程；
- 向后兼容已有的开发模式与测试用例。

## 3. 自测验证方案

### 3.1 自动化检查
```bash
# 1. 前端类型检查与打包
pnpm build

# 2. Rust 编译与单元测试保持正常
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo test --manifest-path src-tauri/Cargo.toml

# 3. 完整 Release 构建与打包
PATH=/Users/ryantorres/.cargo/bin:$PATH pnpm tauri build --bundles dmg,app

# 4. 产物完整性与权限自动化验收
node scripts/verify-bundle.mjs

# 5. 提交物与空白符检查
git diff --check
```

## 4. 自测记录

- 平台环境：macOS (Darwin aarch64 - Apple Silicon M-series)
- 执行记录：待构建完成后填充
