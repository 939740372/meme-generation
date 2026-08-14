# 项目文档

本目录是 Meme Studio 的开发文档中心。文档是功能范围、技术决策和验证结果的可追溯记录，不是开发结束后才补写的一次性附件。

## 目录结构

~~~text
docs/
├── README.md
├── development-workflow.md
└── features/
    ├── _template/
    │   ├── research.md
    │   ├── requirements.md
    │   ├── design.md
    │   ├── implementation.md
    │   └── todo.md
    └── <feature-slug>/
        ├── research.md
        ├── requirements.md
        ├── design.md
        ├── implementation.md
        └── todo.md
~~~

## 使用方式

1. 从 features/_template 复制一份目录，并将目录名改成小写 kebab-case，例如 ffmpeg-gif-mvp。
2. 按 research、requirements、design、implementation、todo 的顺序完成文档初稿。
3. 开发过程中持续更新 implementation.md 和 todo.md，需求或设计发生变化时先更新对应文档。
4. 自测结果、已知限制、阻塞项和回滚方式必须写入 implementation.md。
5. 合并主分支后，将该功能五份文档的状态更新为 merged，并在根目录 TODO.md 保留完成记录和文档链接。

## 当前功能文档

- [FFmpeg GIF MVP](features/ffmpeg-gif-mvp/)，作为视频转 GIF 主链路的第一份功能文档。
- [开发流程规范](development-workflow.md)，说明从方向、调研、设计、开发、自测到合并的完整流程。

## 文档状态

统一使用以下状态，便于快速判断一个功能处于哪个阶段：

proposed → researching → designed → implementing → verifying → ready-to-merge → merged

状态不是替代内容的标签。只有当文档中的入口条件、验收条件和验证记录真实满足时，才可以推进状态。
