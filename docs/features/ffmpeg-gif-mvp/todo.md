# 细化 TODO：FFmpeg GIF MVP

- 状态：proposed
- Feature slug：ffmpeg-gif-mvp
- 分支：feature/ffmpeg-gif-mvp
- 负责人：待定
- 日期：2026-08-14

## 1. 文档准备

- [x] 完成 research.md 初稿，列出事实、假设、未决问题和许可证风险
- [x] 完成 requirements.md 初稿，列出 MVP 范围和验收标准
- [x] 完成 design.md 初稿，定义 RenderPlan、模块边界和任务事件
- [x] 完成 implementation.md 初稿，列出文件计划、测试和回滚方式
- [ ] 完成 FFmpeg 版本、构建来源和许可证审核

## 2. Sidecar 和媒体基础

- [ ] 确定 FFmpeg 版本、目标平台和构建产物命名
- [ ] 将 FFmpeg 许可证和分发说明写入项目文档
- [ ] 接入 media_probe 并返回时长、尺寸、帧率和格式信息
- [ ] 为 sidecar 缺失、无法启动和非零退出码建立错误分类

## 3. RenderPlan 和渲染管线

- [ ] 定义并校验 RenderPlan 类型
- [ ] 实现 trim、crop、scale 和 fps 参数构建
- [ ] 实现文字 drawtext 参数和字体路径校验
- [ ] 实现 PNG overlay 参数和路径校验
- [ ] 实现 palettegen/paletteuse 两阶段 GIF 输出
- [ ] 实现进度事件、取消、子进程终止和临时目录清理
- [ ] 实现输出存在性、可打开性和基本媒体属性验证

## 4. 任务体验和界面

- [ ] 增加视频导入和媒体信息展示
- [ ] 增加时间、裁切、尺寸和帧率编辑
- [ ] 增加文字和图案编辑入口
- [ ] 增加导出进度、取消、成功路径和错误摘要
- [ ] 增加输出文件命名和覆盖确认

## 5. 测试和验证

- [ ] 准备不进入 Git 的固定短视频、字体和 PNG 测试样例
- [ ] 验证正常导出、参数变化、取消、损坏输入和 sidecar 缺失
- [ ] 验证输出能打开、播放，尺寸、帧率和时长符合设置
- [ ] 执行 git diff --check
- [ ] 执行 pnpm build
- [ ] 执行 Rust check
- [ ] 运行至少一个目标平台的 pnpm tauri:dev 冒烟
- [ ] 将所有真实结果写入 implementation.md

## 6. 合并准备

- [ ] 将五份文档状态更新为 ready-to-merge
- [ ] 同步根目录 TODO.md 的里程碑
- [ ] 检查 sidecar、字体和素材许可证
- [ ] 检查敏感文件、用户媒体和生成物未进入提交
- [ ] 提交、合并 main，并将文档状态更新为 merged
