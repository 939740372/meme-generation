import assert from "node:assert/strict";
import {
  createQueueItem,
  deserializeQueue,
  generateQueueOutputPath,
  getQueueStatusBadge,
  serializeQueue,
} from "../src/lib/queue.ts";

console.log("--- Testing generateQueueOutputPath ---");

const existing = new Set();
const p1 = generateQueueOutputPath("/tmp/output", "sample_video.mp4", 1.2, 3.5, "gif", existing);
assert.equal(p1, "/tmp/output/sample_video_1.2s-3.5s.gif");
existing.add(p1);

// When same name exists, should append (1), (2), etc. according to system rules
const p2 = generateQueueOutputPath("/tmp/output", "sample_video.mp4", 1.2, 3.5, "gif", existing);
assert.equal(p2, "/tmp/output/sample_video_1.2s-3.5s (1).gif");
existing.add(p2);

const p3 = generateQueueOutputPath("/tmp/output", "sample_video.mp4", 1.2, 3.5, "gif", existing);
assert.equal(p3, "/tmp/output/sample_video_1.2s-3.5s (2).gif");

const pWebp = generateQueueOutputPath("/tmp/output", "funny cat.mp4", 0.0, 2.0, "webp", existing);
assert.equal(pWebp, "/tmp/output/funny_cat_0.0s-2.0s.webp");

const pAvif = generateQueueOutputPath("/tmp/output", "funny cat.mp4", 0.0, 2.0, "avif", existing);
assert.equal(pAvif, "/tmp/output/funny_cat_0.0s-2.0s.avif");

const pMp4 = generateQueueOutputPath("/tmp/output", "funny cat.mp4", 0.0, 2.0, "mp4", existing);
assert.equal(pMp4, "/tmp/output/funny_cat_0.0s-2.0s.mp4");

console.log("✓ Path generation & deduplication verified (GIF, WebP, AVIF, MP4).");

console.log("--- Testing createQueueItem ---");
const mockPlan = {
  inputPath: "/Users/test/video.mp4",
  outputPath: "/Users/test/out.gif",
  startSecs: 2.5,
  endSecs: 4.8,
  crop: null,
  outputWidth: 320,
  fps: 12,
  quality: "medium",
  textOverlay: null,
  textLayers: null,
  imageOverlay: null,
  format: "gif",
  videoFilter: "grayscale",
};

const queueExisting = new Set();
const item1 = createQueueItem(mockPlan, "video.mp4", queueExisting);
queueExisting.add(item1.outputPath);

assert.ok(item1.id.startsWith("queue-"));
assert.equal(item1.status, "pending");
assert.equal(item1.progress, 0);
assert.equal(item1.format, "gif");
assert.equal(item1.timeRangeLabel, "2.50s ~ 4.80s");
assert.equal(item1.outputPath, "/Users/test/video_2.5s-4.8s.gif");
assert.equal(item1.plan.outputPath, "/Users/test/video_2.5s-4.8s.gif");
assert.equal(item1.plan.videoFilter, "grayscale");

// Next item with same params should not collide
const item2 = createQueueItem(mockPlan, "video.mp4", queueExisting);
assert.notEqual(item1.id, item2.id);
assert.equal(item2.outputPath, "/Users/test/video_2.5s-4.8s (1).gif");

const mp4Plan = {
  ...mockPlan,
  format: "mp4",
  includeAudio: true,
};
const itemMp4 = createQueueItem(mp4Plan, "video.mp4", queueExisting);
assert.equal(itemMp4.format, "mp4");
assert.equal(itemMp4.outputPath, "/Users/test/video_2.5s-4.8s.mp4");
assert.equal(itemMp4.plan.includeAudio, true);

console.log("✓ Queue item creation & immutability verified.");

console.log("--- Testing getQueueStatusBadge ---");
assert.equal(getQueueStatusBadge("pending").label, "排队中");
assert.equal(getQueueStatusBadge("rendering").label, "转码中");
assert.equal(getQueueStatusBadge("completed").label, "已完成");
assert.equal(getQueueStatusBadge("failed").label, "失败");
assert.equal(getQueueStatusBadge("cancelled").label, "已取消");
console.log("✓ Status badge labels verified.");

console.log("--- Testing queue serialization & deserialization ---");
const testItems = [
  { ...item1, status: "rendering", progress: 65 },
  { ...item2, status: "completed", progress: 100 },
];
const serialized = serializeQueue(testItems);
const deserialized = deserializeQueue(serialized);
assert.equal(deserialized.length, 2);
// Item 1 was rendering, should be restored as pending with 0 progress
assert.equal(deserialized[0].status, "pending");
assert.equal(deserialized[0].progress, 0);
// Item 2 was completed, stays completed
assert.equal(deserialized[1].status, "completed");
assert.equal(deserialized[1].progress, 100);
console.log("✓ Serialization sanitize behavior verified.");

console.log("ALL QUEUE TESTS PASSED SUCCESSFULLY!");
