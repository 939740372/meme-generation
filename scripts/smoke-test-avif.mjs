import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const tmpDir = path.resolve("./target/tmp-avif-smoke");
fs.mkdirSync(tmpDir, { recursive: true });

const ffmpegBinary = path.resolve("./src-tauri/binaries/ffmpeg-aarch64-apple-darwin");
const testVideoPath = path.join(tmpDir, "source.mp4");
const outAvifPath = path.join(tmpDir, "output.avif");

console.log("1. Generating synthetic test video...");
execFileSync(ffmpegBinary, [
  "-y",
  "-f", "lavfi",
  "-i", "testsrc=size=320x240:rate=15:duration=2",
  "-pix_fmt", "yuv420p",
  testVideoPath,
]);
assert.ok(fs.existsSync(testVideoPath), "Test source video must exist");

console.log("2. Transcoding to AVIF using sidecar libsvtav1...");
// Pipeline: scale, drawtext, format=yuv420p, libsvtav1, preset 8, crf 32
execFileSync(ffmpegBinary, [
  "-y",
  "-ss", "0.0",
  "-t", "1.5",
  "-i", testVideoPath,
  "-vf", "scale=240:-1:flags=lanczos,format=yuv420p",
  "-c:v", "libsvtav1",
  "-preset", "8",
  "-crf", "32",
  "-pix_fmt", "yuv420p",
  "-an",
  outAvifPath,
]);

assert.ok(fs.existsSync(outAvifPath), "AVIF output file must exist");
const stat = fs.statSync(outAvifPath);
console.log(`✓ AVIF file produced: ${stat.size} bytes`);
assert.ok(stat.size > 500, "AVIF file size should be reasonable (> 500 bytes)");

// Check file header contains 'ftyp' and 'avif' or 'avis'
const buf = fs.readFileSync(outAvifPath);
const headerStr = buf.subarray(0, 32).toString("binary");
assert.ok(headerStr.includes("ftyp"), "File must have ISO Media ftyp box");
assert.ok(headerStr.includes("avif") || headerStr.includes("avis"), "File brand must be avif or avis");

console.log("3. Comparing AVIF size against GIF and WebP...");
const outGifPath = path.join(tmpDir, "output.gif");
const palettePath = path.join(tmpDir, "palette.png");
execFileSync(ffmpegBinary, [
  "-y", "-ss", "0.0", "-t", "1.5", "-i", testVideoPath,
  "-vf", "scale=240:-1:flags=lanczos,palettegen", palettePath,
]);
execFileSync(ffmpegBinary, [
  "-y", "-ss", "0.0", "-t", "1.5", "-i", testVideoPath, "-i", palettePath,
  "-lavfi", "scale=240:-1:flags=lanczos [x]; [x][1:v] paletteuse", outGifPath,
]);
const gifStat = fs.statSync(outGifPath);

const outWebpPath = path.join(tmpDir, "output.webp");
execFileSync(ffmpegBinary, [
  "-y", "-ss", "0.0", "-t", "1.5", "-i", testVideoPath,
  "-vf", "scale=240:-1:flags=lanczos",
  "-c:v", "libwebp", "-q:v", "75", "-loop", "0", "-an", outWebpPath,
]);
const webpStat = fs.statSync(outWebpPath);

console.log(`  - GIF  Size: ${(gifStat.size / 1024).toFixed(1)} KB`);
console.log(`  - WebP Size: ${(webpStat.size / 1024).toFixed(1)} KB`);
console.log(`  - AVIF Size: ${(stat.size / 1024).toFixed(1)} KB`);

// AVIF should be much smaller than GIF
assert.ok(stat.size < gifStat.size * 0.4, "AVIF should be at least 60% smaller than GIF");

fs.rmSync(tmpDir, { recursive: true, force: true });
console.log("🎉 ALL AVIF SMOKE TESTS PASSED!");
