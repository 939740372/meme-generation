import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const stickersDir = path.resolve(__dirname, "../src-tauri/stickers");

const EXPECTED_STICKER_IDS = [
  "deal_with_it",
  "red_question",
  "confused_questions",
  "doge_face",
  "panda_face",
  "clown_nose",
  "sweat_smile",
  "approved_stamp",
  "rejected_stamp",
  "anger_vein",
  "sweat_drop",
  "sparkle_stars",
  "heart_eyes",
  "fire_flame",
  "crying_tears",
  "thumbs_up",
  "shout_bubble",
  "dots_bubble",
];

console.log("--- Testing Sticker Files Integrity ---");
assert.ok(fs.existsSync(stickersDir), "src-tauri/stickers directory must exist");

const files = fs.readdirSync(stickersDir).filter((f) => f.endsWith(".png"));
assert.equal(files.length, 18, `Expected exactly 18 PNG stickers, found ${files.length}`);

for (const id of EXPECTED_STICKER_IDS) {
  const filePath = path.join(stickersDir, `${id}.png`);
  assert.ok(fs.existsSync(filePath), `Sticker file ${id}.png must exist`);

  const buffer = fs.readFileSync(filePath);
  assert.ok(buffer.length > 500, `Sticker ${id}.png must not be empty or truncated`);

  // Verify PNG Magic Number: 0x89 0x50 0x4E 0x47 0x0D 0x0A 0x1A 0x0A
  const isPng =
    buffer[0] === 0x89 &&
    buffer[1] === 0x50 &&
    buffer[2] === 0x4e &&
    buffer[3] === 0x47 &&
    buffer[4] === 0x0d &&
    buffer[5] === 0x0a &&
    buffer[6] === 0x1a &&
    buffer[7] === 0x0a;
  assert.ok(isPng, `File ${id}.png must be a valid PNG format`);

  // Verify IHDR Chunk (starts at offset 12: Length (4), "IHDR" (4), Width (4), Height (4), BitDepth (1), ColorType (1))
  const ihdrType = buffer.toString("ascii", 12, 16);
  assert.equal(ihdrType, "IHDR", `IHDR chunk must be first chunk in ${id}.png`);

  const width = buffer.readUInt32BE(16);
  const height = buffer.readUInt32BE(20);
  const colorType = buffer[25]; // 6 means RGBA

  assert.equal(width, 256, `${id}.png width should be 256px, got ${width}`);
  assert.equal(height, 256, `${id}.png height should be 256px, got ${height}`);
  assert.equal(colorType, 6, `${id}.png must have RGBA color type (6) with alpha channel, got ${colorType}`);

  console.log(`✓ ${id}.png: 256x256 RGBA transparent PNG (${(buffer.length / 1024).toFixed(1)} KB)`);
}

console.log("ALL 18 STICKERS VALIDATED SUCCESSFULLY!");
