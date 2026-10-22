# 需求调研：macOS DMG 桌面安装包构建与分发验收

- 状态：ready-to-merge
- Feature slug: desktop-packaging
- 分支：feature/desktop-packaging
- 负责人：Antigravity
- 日期：2026-10-22

## 1. 背景与核心问题

在之前的开发阶段中，Meme Studio 始终运行在开发者模式下（通过 `pnpm tauri:dev` 或浏览器开发服务器）。然而作为面向终端创作者的桌面客户端软件，用户不可能在电脑中预装 Node.js、Rust 编译链或配置全局 FFmpeg 环境变量。

因此，必须完成项目 P1 遗留的最后一道门槛 —— **桌面安装包分发闭环**：
1. **脱离开发环境免配置运行**：产出标准 macOS DMG 镜像包，用户双击打开后拖拽至 Applications 即可安装，无任何外部运行环境依赖；
2. **Sidecar 二进制开箱即用打包**：确保构建打包时，具备全量编码能力的 FFmpeg 与 FFprobe 正确嵌入到 macOS `.app` 包体结构中（`Contents/MacOS` 或 `Contents/Resources`）；
3. **独立运行下的路径解析健壮性**：`resolve_binary` 必须在独立 App 沙箱及打包目录结构下准确命中内置的 FFmpeg/FFprobe，绝不回退至系统 PATH 或报“找不到可执行文件”。

## 2. 用户与交付场景

- **场景 1：创作者一键安装与免配置使用**
  普通用户下载 `表情工坊_0.1.0_aarch64.dmg`，双击挂载，将应用拖拽入“应用程序”文件夹，双击即可直接启动，无需在终端敲任何命令。
- **场景 2：离线无网络媒体处理**
  在完全断网且本机从未安装过 Homebrew/FFmpeg 的环境下，用户导入 4K 视频，进行裁切、文字叠加并导出为 GIF / WebP，应用调用内置 sidecar 顺畅完成全部渲染转码。
- **场景 3：分发产物校验与体积评估**
  构建出合法的 Release DMG 安装包与 App Bundle，统计安装包体积，验证安装包的挂载可读性与文件完整性。

## 3. 技术调研与打包方案

### 3.1 Tauri 2 的 Bundle 机制
- 在 `src-tauri/tauri.conf.json` 中：
  ```json
  "bundle": {
    "active": true,
    "targets": "all",
    "externalBin": [
      "binaries/ffmpeg",
      "binaries/ffprobe"
    ]
  }
  ```
- **Sidecar 寻找机制**：
  构建时，Tauri CLI 会根据当前构建 target（`aarch64-apple-darwin`），在 `src-tauri/` 对应路径下寻找 `binaries/ffmpeg-aarch64-apple-darwin` 与 `binaries/ffprobe-aarch64-apple-darwin`；
  目前该目录下已存在具备 `libfreetype` 与 `libwebp` 的完整静态二进制。

### 3.2 macOS App Bundle 内部结构与二进制寻址
打包后产生的应用包为 `表情工坊.app`，内部典型布局：
```text
表情工坊.app/
└── Contents/
    ├── Info.plist
    ├── MacOS/
    │   ├── 表情工坊 (主二进制)
    │   ├── ffmpeg (或 ffmpeg-aarch64-apple-darwin)
    │   └── ffprobe (或 ffprobe-aarch64-apple-darwin)
    └── Resources/
        ├── icon.icns
        └── ...
```
- **`resolve_binary` 升级方案**：
  在 Rust 中，`current_exe.parent()` 即指向 `Contents/MacOS/`。
  我们需要增强 `resolve_binary`，依次安全探测：
  1. `exe_dir.join(name)`（如 `Contents/MacOS/ffmpeg`）
  2. `exe_dir.join(format!("{}-{}", name, target_triple))`（如 `Contents/MacOS/ffmpeg-aarch64-apple-darwin`）
  3. `exe_dir.join("../Resources/binaries").join(name)`
  4. `exe_dir.join("../Resources").join(name)`
  5. `exe_dir.join("../Resources").join(format!("{}-{}", name, target_triple))`
  6. 源码开发目录候选路径与 PATH 回退。
  确保 100% 能够在脱离开发环境时精准定位到内置 sidecar。

### 3.3 构建指令与环境要求
- 打包命令：
  ```bash
  PATH=/Users/ryantorres/.cargo/bin:$PATH pnpm tauri build --bundles dmg,app
  ```
- 产物目录：
  - App Bundle：`src-tauri/target/release/bundle/macos/表情工坊.app`
  - DMG 镜像：`src-tauri/target/release/bundle/dmg/表情工坊_0.1.0_aarch64.dmg`

## 4. 结论与决策

1. **增强二进制解析**：全面强化 `resolve_binary` 在生产打包 App Bundle 中的定位能力；
2. **配置完善**：确认 `tauri.conf.json` 中 bundle 相关配置规范，包含最低系统版本、应用标识符、图标和产品名称；
3. **真实构建与验收**：执行全量 Release 编译打包，生成实际的 `.dmg` 安装包，并挂载验证 bundle 内部二进制存在性与权限。
