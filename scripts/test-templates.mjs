import assert from "node:assert/strict";

// Mock localStorage environment for headless testing
const mockStorage = new Map();
globalThis.window = {
  localStorage: {
    getItem(key) {
      return mockStorage.has(key) ? mockStorage.get(key) : null;
    },
    setItem(key, value) {
      mockStorage.set(key, String(value));
    },
    removeItem(key) {
      mockStorage.delete(key);
    },
    clear() {
      mockStorage.clear();
    },
  },
};
globalThis.localStorage = globalThis.window.localStorage;

// Dynamically import the compiled module or TS module via tsx / esbuild or mock testing
import {
  BUILTIN_TEMPLATES,
  getCustomTemplates,
  saveCustomTemplate,
  deleteCustomTemplate,
  getAllTemplates,
  saveDraftToStorage,
  loadDraftFromStorage,
  clearDraftFromStorage,
} from "../src/lib/templates.ts";

console.log("--- Testing BUILTIN_TEMPLATES ---");
assert.equal(BUILTIN_TEMPLATES.length, 3, "Should have exactly 3 builtin templates");
assert.equal(BUILTIN_TEMPLATES[0].id, "builtin-subtitles-bar");
assert.equal(BUILTIN_TEMPLATES[1].id, "builtin-top-banner");
assert.equal(BUILTIN_TEMPLATES[2].id, "builtin-impact-headline");
console.log("✓ Builtin templates verified.");

console.log("--- Testing Custom Template CRUD ---");
mockStorage.clear();
assert.equal(getCustomTemplates().length, 0);

const saved = saveCustomTemplate({
  name: "我的测试模板",
  description: "测试描述",
  badge: "测试",
  textStyle: {
    defaultText: "测试文案",
    size: 28,
    color: "#ff0000",
    strokeWidth: 2,
    strokeColor: "#000000",
    useBox: true,
    boxColor: "#111111",
    boxOpacity: 0.7,
    positionRatio: { x: 0.5, y: 0.9 },
  },
});

assert.ok(saved.id.startsWith("custom-"));
assert.equal(saved.category, "custom");
assert.equal(getCustomTemplates().length, 1);
assert.equal(getAllTemplates().length, 4);

deleteCustomTemplate(saved.id);
assert.equal(getCustomTemplates().length, 0);
assert.equal(getAllTemplates().length, 3);
console.log("✓ Custom templates CRUD verified.");

console.log("--- Testing Draft Persistence ---");
mockStorage.clear();
assert.equal(loadDraftFromStorage(), null);

const draftSample = {
  version: 1,
  updatedAt: Date.now(),
  inputPath: "/tmp/sample.mp4",
  outputPath: "/tmp/sample.gif",
  startSecs: 1.2,
  endSecs: 3.5,
  useCrop: true,
  crop: { x: 10, y: 10, width: 300, height: 300 },
  outputWidth: 320,
  fps: 15,
  quality: "high",
  format: "webp",
  selectedPresetId: "large",
  useText: true,
  text: "草稿台词",
  textX: 30,
  textY: 50,
  textSize: 32,
  textColor: "#ffffff",
  textStrokeWidth: 3,
  textStrokeColor: "#000000",
  textUseBox: true,
  textBoxColor: "#000000",
  textBoxOpacity: 0.65,
  useImage: false,
  imagePath: "",
  imageX: 0,
  imageY: 0,
  imageWidth: 100,
};

saveDraftToStorage(draftSample);
const restored = loadDraftFromStorage();
assert.ok(restored !== null);
assert.equal(restored.inputPath, "/tmp/sample.mp4");
assert.equal(restored.text, "草稿台词");
assert.equal(restored.format, "webp");

clearDraftFromStorage();
assert.equal(loadDraftFromStorage(), null);
console.log("✓ Draft persistence & restoration verified.");

console.log("ALL TESTS PASSED SUCCESSFULLY!");
