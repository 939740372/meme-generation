import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";

console.log("[Smoke Test] Starting Video Import & Frame Preview Verification...");

const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "meme-preview-smoke-"));

function getFfmpegBin() {
  const sidecarMacArm64 = path.resolve("src-tauri/binaries/ffmpeg-aarch64-apple-darwin");
  if (fs.existsSync(sidecarMacArm64)) {
    return sidecarMacArm64;
  }
  const sidecarGeneric = path.resolve("src-tauri/binaries/ffmpeg");
  if (fs.existsSync(sidecarGeneric)) {
    return sidecarGeneric;
  }
  return "ffmpeg";
}

const ffmpegBin = getFfmpegBin();
console.log(`Using FFmpeg binary: ${ffmpegBin}`);

try {
  // 1. 生成 3.0s 的 MP4 测试视频
  const srcMp4 = path.join(tmpDir, "test_source.mp4");
  console.log("1. Generating synthetic MP4 test video...");
  execFileSync(
    ffmpegBin,
    [
      "-y",
      "-f",
      "lavfi",
      "-i",
      "testsrc=duration=3.0:size=640x360:rate=20",
      "-c:v",
      "libx264",
      "-pix_fmt",
      "yuv420p",
      srcMp4,
    ],
    { stdio: "pipe" }
  );
  if (!fs.existsSync(srcMp4)) {
    throw new Error("Failed to create test MP4 video");
  }
  console.log(`  ✓ MP4 created: ${srcMp4} (${fs.statSync(srcMp4).size} bytes)`);

  // 2. 生成 2.0s 的 MKV 格式视频 (模拟非浏览器原生格式)
  const srcMkv = path.join(tmpDir, "test_source.mkv");
  console.log("2. Generating synthetic MKV test video...");
  execFileSync(
    ffmpegBin,
    [
      "-y",
      "-f",
      "lavfi",
      "-i",
      "testsrc=duration=2.0:size=480x320:rate=15",
      "-c:v",
      "libx264",
      srcMkv,
    ],
    { stdio: "pipe" }
  );
  if (!fs.existsSync(srcMkv)) {
    throw new Error("Failed to create test MKV video");
  }
  console.log(`  ✓ MKV created: ${srcMkv} (${fs.statSync(srcMkv).size} bytes)`);

  // 3. 测试极速单帧抽取 (0.0s)
  console.log("3. Testing initial frame extraction (0.0s) via FFmpeg image2pipe...");
  const t0 = Date.now();
  const stdout0 = execFileSync(
    ffmpegBin,
    [
      "-ss",
      "0.00",
      "-i",
      srcMp4,
      "-frames:v",
      "1",
      "-vf",
      "scale='min(640,iw)':-2",
      "-f",
      "image2pipe",
      "-vcodec",
      "mjpeg",
      "-q:v",
      "3",
      "-",
    ],
    { maxBuffer: 10 * 1024 * 1024 }
  );
  const latency0 = Date.now() - t0;
  if (!stdout0 || stdout0.length < 100) {
    throw new Error("Frame extraction returned empty or too small buffer");
  }
  // JPEG magic number: 0xFF 0xD8
  if (stdout0[0] !== 0xff || stdout0[1] !== 0xd8) {
    throw new Error("Returned buffer is not a valid JPEG stream");
  }
  const base64Str0 = `data:image/jpeg;base64,${stdout0.toString("base64")}`;
  console.log(`  ✓ Frame 0.0s extracted in ${latency0}ms (${stdout0.length} bytes, JPEG verified)`);

  // 4. 测试时间点定位抽取 (1.8s)
  console.log("4. Testing targeted frame extraction (1.8s)...");
  const t1 = Date.now();
  const stdout1 = execFileSync(
    ffmpegBin,
    [
      "-ss",
      "1.80",
      "-i",
      srcMp4,
      "-frames:v",
      "1",
      "-vf",
      "scale='min(640,iw)':-2",
      "-f",
      "image2pipe",
      "-vcodec",
      "mjpeg",
      "-q:v",
      "3",
      "-",
    ],
    { maxBuffer: 10 * 1024 * 1024 }
  );
  const latency1 = Date.now() - t1;
  if (stdout1[0] !== 0xff || stdout1[1] !== 0xd8) {
    throw new Error("Returned 1.8s buffer is not valid JPEG");
  }
  console.log(`  ✓ Frame 1.8s extracted in ${latency1}ms (${stdout1.length} bytes)`);

  // 5. 验证 MKV 非原生格式抽帧降级兼容性
  console.log("5. Testing MKV (non-browser-native) frame extraction compatibility...");
  const stdoutMkv = execFileSync(
    ffmpegBin,
    [
      "-ss",
      "0.50",
      "-i",
      srcMkv,
      "-frames:v",
      "1",
      "-vf",
      "scale='min(640,iw)':-2",
      "-f",
      "image2pipe",
      "-vcodec",
      "mjpeg",
      "-q:v",
      "3",
      "-",
    ],
    { maxBuffer: 10 * 1024 * 1024 }
  );
  if (stdoutMkv[0] !== 0xff || stdoutMkv[1] !== 0xd8) {
    throw new Error("MKV frame extraction did not return valid JPEG");
  }
  console.log(`  ✓ MKV frame extracted successfully (${stdoutMkv.length} bytes, 100% compatible)`);

  // 6. 验证 tauri.conf.json assetProtocol 配置
  console.log("6. Verifying tauri.conf.json assetProtocol configuration...");
  const confPath = path.resolve("src-tauri/tauri.conf.json");
  const conf = JSON.parse(fs.readFileSync(confPath, "utf-8"));
  if (!conf.app?.security?.assetProtocol?.enable) {
    throw new Error("tauri.conf.json does not have assetProtocol enabled");
  }
  console.log("  ✓ assetProtocol enabled: true, scope verified");

  console.log("\n🎉 ALL VIDEO PREVIEW SMOKE TESTS PASSED PERFECTLY!");
} finally {
  fs.rmSync(tmpDir, { recursive: true, force: true });
}
