import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const tmpDir = path.resolve("./target/tmp-advanced-text-smoke");
fs.mkdirSync(tmpDir, { recursive: true });

const ffmpegBinary = fs.existsSync(path.resolve("./src-tauri/binaries/ffmpeg-aarch64-apple-darwin"))
  ? path.resolve("./src-tauri/binaries/ffmpeg-aarch64-apple-darwin")
  : "ffmpeg";
const ffprobeBinary = fs.existsSync(path.resolve("./src-tauri/binaries/ffprobe-aarch64-apple-darwin"))
  ? path.resolve("./src-tauri/binaries/ffprobe-aarch64-apple-darwin")
  : "ffprobe";

const testVideoPath = path.join(tmpDir, "source.mp4");

console.log("1. Generating test source video...");
execFileSync(ffmpegBinary, [
  "-y",
  "-f", "lavfi",
  "-i", "testsrc=size=320x240:rate=15:duration=2",
  "-pix_fmt", "yuv420p",
  testVideoPath,
]);
assert.ok(fs.existsSync(testVideoPath), "Test source video must be created");

console.log("2. Testing Rotated + Italic + Drop Shadow Text -> GIF...");
const outGif = path.join(tmpDir, "advanced_rot_italic.gif");
const gifFilter = [
  "[0:v]scale=320:-1:flags=lanczos,fps=12[v0]",
  "color=c=black@0.0:s=240x120:d=1.50,format=rgba,drawtext=text='ROTATED 45':fontsize=28:fontcolor=0xFFFFFF:borderw=2:bordercolor=0x000000:shadowx=4:shadowy=4:shadowcolor=0x222222:x=(w-text_w)/2:y=(h-text_h)/2,shear=shx=-0.22:c=none,rotate=45*PI/180:c=none:ow=rotw(45*PI/180):oh=roth(45*PI/180)[t_1]",
  "[v0][t_1]overlay=160-w/2:100-h/2:eof_action=pass[v_1]",
  "[v_1]split[gif][palette]",
  "[palette]palettegen=stats_mode=diff[p]",
  "[gif][p]paletteuse=dither=sierra2_4a[vout]",
].join(";");

execFileSync(ffmpegBinary, [
  "-y", "-ss", "0.0", "-t", "1.5", "-i", testVideoPath,
  "-filter_complex", gifFilter,
  "-map", "[vout]",
  outGif,
]);
assert.ok(fs.existsSync(outGif), "Rotated GIF must exist");
assert.ok(fs.statSync(outGif).size > 1000, "Rotated GIF size must be valid");
console.log(`✓ Rotated + Italic + Shadow GIF export OK (${fs.statSync(outGif).size} bytes)`);

console.log("3. Testing Curved Arc Text -> WebP...");
const outWebp = path.join(tmpDir, "advanced_arc.webp");
const arcFilter = [
  "[0:v]scale=320:-1:flags=lanczos,fps=12[v0]",
  "color=c=black@0.0:s=60x60:d=1.50,format=rgba,drawtext=text='高':fontsize=26:fontcolor=0xFFFF00:x=(w-text_w)/2:y=(h-text_h)/2,rotate=-25*PI/180:c=none:ow=rotw(-25*PI/180):oh=roth(-25*PI/180)[tc_1_0]",
  "[v0][tc_1_0]overlay=100-w/2:70-h/2:eof_action=pass[v_1_0]",
  "color=c=black@0.0:s=60x60:d=1.50,format=rgba,drawtext=text='能':fontsize=26:fontcolor=0xFFFF00:x=(w-text_w)/2:y=(h-text_h)/2,rotate=-10*PI/180:c=none:ow=rotw(-10*PI/180):oh=roth(-10*PI/180)[tc_1_1]",
  "[v_1_0][tc_1_1]overlay=135-w/2:55-h/2:eof_action=pass[v_1_1]",
  "color=c=black@0.0:s=60x60:d=1.50,format=rgba,drawtext=text='预':fontsize=26:fontcolor=0xFFFF00:x=(w-text_w)/2:y=(h-text_h)/2,rotate=10*PI/180:c=none:ow=rotw(10*PI/180):oh=roth(10*PI/180)[tc_1_2]",
  "[v_1_1][tc_1_2]overlay=170-w/2:55-h/2:eof_action=pass[v_1_2]",
  "color=c=black@0.0:s=60x60:d=1.50,format=rgba,drawtext=text='警':fontsize=26:fontcolor=0xFFFF00:x=(w-text_w)/2:y=(h-text_h)/2,rotate=25*PI/180:c=none:ow=rotw(25*PI/180):oh=roth(25*PI/180)[tc_1_3]",
  "[v_1_2][tc_1_3]overlay=205-w/2:70-h/2:eof_action=pass[vout]",
].join(";");

execFileSync(ffmpegBinary, [
  "-y", "-ss", "0.0", "-t", "1.5", "-i", testVideoPath,
  "-filter_complex", arcFilter,
  "-map", "[vout]",
  "-c:v", "libwebp", "-q:v", "75", "-loop", "0", "-an",
  outWebp,
]);
assert.ok(fs.existsSync(outWebp), "Curved Arc WebP must exist");
assert.ok(fs.statSync(outWebp).size > 1000, "Arc WebP size must be valid");
console.log(`✓ Curved Arc WebP export OK (${fs.statSync(outWebp).size} bytes)`);

console.log("4. Testing Multi-layer Compound Effects -> MP4...");
const outMp4 = path.join(tmpDir, "advanced_compound.mp4");
const compoundFilter = [
  "[0:v]scale=320:-2:flags=lanczos,fps=15[v0]",
  "color=c=black@0.0:s=200x100:d=1.50,format=rgba,drawtext=text='前方':fontsize=26:fontcolor=0xFF3333:borderw=2:bordercolor=0x000000:shadowx=3:shadowy=3:shadowcolor=0x111111:x=(w-text_w)/2:y=(h-text_h)/2,shear=shx=-0.22:c=none,rotate=-15*PI/180:c=none:ow=rotw(-15*PI/180):oh=roth(-15*PI/180)[t_1]",
  "[v0][t_1]overlay=80-w/2:60-h/2:eof_action=pass[v_1]",
  "[v_1]drawtext=text='高能弹幕来袭':fontsize=22:fontcolor=0xFFFFFF:borderw=1:bordercolor=0x000000:box=1:boxcolor=0x000000@0.65:boxborderw=8:shadowx=2:shadowy=2:shadowcolor=0x000000:x=40:y=180[v_2]",
  "[v_2]format=yuv420p[vout]",
].join(";");

execFileSync(ffmpegBinary, [
  "-y", "-ss", "0.0", "-t", "1.5", "-i", testVideoPath,
  "-filter_complex", compoundFilter,
  "-map", "[vout]",
  "-c:v", "libx264", "-preset", "veryfast", "-crf", "24",
  "-an",
  outMp4,
]);
assert.ok(fs.existsSync(outMp4), "Compound MP4 must exist");
assert.ok(fs.statSync(outMp4).size > 1000, "Compound MP4 size must be valid");

// Probe MP4
const probeOut = execFileSync(ffprobeBinary, [
  "-v", "error",
  "-select_streams", "v:0",
  "-show_entries", "stream=codec_name,width,height",
  "-of", "json",
  outMp4,
]);
const probeInfo = JSON.parse(probeOut.toString());
assert.equal(probeInfo.streams[0].codec_name, "h264");
console.log(`✓ Multi-layer Compound MP4 export OK (${fs.statSync(outMp4).size} bytes, codec: ${probeInfo.streams[0].codec_name})`);

fs.rmSync(tmpDir, { recursive: true, force: true });
console.log("🎉 ALL ADVANCED TEXT EFFECTS SMOKE TESTS PASSED!");
