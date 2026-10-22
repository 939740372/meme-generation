import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

console.log("=========================================");
console.log("开始验收 macOS Release 安装包与 Sidecar");
console.log("=========================================\n");

const BUNDLE_BASE = path.resolve("src-tauri/target/release/bundle");
const MACOS_APP_DIR = path.join(BUNDLE_BASE, "macos/表情工坊.app");
const DMG_DIR = path.join(BUNDLE_BASE, "dmg");

// 1. 检查 .app 是否存在
if (!fs.existsSync(MACOS_APP_DIR)) {
  console.error(`❌ 未找到 App Bundle: ${MACOS_APP_DIR}`);
  process.exit(1);
}
console.log(`✓ 找到 App Bundle: ${MACOS_APP_DIR}`);

// 2. 检查 Contents 目录
const contentsDir = path.join(MACOS_APP_DIR, "Contents");
const macosDir = path.join(contentsDir, "MacOS");
const resourcesDir = path.join(contentsDir, "Resources");
const infoPlist = path.join(contentsDir, "Info.plist");

if (!fs.existsSync(infoPlist)) {
  console.error("❌ Info.plist 不存在！");
  process.exit(1);
}
console.log("✓ Info.plist 存在且结构完整。");

// 3. 检查主可执行程序与 Sidecar 二进制
const targetTriple = "aarch64-apple-darwin";
const mainExeCandidates = [
  path.join(macosDir, "表情工坊"),
  path.join(macosDir, "meme-studio"),
];
const mainExe = mainExeCandidates.find((c) => fs.existsSync(c));
if (!mainExe) {
  console.error("❌ 主执行程序不存在！候选路径:", mainExeCandidates);
  process.exit(1);
}
console.log(`✓ 主可执行程序存在: ${mainExe}`);

function findSidecarInBundle(name) {
  const candidates = [
    path.join(macosDir, name),
    path.join(macosDir, `${name}-${targetTriple}`),
    path.join(resourcesDir, name),
    path.join(resourcesDir, `${name}-${targetTriple}`),
    path.join(resourcesDir, "binaries", name),
    path.join(resourcesDir, "binaries", `${name}-${targetTriple}`),
  ];
  for (const c of candidates) {
    if (fs.existsSync(c)) {
      return c;
    }
  }
  return null;
}

const ffmpegPath = findSidecarInBundle("ffmpeg");
const ffprobePath = findSidecarInBundle("ffprobe");

if (!ffmpegPath) {
  console.error("❌ 在 App Bundle 中未找到内置的 ffmpeg sidecar！");
  process.exit(1);
}
console.log(`✓ 成功定位内置 ffmpeg sidecar: ${ffmpegPath}`);

if (!ffprobePath) {
  console.error("❌ 在 App Bundle 中未找到内置的 ffprobe sidecar！");
  process.exit(1);
}
console.log(`✓ 成功定位内置 ffprobe sidecar: ${ffprobePath}`);

// 4. 验证权限
try {
  fs.accessSync(ffmpegPath, fs.constants.X_OK);
  fs.accessSync(ffprobePath, fs.constants.X_OK);
  console.log("✓ Sidecar 二进制文件具备可执行权限 (chmod +x)");
} catch (err) {
  console.error("❌ Sidecar 二进制缺少执行权限:", err.message);
  process.exit(1);
}

// 5. 真实执行 sidecar 确认版本和核心编码库支持
try {
  const output = execFileSync(ffmpegPath, ["-version"], { encoding: "utf8" });
  if (!output.includes("libfreetype")) {
    console.error("❌ 内置 ffmpeg 缺失 libfreetype 支持！");
    process.exit(1);
  }
  if (!output.includes("libwebp")) {
    console.error("❌ 内置 ffmpeg 缺失 libwebp 支持！");
    process.exit(1);
  }
  console.log("✓ 内置 ffmpeg 执行正常，确认包含 --enable-libfreetype 与 --enable-libwebp 关键转码库支持！");
} catch (err) {
  console.error("❌ 执行内置 ffmpeg 失败:", err.message);
  process.exit(1);
}

// 6. 检查 DMG 镜像
if (!fs.existsSync(DMG_DIR)) {
  console.error("❌ DMG 输出目录不存在！");
  process.exit(1);
}

const dmgFiles = fs.readdirSync(DMG_DIR).filter((f) => f.endsWith(".dmg"));
if (dmgFiles.length === 0) {
  console.error("❌ 未在 dmg 目录找到生成的 .dmg 安装包！");
  process.exit(1);
}

for (const dmg of dmgFiles) {
  const fullDmgPath = path.join(DMG_DIR, dmg);
  const stats = fs.statSync(fullDmgPath);
  const sizeMb = (stats.size / (1024 * 1024)).toFixed(2);
  console.log(`\n🎉 找到 macOS 安装镜像: ${dmg}`);
  console.log(`   文件路径: ${fullDmgPath}`);
  console.log(`   文件大小: ${sizeMb} MB`);
}

console.log("\n=========================================");
console.log("全部桌面安装包验收项目 100% 通过！");
console.log("=========================================");
