import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import {
  createQueueItem,
  deserializeQueue,
  generateQueueOutputPath,
  serializeQueue,
} from "../src/lib/queue.ts";

const tmpDir = path.resolve("./target/tmp-queue-smoke");
fs.mkdirSync(tmpDir, { recursive: true });

const ffmpegBinary = path.resolve("./src-tauri/binaries/ffmpeg-aarch64-apple-darwin");
const testVideoPath = path.join(tmpDir, "source.mp4");

console.log("1. Generating synthetic test video with FFmpeg...");
execFileSync(ffmpegBinary, [
  "-y",
  "-f", "lavfi",
  "-i", "testsrc=size=320x240:rate=15:duration=3",
  "-pix_fmt", "yuv420p",
  testVideoPath,
]);
assert.ok(fs.existsSync(testVideoPath), "Source video should be created");

console.log("2. Simulating Queue Item additions...");
const existingPaths = new Set();

const plan1 = {
  inputPath: testVideoPath,
  outputPath: path.join(tmpDir, "out1.gif"),
  startSecs: 0.0,
  endSecs: 1.0,
  crop: null,
  outputWidth: 200,
  fps: 10,
  quality: "medium",
  textOverlay: null,
  textLayers: null,
  imageOverlay: null,
  format: "gif",
};

const item1 = createQueueItem(plan1, "source.mp4", existingPaths);
existingPaths.add(item1.outputPath);

const plan2 = {
  inputPath: testVideoPath,
  outputPath: path.join(tmpDir, "out2.webp"),
  startSecs: 1.0,
  endSecs: 2.2,
  crop: null,
  outputWidth: 240,
  fps: 12,
  quality: "medium",
  textOverlay: null,
  textLayers: null,
  imageOverlay: null,
  format: "webp",
};

const item2 = createQueueItem(plan2, "source.mp4", existingPaths);
existingPaths.add(item2.outputPath);

assert.notEqual(item1.outputPath, item2.outputPath);
assert.ok(item1.outputPath.endsWith(".gif"));
assert.ok(item2.outputPath.endsWith(".webp"));

console.log("3. Testing queue serialization roundtrip...");
const queue = [item1, item2];
const serialized = serializeQueue(queue);
const restored = deserializeQueue(serialized);
assert.equal(restored.length, 2);
assert.equal(restored[0].status, "pending");
assert.equal(restored[1].status, "pending");

console.log("4. Executing simulated batch execution using real FFmpeg sidecar...");
// Process item 1 (GIF pipeline: palettegen + paletteuse)
const palettePath = path.join(tmpDir, "palette.png");
execFileSync(ffmpegBinary, [
  "-y",
  "-ss", String(item1.plan.startSecs),
  "-t", String(item1.plan.endSecs - item1.plan.startSecs),
  "-i", item1.plan.inputPath,
  "-vf", `scale=${item1.plan.outputWidth}:-1:flags=lanczos,palettegen`,
  palettePath,
]);

execFileSync(ffmpegBinary, [
  "-y",
  "-ss", String(item1.plan.startSecs),
  "-t", String(item1.plan.endSecs - item1.plan.startSecs),
  "-i", item1.plan.inputPath,
  "-i", palettePath,
  "-lavfi", `scale=${item1.plan.outputWidth}:-1:flags=lanczos [x]; [x][1:v] paletteuse=dither=sierra2_4a`,
  item1.outputPath,
]);

assert.ok(fs.existsSync(item1.outputPath), "Item 1 output GIF should exist");
const item1Stat = fs.statSync(item1.outputPath);
assert.ok(item1Stat.size > 1000, "Item 1 output GIF should have valid bytes");
console.log(`✓ Item 1 (GIF) exported successfully: ${item1Stat.size} bytes`);

// Process item 2 (WebP pipeline)
execFileSync(ffmpegBinary, [
  "-y",
  "-ss", String(item2.plan.startSecs),
  "-t", String(item2.plan.endSecs - item2.plan.startSecs),
  "-i", item2.plan.inputPath,
  "-vf", `scale=${item2.plan.outputWidth}:-1:flags=lanczos`,
  "-c:v", "libwebp",
  "-lossless", "0",
  "-q:v", "75",
  "-loop", "0",
  "-an",
  item2.outputPath,
]);

assert.ok(fs.existsSync(item2.outputPath), "Item 2 output WebP should exist");
const item2Stat = fs.statSync(item2.outputPath);
assert.ok(item2Stat.size > 500, "Item 2 output WebP should have valid bytes");
console.log(`✓ Item 2 (WebP) exported successfully: ${item2Stat.size} bytes`);

// Clean up smoke test tmp
fs.rmSync(tmpDir, { recursive: true, force: true });

console.log("🎉 ALL BATCH QUEUE SMOKE TESTS PASSED!");
