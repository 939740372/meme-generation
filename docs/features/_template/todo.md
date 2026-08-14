# 细化 TODO：<feature-name>

- 状态：implementing
- Feature slug：<feature-slug>
- 分支：feature/<feature-slug>
- 负责人：<name>
- 日期：<YYYY-MM-DD>

每项任务都要写清动作、对象和完成条件。开发中发现范围变化时，先同步需求/设计文档。

## 1. 文档准备

- [ ] 完成 research.md，补齐事实、假设、未决问题和参考资料
- [ ] 完成 requirements.md，补齐范围、需求和验收标准
- [ ] 完成 design.md，补齐模块、数据、错误和平台方案
- [ ] 完成 implementation.md，补齐文件计划和测试计划

## 2. 基础实现

- [ ]

## 3. 错误和边界

- [ ]

## 4. 测试和验证

- [ ] 执行 git diff --check
- [ ] 执行 pnpm build
- [ ] 执行 Rust check
- [ ] 完成必要的桌面冒烟和功能验收
- [ ] 将真实结果写入 implementation.md

## 5. 合并准备

- [ ] 同步五份 feature 文档状态和内容
- [ ] 同步根目录 TODO.md
- [ ] 检查敏感文件、用户媒体和生成物未进入提交
- [ ] 确认工作区和 diff 范围清晰
- [ ] 提交、合并 main，并记录合并结果
