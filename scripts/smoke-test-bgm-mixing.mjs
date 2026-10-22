#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");
const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "meme-bgm-smoke-"));

console.log(`[Smoke Test] Starting BGM & SFX Audio Mixing Verification...`);
console.log(`[Smoke Test] Workspace temp dir: ${tempDir}`);

function runFfprobe(filePath) {
  const output = execFileSync("ffprobe", [
    "-v",
    "error",
    "-show_entries",
    "stream=codec_type,codec_name:format=duration",
    "-of",
    "json",
    filePath,
  ]);
  return JSON.parse(output.toString("utf8"));
}

try {
  // 1. 验证 8 款内置音效完整性
  const sfxDir = path.resolve(rootDir, "src-tauri", "sfx");
  const expectedSfx = [
    "censor_bleep.mp3",
    "crisp_ding.mp3",
    "boom_808.mp3",
    "alarm_danger.mp3",
    "boing_spring.mp3",
    "victory_cheer.mp3",
    "face_slap.mp3",
    "goofy_laugh.mp3",
  ];

  console.log(`\n--- Step 1: Validating 8 built-in sound effects in ${sfxDir} ---`);
  for (const filename of expectedSfx) {
    const sfxPath = path.join(sfxDir, filename);
    if (!fs.existsSync(sfxPath)) {
      throw new Error(`Missing expected sfx file: ${filename}`);
    }
    const stat = fs.statSync(sfxPath);
    const probe = runFfprobe(sfxPath);
    const dur = parseFloat(probe.format?.duration || "0");
    console.log(`  ✓ ${filename} (${(stat.size / 1024).toFixed(1)} KB, duration: ${dur.toFixed(2)}s)`);
    if (dur <= 0) {
      throw new Error(`Invalid duration for sfx ${filename}: ${dur}`);
    }
  }

  // 2. 生成测试源素材
  const srcWithAudio = path.join(tempDir, "src_with_audio.mp4");
  const srcNoAudio = path.join(tempDir, "src_no_audio.mp4");

  console.log(`\n--- Step 2: Generating synthetic test sources ---`);
  // 源视频 A: 2.0s 包含 440Hz 正弦音频
  execFileSync("ffmpeg", [
    "-y",
    "-f",
    "lavfi",
    "-i",
    "testsrc=duration=2.0:size=320x240:rate=20",
    "-f",
    "lavfi",
    "-i",
    "sine=frequency=440:duration=2.0",
    "-c:v",
    "libx264",
    "-c:a",
    "aac",
    srcWithAudio,
  ]);
  console.log(`  ✓ Created 2.0s video with audio: ${srcWithAudio}`);

  // 源视频 B: 2.0s 纯静音无音频流
  execFileSync("ffmpeg", [
    "-y",
    "-f",
    "lavfi",
    "-i",
    "testsrc=duration=2.0:size=320x240:rate=20",
    "-c:v",
    "libx264",
    "-an",
    srcNoAudio,
  ]);
  console.log(`  ✓ Created 2.0s silent video (no audio stream): ${srcNoAudio}`);

  // 3. 场景 1：源视频有原声 + 选用消音哔哔 (censor_bleep.mp3) 双路混流
  console.log(`\n--- Step 3: Testing dual-stream mixing (Original + Censor Bleep) ---`);
  const outMixed = path.join(tempDir, "out_dual_mixed.mp4");
  const bleepPath = path.join(sfxDir, "censor_bleep.mp3");

  // 模拟 Rust 端 build_ffmpeg_args 生成的命令
  execFileSync("ffmpeg", [
    "-y",
    "-hide_banner",
    "-loglevel",
    "error",
    "-ss",
    "0.000",
    "-t",
    "2.000",
    "-i",
    srcWithAudio,
    "-i",
    bleepPath,
    "-filter_complex",
    "[0:v]scale=320:-2:flags=lanczos,fps=20,format=yuv420p[vout];[0:a]volume=0.60[a0];[1:a]volume=0.80,atrim=0:2.000[a1];[a0][a1]amix=inputs=2:duration=first:dropout_transition=0[aout]",
    "-map",
    "[vout]",
    "-c:v",
    "libx264",
    "-pix_fmt",
    "yuv420p",
    "-movflags",
    "+faststart",
    "-map",
    "[aout]",
    "-c:a",
    "aac",
    "-b:a",
    "128k",
    outMixed,
  ]);

  const probeMixed = runFfprobe(outMixed);
  const vStreams1 = probeMixed.streams.filter((s) => s.codec_type === "video");
  const aStreams1 = probeMixed.streams.filter((s) => s.codec_type === "audio");
  const durMixed = parseFloat(probeMixed.format.duration);
  console.log(`  ✓ Exported MP4: ${outMixed}`);
  console.log(`  ✓ Video streams: ${vStreams1.length} (${vStreams1[0]?.codec_name})`);
  console.log(`  ✓ Audio streams: ${aStreams1.length} (${aStreams1[0]?.codec_name})`);
  console.log(`  ✓ Total duration: ${durMixed.toFixed(3)}s (strict 2.0s alignment)`);

  if (vStreams1.length !== 1 || aStreams1.length !== 1) {
    throw new Error("Stream counts incorrect in dual-mixed export!");
  }
  if (Math.abs(durMixed - 2.0) > 0.1) {
    throw new Error(`Duration mismatched! Expected 2.0s, got ${durMixed}s`);
  }

  // 4. 场景 2：源视频完全无音频流 + 选用震撼低音 (boom_808.mp3)
  console.log(`\n--- Step 4: Testing silent source + BGM stream (Boom 808) ---`);
  const outNoOrig = path.join(tempDir, "out_no_orig_with_sfx.mp4");
  const boomPath = path.join(sfxDir, "boom_808.mp3");

  execFileSync("ffmpeg", [
    "-y",
    "-hide_banner",
    "-loglevel",
    "error",
    "-ss",
    "0.000",
    "-t",
    "2.000",
    "-i",
    srcNoAudio,
    "-i",
    boomPath,
    "-filter_complex",
    "[0:v]scale=320:-2:flags=lanczos,fps=20,format=yuv420p[vout];[1:a]volume=1.00,atrim=0:2.000[aout]",
    "-map",
    "[vout]",
    "-c:v",
    "libx264",
    "-pix_fmt",
    "yuv420p",
    "-movflags",
    "+faststart",
    "-map",
    "[aout]",
    "-c:a",
    "aac",
    "-b:a",
    "128k",
    outNoOrig,
  ]);

  const probeNoOrig = runFfprobe(outNoOrig);
  const aStreams2 = probeNoOrig.streams.filter((s) => s.codec_type === "audio");
  console.log(`  ✓ Exported MP4 with zero original audio: ${outNoOrig}`);
  console.log(`  ✓ Audio stream correctly injected: ${aStreams2.length} (${aStreams2[0]?.codec_name})`);
  if (aStreams2.length !== 1) {
    throw new Error("Audio stream failed to inject for silent source video!");
  }

  // 5. 场景 3：开启 2x 鬼畜加速 (atempo=2.0) + 原声与清脆叮混流
  console.log(`\n--- Step 5: Testing speedup (fast2x) tempo sync + SFX mixing ---`);
  const outFast = path.join(tempDir, "out_fast2x_sfx.mp4");
  const dingPath = path.join(sfxDir, "crisp_ding.mp3");

  execFileSync("ffmpeg", [
    "-y",
    "-hide_banner",
    "-loglevel",
    "error",
    "-ss",
    "0.000",
    "-t",
    "2.000",
    "-i",
    srcWithAudio,
    "-i",
    dingPath,
    "-filter_complex",
    "[0:v]scale=320:-2:flags=lanczos,fps=20,setpts=0.5*PTS,format=yuv420p[vout];[0:a]atempo=2.0,volume=1.00[a0];[1:a]volume=1.00,atrim=0:1.000[a1];[a0][a1]amix=inputs=2:duration=first:dropout_transition=0[aout]",
    "-map",
    "[vout]",
    "-c:v",
    "libx264",
    "-pix_fmt",
    "yuv420p",
    "-movflags",
    "+faststart",
    "-map",
    "[aout]",
    "-c:a",
    "aac",
    "-b:a",
    "128k",
    outFast,
  ]);

  const probeFast = runFfprobe(outFast);
  const durFast = parseFloat(probeFast.format.duration);
  console.log(`  ✓ Exported 2x Fast MP4: ${outFast} (duration: ${durFast.toFixed(3)}s)`);
  if (Math.abs(durFast - 1.0) > 0.15) {
    throw new Error(`2x accelerated duration expected ~1.0s, got ${durFast}s`);
  }

  // 6. 场景 4：动图格式（GIF）自动 -an 静音
  console.log(`\n--- Step 6: Testing GIF format mute safety (-an) ---`);
  const outGif = path.join(tempDir, "out_muted.gif");
  execFileSync("ffmpeg", [
    "-y",
    "-hide_banner",
    "-loglevel",
    "error",
    "-ss",
    "0.000",
    "-t",
    "1.000",
    "-i",
    srcWithAudio,
    "-filter_complex",
    "[0:v]scale=320:-1:flags=lanczos,fps=12[base];[base]split[gif][palette];[palette]palettegen=stats_mode=diff[p];[gif][p]paletteuse=dither=sierra2_4a[vout]",
    "-map",
    "[vout]",
    "-an",
    "-loop",
    "0",
    "-f",
    "gif",
    outGif,
  ]);

  const probeGif = runFfprobe(outGif);
  const aStreamsGif = probeGif.streams.filter((s) => s.codec_type === "audio");
  console.log(`  ✓ Exported GIF: ${outGif} (audio streams: ${aStreamsGif.length})`);
  if (aStreamsGif.length !== 0) {
    throw new Error("GIF output unexpectedly contains audio stream!");
  }

  console.log(`\n🎉 [Smoke Test] ALL 6 CHECKS PASSED PERFECTLY!`);
} finally {
  try {
    fs.rmSync(tempDir, { recursive: true, force: true });
    console.log(`[Smoke Test] Cleaned up temporary files.`);
  } catch {
    // ignore
  }
}
