# Meme Studio 开发约束

本文件是本仓库的默认开发流程规范，适用于所有代码、配置、文档和资源变更。系统指令、用户明确要求和更高优先级的协作规则优先于本文件；除此之外，开发工作必须遵守以下约束。

## 1. 开始工作前

每次开始一个新任务，先阅读：

1. AGENTS.md
2. TODO.md
3. docs/development-workflow.md
4. 与本次任务相关的 docs/features/<feature-slug>/ 全部文档

如果仓库状态不干净，先确认改动是否属于当前任务。不得静默覆盖、重置或删除已有用户改动。

## 2. 产品大方向

产品方向和跨阶段优先级只记录在根目录 TODO.md。大方向 TODO 不替代 feature 文档，也不直接承载实现细节。

当前产品方向：构建一个本地优先的跨平台 Meme/GIF 创作桌面客户端，以 FFmpeg 为媒体处理基座，支持视频导入、裁切、压缩、抽帧、文字和图案叠加，以及 GIF 导出。

## 3. Feature 文档是开发入口

每一个 feature 必须拥有独立目录：

~~~text
docs/features/<feature-slug>/
├── research.md
├── requirements.md
├── design.md
├── implementation.md
└── todo.md
~~~

<feature-slug> 使用小写 kebab-case，例如 ffmpeg-gif-mvp。

在开始写业务代码前，至少完成并提交：

- research.md：需求背景、竞品/技术调研、约束、未知项和决策依据
- requirements.md：范围、用户故事、功能/非功能需求和验收标准
- design.md：交互、模块边界、数据流、接口和错误处理设计
- implementation.md：文件级实现计划、迁移/兼容策略、测试方案和风险
- todo.md：按依赖关系拆解到可执行任务的细化清单

文档状态必须明确标记为 proposed、researching、designed、implementing、verifying、ready-to-merge 或 merged。代码状态变化时同步更新文档，不允许文档长期落后于实现。

## 4. 分支规则

禁止直接在 main 上开发。分支必须从最新的 main 创建：

~~~bash
git fetch origin
git switch main
git pull --ff-only origin main
git switch -c feature/<feature-slug>
~~~

分支命名：

| 类型 | 命名 |
|------|------|
| 新功能 | feature/<feature-slug> |
| Bug 修复 | fix/<short-description> |
| 重构 | refactor/<short-description> |
| 工程/流程/依赖 | chore/<short-description> |
| 仅文档 | docs/<short-description> |

一个分支只解决一个主题。若发现范围明显扩大，先更新 TODO.md 和 feature 文档，再决定是否拆分分支。

## 5. 实现规则

- 前端负责交互和展示；文件访问、长任务、进程管理和系统能力放在 Tauri/Rust 层。
- FFmpeg 调用必须通过结构化参数构建，禁止把未经校验的用户输入拼接成 shell 字符串。
- 长任务必须支持进度、失败信息和取消；临时文件必须可追踪并在任务结束后清理。
- FFmpeg sidecar、字体、图案和其他二进制资源必须记录来源、版本、平台和许可证。
- UI 预览不等于最终输出；关键效果必须通过实际 FFmpeg 导出验证。
- 不提交密钥、token、用户媒体、node_modules、dist、Cargo target 或本地规划文件。
- 新增依赖必须说明用途、许可证、体积/启动影响和跨平台风险。
- 任何破坏性变更都必须在 implementation.md 中写明迁移方式和回滚方式。

## 6. 提交规范

提交信息使用 Conventional Commits 风格：

~~~text
feat: add gif export pipeline
fix: handle missing ffmpeg binary
refactor: isolate render job service
docs: define feature workflow
chore: update tauri dependencies
~~~

一个提交尽量只表达一个可理解的变更。提交前必须检查：

~~~bash
git diff --check
git status --short
~~~

## 7. 自测与合并门槛

开发完成后，必须在 feature 分支执行并记录结果：

~~~bash
pnpm install
pnpm build
PATH=/Users/ryantorres/.cargo/bin:$PATH cargo check --manifest-path src-tauri/Cargo.toml --locked
~~~

如果变更涉及桌面端能力，再执行可用的原生 smoke test：

~~~bash
pnpm tauri:dev
~~~

自测记录写入对应 feature 的 implementation.md。至少包含：

- 测试命令或手工步骤
- 预期结果和实际结果
- 平台信息
- 已知限制或未覆盖项

满足以下条件才允许合并：

- feature 文档和细化 TODO 已更新
- 功能实现和验收标准一致
- 类型检查、前端构建和 Rust 检查通过
- 相关手工 smoke test 已完成，或明确记录阻塞原因
- git diff --check 无错误
- 工作树没有未说明的改动
- 没有敏感文件或生成物进入提交

## 8. 合并主分支

默认流程是：feature 分支推送到远程，创建 PR，检查通过后合并到 main。个人仓库或没有 PR 的场景可以在本地使用非快进合并：

~~~bash
git switch main
git pull --ff-only origin main
git merge --no-ff feature/<feature-slug> -m "merge: <feature-slug>"
git push origin main
~~~

合并完成后：

1. 将 feature 文档状态改为 merged
2. 将已完成事项同步回 TODO.md
3. 确认 main 与远程一致
4. 删除已完成的本地 feature 分支；远程分支是否删除按仓库协作需要决定

任何未通过自测、文档未完成或工作树不清晰的分支，都不得合并到 main。
