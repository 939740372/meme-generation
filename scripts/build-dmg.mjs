import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

console.log("=== 表情工坊 macOS DMG 自动化打包管道 ===");

// 1. 确保 CARGO PATH
const cargoBin = path.join(process.env.HOME || "", ".cargo/bin");
const env = {
  ...process.env,
  PATH: `${cargoBin}:${process.env.PATH || ""}`,
};

// 2. 执行 Release 构建生成 .app Bundle
console.log("Step 1: 正在执行 Release 构建与 App Bundle 打包...");
execSync("pnpm tauri build --bundles app", {
  env,
  stdio: "inherit",
});

// 3. 检查 App Bundle 是否存在
const appBundle = path.resolve("src-tauri/target/release/bundle/macos/表情工坊.app");
if (!fs.existsSync(appBundle)) {
  console.error("❌ App Bundle 构建失败，未找到:", appBundle);
  process.exit(1);
}
console.log("✓ App Bundle 构建就绪:", appBundle);

// 4. 创建 Staging 临时目录并生成 DMG
console.log("Step 2: 正在生成 DMG 磁盘镜像...");
const dmgOutDir = path.resolve("src-tauri/target/release/bundle/dmg");
fs.mkdirSync(dmgOutDir, { recursive: true });
const targetDmg = path.join(dmgOutDir, "表情工坊_0.1.0_aarch64.dmg");

const stagingDir = path.join("/tmp", `dmg_staging_${Date.now()}`);
fs.mkdirSync(stagingDir, { recursive: true });

try {
  // 复制 App Bundle 到 staging
  execSync(`cp -R "${appBundle}" "${stagingDir}/"`);
  // 创建 Applications 软链接
  fs.symlinkSync("/Applications", path.join(stagingDir, "Applications"));

  // 使用系统原生 hdiutil 生成 UDZO 压缩镜像
  execSync(
    `hdiutil create -volname "表情工坊" -srcfolder "${stagingDir}" -ov -format UDZO "${targetDmg}"`,
    { stdio: "inherit" },
  );
  console.log("✓ DMG 镜像生成完成:", targetDmg);
} finally {
  fs.rmSync(stagingDir, { recursive: true, force: true });
}

// 5. 执行自动验收
console.log("\nStep 3: 正在执行自动化安装包完整性与权限验收...");
execSync("node scripts/verify-bundle.mjs", { stdio: "inherit" });
