import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const tmpDir = path.resolve("./target/tmp-hwaccel-smoke");
fs.mkdirSync(tmpDir, { recursive: true });

const ffmpegBinary = path.resolve("./src-tauri/binaries/ffmpeg-aarch64-apple-darwin");
const ffprobeBinary = path.resolve("./src-tauri/binaries/ffprobe-aarch64-apple-darwin");

const testSourcePath = path.join(tmpDir, "source.mp4");

console.log("1. Generating test source video (640x480, 25fps)...");
execFileSync(ffmpegBinary, [
  "-y",
  "-f", "lavfi", "-i", "testsrc=size=640x480:rate=25:duration=2",
  "-c:v", "libx264", "-pix_fmt", "yuv420p",
  testSourcePath,
]);
assert.ok(fs.existsSync(testSourcePath), "Source video must exist");

function probeVideoCodec(filePath) {
  const stdout = execFileSync(ffprobeBinary, [
    "-v", "error",
    "-select_streams", "v:0",
    "-show_entries", "stream=codec_name,width,height",
    "-of", "json",
    filePath,
  ]);
  const parsed = JSON.parse(stdout.toString("utf-8"));
  return parsed.streams?.[0] || null;
}

console.log("2. Probing available hardware encoders in FFmpeg sidecar...");
const encodersOutput = execFileSync(ffmpegBinary, ["-hide_banner", "-encoders"]).toString("utf-8");
const hasVideoToolbox = encodersOutput.includes("h264_videotoolbox");
const hasNvenc = encodersOutput.includes("h264_nvenc");
const hasQsv = encodersOutput.includes("h264_qsv");

console.log(`Detected encoders: VideoToolbox=${hasVideoToolbox}, NVENC=${hasNvenc}, QSV=${hasQsv}`);

if (hasVideoToolbox && process.platform === "darwin") {
  console.log("3. Testing macOS VideoToolbox hardware accelerated encoding...");
  const outHw = path.join(tmpDir, "out_videotoolbox.mp4");
  const t0 = Date.now();
  execFileSync(ffmpegBinary, [
    "-y",
    "-hwaccel", "auto",
    "-ss", "0.0", "-t", "1.5",
    "-i", testSourcePath,
    "-filter_complex", "[0:v]scale=480:-2:flags=lanczos,fps=24,format=yuv420p[vout]",
    "-map", "[vout]",
    "-c:v", "h264_videotoolbox",
    "-q:v", "65",
    "-movflags", "+faststart",
    outHw,
  ]);
  const tHw = Date.now() - t0;
  assert.ok(fs.existsSync(outHw), "Hardware accelerated MP4 must exist");
  const streamHw = probeVideoCodec(outHw);
  assert.ok(streamHw, "Video stream must be present");
  assert.equal(streamHw.codec_name, "h264");
  assert.equal(streamHw.width, 480);
  console.log(`✓ VideoToolbox hardware encoding succeeded in ${tHw}ms (${fs.statSync(outHw).size} bytes)`);
} else {
  console.log("3. Skipping VideoToolbox test (not on macOS or not supported)");
}

console.log("4. Testing CPU software encoding fallback (libx264)...");
const outSw = path.join(tmpDir, "out_software.mp4");
const t0Sw = Date.now();
execFileSync(ffmpegBinary, [
  "-y",
  "-ss", "0.0", "-t", "1.5",
  "-i", testSourcePath,
  "-filter_complex", "[0:v]scale=480:-2:flags=lanczos,fps=24,format=yuv420p[vout]",
  "-map", "[vout]",
  "-c:v", "libx264",
  "-preset", "medium",
  "-crf", "23",
  "-movflags", "+faststart",
  outSw,
]);
const tSw = Date.now() - t0Sw;
assert.ok(fs.existsSync(outSw), "Software MP4 must exist");
const streamSw = probeVideoCodec(outSw);
assert.ok(streamSw, "Video stream must be present");
assert.equal(streamSw.codec_name, "h264");
assert.equal(streamSw.width, 480);
console.log(`✓ CPU software encoding succeeded in ${tSw}ms (${fs.statSync(outSw).size} bytes)`);

fs.rmSync(tmpDir, { recursive: true, force: true });
console.log("🎉 ALL HARDWARE ACCELERATION SMOKE TESTS PASSED!");
