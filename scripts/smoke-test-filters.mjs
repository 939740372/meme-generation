import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const tmpDir = path.resolve("./target/tmp-filter-smoke");
fs.mkdirSync(tmpDir, { recursive: true });

const ffmpegBinary = path.resolve("./src-tauri/binaries/ffmpeg-aarch64-apple-darwin");
const testVideoPath = path.join(tmpDir, "source.mp4");

console.log("1. Generating test source video...");
execFileSync(ffmpegBinary, [
  "-y",
  "-f", "lavfi",
  "-i", "testsrc=size=240x180:rate=15:duration=2",
  "-pix_fmt", "yuv420p",
  testVideoPath,
]);
assert.ok(fs.existsSync(testVideoPath), "Test video must be created");

console.log("2. Testing Grayscale Filter (hue=s=0)...");
const outBw = path.join(tmpDir, "filter_bw.gif");
execFileSync(ffmpegBinary, [
  "-y", "-ss", "0.0", "-t", "1.0", "-i", testVideoPath,
  "-vf", "scale=180:-1:flags=lanczos,fps=10,hue=s=0",
  "-c:v", "gif",
  outBw,
]);
assert.ok(fs.existsSync(outBw), "Grayscale GIF must exist");
assert.ok(fs.statSync(outBw).size > 500, "Grayscale file size should be valid");
console.log(`✓ Grayscale filter OK (${fs.statSync(outBw).size} bytes)`);

console.log("3. Testing Invert/Negate Filter (negate)...");
const outNeg = path.join(tmpDir, "filter_neg.webp");
execFileSync(ffmpegBinary, [
  "-y", "-ss", "0.0", "-t", "1.0", "-i", testVideoPath,
  "-vf", "scale=180:-1:flags=lanczos,fps=10,negate",
  "-c:v", "libwebp", "-q:v", "75", "-loop", "0", "-an",
  outNeg,
]);
assert.ok(fs.existsSync(outNeg), "Negate WebP must exist");
console.log(`✓ Negate filter OK (${fs.statSync(outNeg).size} bytes)`);

console.log("4. Testing 2x Speed Filter (setpts=0.5*PTS)...");
const outFast = path.join(tmpDir, "filter_fast.gif");
execFileSync(ffmpegBinary, [
  "-y", "-ss", "0.0", "-t", "1.0", "-i", testVideoPath,
  "-vf", "scale=180:-1:flags=lanczos,fps=10,setpts=0.5*PTS",
  "-c:v", "gif",
  outFast,
]);
assert.ok(fs.existsSync(outFast), "Fast 2x GIF must exist");
console.log(`✓ Fast 2x filter OK (${fs.statSync(outFast).size} bytes)`);

console.log("5. Testing Reverse Filter (reverse)...");
const outRev = path.join(tmpDir, "filter_rev.avif");
execFileSync(ffmpegBinary, [
  "-y", "-ss", "0.0", "-t", "1.0", "-i", testVideoPath,
  "-vf", "scale=180:-1:flags=lanczos,fps=10,reverse,format=yuv420p",
  "-c:v", "libsvtav1", "-preset", "8", "-crf", "32", "-pix_fmt", "yuv420p", "-an",
  outRev,
]);
assert.ok(fs.existsSync(outRev), "Reverse AVIF must exist");
console.log(`✓ Reverse filter OK (${fs.statSync(outRev).size} bytes)`);

fs.rmSync(tmpDir, { recursive: true, force: true });
console.log("🎉 ALL VIDEO FILTER SMOKE TESTS PASSED!");
