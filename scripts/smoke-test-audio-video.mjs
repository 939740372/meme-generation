import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const tmpDir = path.resolve("./target/tmp-audio-video-smoke");
fs.mkdirSync(tmpDir, { recursive: true });

const ffmpegBinary = path.resolve("./src-tauri/binaries/ffmpeg-aarch64-apple-darwin");
const ffprobeBinary = path.resolve("./src-tauri/binaries/ffprobe-aarch64-apple-darwin");

const sourceWithAudio = path.join(tmpDir, "source_with_audio.mp4");
const sourceNoAudio = path.join(tmpDir, "source_no_audio.mp4");

console.log("1. Generating synthetic test source videos...");
// Video + Audio source
execFileSync(ffmpegBinary, [
  "-y",
  "-f", "lavfi", "-i", "testsrc=size=320x240:rate=25:duration=2",
  "-f", "lavfi", "-i", "sine=frequency=1000:duration=2",
  "-c:v", "libx264", "-pix_fmt", "yuv420p",
  "-c:a", "aac", "-b:a", "128k",
  sourceWithAudio,
]);
assert.ok(fs.existsSync(sourceWithAudio), "Source with audio must exist");

// Video without Audio source
execFileSync(ffmpegBinary, [
  "-y",
  "-f", "lavfi", "-i", "testsrc=size=320x240:rate=25:duration=2",
  "-c:v", "libx264", "-pix_fmt", "yuv420p",
  "-an",
  sourceNoAudio,
]);
assert.ok(fs.existsSync(sourceNoAudio), "Source without audio must exist");

function probeStreams(filePath) {
  const stdout = execFileSync(ffprobeBinary, [
    "-v", "error",
    "-show_entries", "stream=codec_name,codec_type,duration",
    "-of", "json",
    filePath,
  ]);
  const parsed = JSON.parse(stdout.toString("utf-8"));
  return parsed.streams || [];
}

console.log("2. Testing MP4 export with audio enabled...");
const outNormal = path.join(tmpDir, "out_with_audio.mp4");
execFileSync(ffmpegBinary, [
  "-y",
  "-ss", "0.0", "-t", "1.0",
  "-i", sourceWithAudio,
  "-filter_complex", "[0:v]scale=320:-2:flags=lanczos,fps=24,format=yuv420p[vout]",
  "-map", "[vout]",
  "-c:v", "libx264", "-preset", "medium", "-crf", "23", "-pix_fmt", "yuv420p", "-movflags", "+faststart",
  "-map", "0:a?", "-c:a", "aac", "-b:a", "128k",
  outNormal,
]);
assert.ok(fs.existsSync(outNormal), "Exported MP4 must exist");
const streams1 = probeStreams(outNormal);
const videoStream1 = streams1.find((s) => s.codec_type === "video");
const audioStream1 = streams1.find((s) => s.codec_type === "audio");
assert.ok(videoStream1, "Video stream must be present");
assert.equal(videoStream1.codec_name, "h264");
assert.ok(audioStream1, "Audio stream must be present");
assert.equal(audioStream1.codec_name, "aac");
console.log(`✓ MP4 with H.264 + AAC audio verified (${fs.statSync(outNormal).size} bytes)`);

console.log("3. Testing MP4 export with audio muted (-an)...");
const outMuted = path.join(tmpDir, "out_muted.mp4");
execFileSync(ffmpegBinary, [
  "-y",
  "-ss", "0.0", "-t", "1.0",
  "-i", sourceWithAudio,
  "-filter_complex", "[0:v]scale=320:-2:flags=lanczos,fps=24,format=yuv420p[vout]",
  "-map", "[vout]",
  "-c:v", "libx264", "-preset", "medium", "-crf", "23", "-pix_fmt", "yuv420p", "-movflags", "+faststart",
  "-an",
  outMuted,
]);
assert.ok(fs.existsSync(outMuted), "Muted MP4 must exist");
const streams2 = probeStreams(outMuted);
assert.ok(streams2.some((s) => s.codec_type === "video"));
assert.ok(!streams2.some((s) => s.codec_type === "audio"), "No audio stream when muted");
console.log(`✓ Muted MP4 verified (${fs.statSync(outMuted).size} bytes)`);

console.log("4. Testing MP4 export tolerance on no-audio input (0:a? fallback)...");
const outFallback = path.join(tmpDir, "out_fallback_no_audio.mp4");
execFileSync(ffmpegBinary, [
  "-y",
  "-ss", "0.0", "-t", "1.0",
  "-i", sourceNoAudio,
  "-filter_complex", "[0:v]scale=320:-2:flags=lanczos,fps=24,format=yuv420p[vout]",
  "-map", "[vout]",
  "-c:v", "libx264", "-preset", "medium", "-crf", "23", "-pix_fmt", "yuv420p", "-movflags", "+faststart",
  "-map", "0:a?", "-c:a", "aac", "-b:a", "128k",
  outFallback,
]);
assert.ok(fs.existsSync(outFallback), "Fallback MP4 must exist");
const streams3 = probeStreams(outFallback);
assert.ok(streams3.some((s) => s.codec_type === "video"));
assert.ok(!streams3.some((s) => s.codec_type === "audio"), "Silent input produces clean video without crashing");
console.log(`✓ 0:a? graceful tolerance verified without crashing (${fs.statSync(outFallback).size} bytes)`);

console.log("5. Testing MP4 audio synchronization with 2x speed filter (setpts + atempo)...");
const outFast2x = path.join(tmpDir, "out_fast2x.mp4");
execFileSync(ffmpegBinary, [
  "-y",
  "-ss", "0.0", "-t", "1.0",
  "-i", sourceWithAudio,
  "-filter_complex", "[0:v]scale=320:-2:flags=lanczos,fps=24,setpts=0.5*PTS,format=yuv420p[vout]",
  "-map", "[vout]",
  "-c:v", "libx264", "-preset", "medium", "-crf", "23", "-pix_fmt", "yuv420p", "-movflags", "+faststart",
  "-map", "0:a?", "-c:a", "aac", "-b:a", "128k", "-af", "atempo=2.0",
  outFast2x,
]);
assert.ok(fs.existsSync(outFast2x), "Fast 2x MP4 must exist");
const streams4 = probeStreams(outFast2x);
const video4 = streams4.find((s) => s.codec_type === "video");
const audio4 = streams4.find((s) => s.codec_type === "audio");
assert.ok(video4 && audio4);
const vDur = parseFloat(video4.duration);
const aDur = parseFloat(audio4.duration);
assert.ok(vDur >= 0.4 && vDur <= 0.65, `Video duration should be approx 0.5s (got ${vDur}s)`);
assert.ok(aDur >= 0.4 && aDur <= 0.65, `Audio duration should be approx 0.5s (got ${aDur}s)`);
console.log(`✓ 2x speed sync (setpts=0.5*PTS + atempo=2.0) verified (${fs.statSync(outFast2x).size} bytes)`);

console.log("6. Testing MP4 audio reverse synchronization (reverse + areverse)...");
const outReverse = path.join(tmpDir, "out_reverse.mp4");
execFileSync(ffmpegBinary, [
  "-y",
  "-ss", "0.0", "-t", "1.0",
  "-i", sourceWithAudio,
  "-filter_complex", "[0:v]scale=320:-2:flags=lanczos,fps=24,reverse,format=yuv420p[vout]",
  "-map", "[vout]",
  "-c:v", "libx264", "-preset", "medium", "-crf", "23", "-pix_fmt", "yuv420p", "-movflags", "+faststart",
  "-map", "0:a?", "-c:a", "aac", "-b:a", "128k", "-af", "areverse",
  outReverse,
]);
assert.ok(fs.existsSync(outReverse), "Reverse MP4 must exist");
const streams5 = probeStreams(outReverse);
assert.ok(streams5.some((s) => s.codec_type === "video"));
assert.ok(streams5.some((s) => s.codec_type === "audio"));
console.log(`✓ Reverse sync (reverse + areverse) verified (${fs.statSync(outReverse).size} bytes)`);

fs.rmSync(tmpDir, { recursive: true, force: true });
console.log("🎉 ALL AUDIO VIDEO EXPORT SMOKE TESTS PASSED!");
