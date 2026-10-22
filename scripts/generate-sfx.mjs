#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");
const sfxDir = path.resolve(rootDir, "src-tauri", "sfx");

if (!fs.existsSync(sfxDir)) {
  fs.mkdirSync(sfxDir, { recursive: true });
}

// 8 款经典高频二创音效定义
const soundEffects = [
  {
    filename: "censor_bleep.mp3",
    name: "消音哔哔",
    filter: "sine=frequency=1000:duration=0.8,afade=t=in:ss=0:d=0.01,afade=t=out:st=0.75:d=0.05,volume=0.8",
  },
  {
    filename: "crisp_ding.mp3",
    name: "绝了清脆叮",
    filter: "sine=frequency=1760:duration=0.8,afade=t=in:ss=0:d=0.005,afade=t=out:st=0.05:d=0.75,volume=0.85",
  },
  {
    filename: "boom_808.mp3",
    name: "震撼低音808",
    filter: "sine=frequency=80:duration=1.2,afade=t=in:ss=0:d=0.01,afade=t=out:st=0.2:d=1.0,volume=1.0",
  },
  {
    filename: "alarm_danger.mp3",
    name: "高能警报",
    filter: "sine=frequency=800:duration=1.5,vibrato=f=5:d=0.6,afade=t=in:ss=0:d=0.02,afade=t=out:st=1.3:d=0.2,volume=0.8",
  },
  {
    filename: "boing_spring.mp3",
    name: "弹簧搞笑",
    filter: "sine=frequency=240:duration=0.8,vibrato=f=12:d=0.8,afade=t=in:ss=0:d=0.01,afade=t=out:st=0.2:d=0.6,volume=0.85",
  },
  {
    filename: "victory_cheer.mp3",
    name: "欢呼和弦",
    filter: "sine=frequency=523.25:duration=1.6,vibrato=f=4:d=0.3,afade=t=in:ss=0:d=0.05,afade=t=out:st=1.0:d=0.6,volume=0.85",
  },
  {
    filename: "face_slap.mp3",
    name: "打脸啪啪",
    filter: "anoisesrc=d=0.35:c=white:r=44100:a=0.9,afade=t=out:st=0.01:d=0.34,volume=0.9",
  },
  {
    filename: "goofy_laugh.mp3",
    name: "滑稽笑声",
    filter: "sine=frequency=440:duration=1.5,tremolo=f=7:d=0.8,afade=t=in:ss=0:d=0.02,afade=t=out:st=1.2:d=0.3,volume=0.85",
  },
];

console.log("Generating 8 built-in sound effects into src-tauri/sfx/ ...");

for (const sfx of soundEffects) {
  const targetPath = path.join(sfxDir, sfx.filename);
  console.log(`- Synthesizing ${sfx.name} (${sfx.filename})...`);
  execFileSync("ffmpeg", [
    "-y",
    "-f",
    "lavfi",
    "-i",
    sfx.filter,
    "-c:a",
    "libmp3lame",
    "-b:a",
    "128k",
    targetPath,
  ]);
  const stat = fs.statSync(targetPath);
  console.log(`  Done: ${(stat.size / 1024).toFixed(1)} KB`);
}

console.log("All 8 built-in sound effects generated successfully!");
