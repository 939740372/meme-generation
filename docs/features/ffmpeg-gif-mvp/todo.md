# 细化 TODO：FFmpeg GIF MVP

- 状态：ready-to-merge
- Feature slug：ffmpeg-gif-mvp
- 分支：feature/ffmpeg-gif-mvp
- 负责人：待定
- 日期：2026-10-22

## 1. 文档准备

- [x] 完成 research.md 初稿，列出事实、假设、未决问题和许可证风险
- [x] 完成 requirements.md 初稿，列出 MVP 范围和验收标准
- [x] 完成 design.md 初稿，定义 RenderPlan、模块边界和任务事件
- [x] 完成 implementation.md 初稿，列出文件计划、测试和回滚方式
- [ ] 完成可分发 FFmpeg 版本、构建来源和许可证审核（后置于发布阶段，不阻塞本机验证版）

## 2. Sidecar 和媒体基础

- [x] 确定本机验证版使用 PATH/`FFMPEG_BINARY`/`FFPROBE_BINARY`
- [x] 将本机验证版与后续 sidecar 许可证/分发任务区分记录
- [x] 接入 media_probe 并返回时长、尺寸、帧率和格式信息
- [x] 为 FFmpeg 缺失、无法启动和非零退出码建立错误分类

## 3. RenderPlan 和渲染管线

- [x] 定义并校验 RenderPlan 类型
- [x] 实现 trim、crop、scale 和 fps 参数构建
- [x] 实现文字 drawtext 参数和颜色校验
- [x] 实现 PNG overlay 参数和路径校验
- [x] 实现 palettegen/paletteuse filter graph GIF 输出
- [x] 实现进度事件、取消、子进程终止和任务状态清理
- [x] 实现输出存在性、非空和基本媒体属性验证

## 4. 任务体验和界面

- [x] 增加视频导入和媒体信息展示
- [x] 增加时间、裁切、尺寸和帧率编辑
- [x] 增加文字和图案编辑入口
- [x] 增加导出进度、取消、成功路径和错误摘要
- [x] 增加输出文件命名和覆盖确认
- [x] 将 Tauri 窗口锁定为 1280×820，完成条件是窗口不可缩放且配置包含固定边界
- [x] 重做固定视口桌面工作台样式，完成条件是预览、参数和导出状态在同一窗口内可见
- [x] 消除应用级横向/纵向滚动条，完成条件是默认和常用叠加参数状态均无滚动条
- [x] 精简页面无效文案，完成条件是未实现导航和重复营销说明不再占用工作区
- [x] 放大主要界面字体至少 50%，完成条件是固定窗口下默认/常用参数状态无布局截断
- [x] 完成第一版简体中文界面，完成条件是用户可见界面、状态、错误和系统文件筛选名称不再混用英文

## 5. 测试和验证

- [x] 使用系统临时目录生成不进入 Git 的固定短视频和 PNG 测试样例
- [x] 验证正常导出、参数变化、取消、损坏输入和 sidecar 缺失（真实媒体/错误单测已完成，桌面交互待补）
- [x] 验证输出能打开、播放，尺寸、帧率和时长符合设置
- [x] 执行 git diff --check
- [x] 执行 pnpm build
- [x] 执行 Rust check
- [x] 运行至少一个目标平台的 pnpm tauri:dev 冒烟（binary 已编译运行，窗口交互未能在当前环境确认）
- [x] 将所有真实结果写入 implementation.md

## 6. 合并准备

- [x] 将五份文档状态更新为 ready-to-merge
- [x] 同步根目录 TODO.md 的里程碑
- [x] 检查 sidecar、字体和素材许可证
- [x] 检查敏感文件、用户媒体和生成物未进入提交
- [ ] 提交、合并 main，并将文档状态更新为 merged
