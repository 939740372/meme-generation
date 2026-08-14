# Meme Studio

一个以本地优先为方向的跨平台桌面客户端底座，当前使用 Tauri 2 + Vue 3 + TypeScript + Vite。

## 开发流程

项目按“方向 TODO → feature 文档 → 独立分支 → 实现 → 自测 → 合并 main”推进。

- [AGENTS.md](AGENTS.md)：仓库级开发约束和合并门槛。
- [TODO.md](TODO.md)：产品大方向、优先级和里程碑。
- [docs/development-workflow.md](docs/development-workflow.md)：完整开发流程。
- [docs/features/_template/](docs/features/_template/)：每个 feature 的文档模板。

每个 feature 必须在 docs/features/<feature-slug>/ 下维护 research.md、requirements.md、design.md、implementation.md 和 todo.md，并在功能完成、自测通过后再合并到 main。

## 当前包含

- Tauri 2 桌面窗口与 src-tauri Rust 工程
- Vue 3 + TypeScript 前端入口
- Vite 开发服务器与 Tauri 开发/构建脚本
- 前端到 Rust 的 invoke 示例：greet、get_app_info
- 一个可继续扩展的工作区导航、状态栏和空状态界面
- Tauri targets: all 跨平台打包配置

## 开始开发

环境要求：Node.js、pnpm、Rust（包含 Cargo）。

~~~bash
pnpm install
pnpm dev
~~~

pnpm dev 用于浏览器预览；要启动原生桌面壳：

~~~bash
pnpm tauri:dev
~~~

构建当前平台的安装包：

~~~bash
pnpm tauri:build
~~~

## 目录约定

~~~text
src/                     Vue 页面、样式与桌面桥接封装
src/lib/desktop.ts       Tauri runtime 检测与 invoke 入口
src-tauri/src/           Rust command 与应用入口
src-tauri/capabilities/  Tauri 2 权限能力声明
~~~

跨平台发布通常需要在对应系统或 CI runner 上分别执行构建；当前配置已经启用所有 Tauri bundle targets。

## 推荐 IDE

VS Code + Vue - Official + Tauri + rust-analyzer。
