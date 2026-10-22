<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from "vue";
import {
  cancelRender,
  copyFileToClipboard,
  getDesktopInfo,
  isDesktopRuntime,
  listenForRenderEvents,
  pickMediaOutput,
  pickPng,
  pickVideo,
  probeMedia,
  readFileAsDataUrl,
  renderGif,
  revealFileInDir,
  type DesktopInfo,
  type ExportRecord,
  type MediaInfo,
  type RenderPlan,
} from "./lib/desktop";
import {
  BUILTIN_TEMPLATES,
  clearDraftFromStorage,
  deleteCustomTemplate,
  getCustomTemplates,
  loadDraftFromStorage,
  saveCustomTemplate,
  saveDraftToStorage,
  type CanvasTextLayer,
  type MemeTemplate,
  type StudioDraft,
} from "./lib/templates";

type TaskStatus =
  | "idle"
  | "probing"
  | "ready"
  | "rendering"
  | "completed"
  | "failed"
  | "cancelled";

const desktopInfo = ref<DesktopInfo | null>(null);
const media = ref<MediaInfo | null>(null);
const inputPath = ref("");
const outputPath = ref("");
const exportFormat = ref<"gif" | "webp">("gif");
const status = ref<TaskStatus>("idle");
const progress = ref(0);
const statusMessage = ref("选择一个视频开始制作。");
const errorMessage = ref("");
const taskId = ref<string | null>(null);
const outputBytes = ref(0);
const stopListening = ref<(() => void) | null>(null);

const drawerTab = ref<"history" | "templates">("history");
const customTemplates = ref<MemeTemplate[]>([]);
const isSaveTemplateModalOpen = ref(false);
const newTemplateName = ref("");
const templateNameInputRef = ref<HTMLInputElement | null>(null);

const allTemplatesList = computed(() => [
  ...BUILTIN_TEMPLATES,
  ...customTemplates.value,
]);

const allTemplatesCount = computed(() => allTemplatesList.value.length);

const startSecs = ref(0);
const endSecs = ref(3);
const useCrop = ref(false);
const crop = reactive({ x: 0, y: 0, width: 480, height: 270 });
const outputWidth = ref(480);
const fps = ref(12);

export type PresetQuality = "low" | "medium" | "high";

export interface ExportPreset {
  id: string;
  name: string;
  badge: string;
  width: number;
  fps: number;
  quality: PresetQuality;
  description: string;
}

const PRESETS: ExportPreset[] = [
  {
    id: "wechat-small",
    name: "微信表情",
    badge: "< 1MB",
    width: 240,
    fps: 10,
    quality: "low",
    description: "微信聊天窗口表情包，体积严格压缩",
  },
  {
    id: "wechat-large",
    name: "大表情",
    badge: "< 5MB",
    width: 320,
    fps: 12,
    quality: "medium",
    description: "微信大表情与动态贴纸，画质与体积均衡",
  },
  {
    id: "hd-social",
    name: "高清动图",
    badge: "高清",
    width: 480,
    fps: 15,
    quality: "high",
    description: "Discord、Twitter、小红书高清分享",
  },
  {
    id: "lite",
    name: "轻量秒发",
    badge: "秒开",
    width: 180,
    fps: 8,
    quality: "low",
    description: "极小文件体积，弱网环境极速加载",
  },
];

const selectedPresetId = ref<string>("wechat-large");
const currentQuality = ref<PresetQuality>("medium");

function applyPreset(preset: ExportPreset): void {
  selectedPresetId.value = preset.id;
  outputWidth.value = preset.width;
  fps.value = preset.fps;
  currentQuality.value = preset.quality;
  if (media.value && useCrop.value) {
    crop.width = Math.min(media.value.width, preset.width);
    crop.height = Math.min(media.value.height, Math.round(crop.width * 0.5625));
  }
}

function onManualParameterChange(): void {
  const matched = PRESETS.find(
    (p) => p.width === outputWidth.value && p.fps === fps.value && p.quality === currentQuality.value,
  );
  selectedPresetId.value = matched ? matched.id : "custom";
}

const currentPresetInfo = computed(() => {
  if (selectedPresetId.value === "custom") {
    return { name: "自定义参数", badge: "自定", description: "手动调整的分辨率与帧率" };
  }
  return PRESETS.find((p) => p.id === selectedPresetId.value) ?? {
    name: "标准表情",
    badge: "平衡",
    description: "标准导出参数",
  };
});

const useText = ref(false);
const textLayers = ref<CanvasTextLayer[]>([
  {
    id: "layer-1",
    text: "表情工坊",
    x: 24,
    y: 24,
    size: 42,
    color: "#ffffff",
    strokeWidth: 2,
    strokeColor: "#000000",
    useBox: false,
    boxColor: "#000000",
    boxOpacity: 0.60,
    visible: true,
  },
]);
const activeLayerId = ref<string>("layer-1");

const activeLayer = computed(() => {
  return textLayers.value.find((l) => l.id === activeLayerId.value) || textLayers.value[0];
});

function addTextLayer(): void {
  if (textLayers.value.length >= 4) return;
  const newIndex = textLayers.value.length + 1;
  const id = `layer-${Date.now()}`;
  const defaultY = media.value
    ? Math.min(media.value.height - 40, 24 + (newIndex - 1) * 60)
    : 24 + (newIndex - 1) * 60;
  const newLayer: CanvasTextLayer = {
    id,
    text: newIndex === 2 ? "第二行吐槽" : `文字图层 ${newIndex}`,
    x: 24,
    y: defaultY,
    size: 32,
    color: "#ffffff",
    strokeWidth: 2,
    strokeColor: "#000000",
    useBox: false,
    boxColor: "#000000",
    boxOpacity: 0.60,
    visible: true,
  };
  textLayers.value.push(newLayer);
  activeLayerId.value = id;
}

function removeTextLayer(id: string): void {
  if (textLayers.value.length <= 1) return;
  const index = textLayers.value.findIndex((l) => l.id === id);
  if (index === -1) return;
  textLayers.value.splice(index, 1);
  if (activeLayerId.value === id) {
    const nextLayer = textLayers.value[Math.max(0, index - 1)];
    activeLayerId.value = nextLayer ? nextLayer.id : textLayers.value[0].id;
  }
}

function toggleLayerVisibility(layer: CanvasTextLayer): void {
  layer.visible = !layer.visible;
}

const text = computed({
  get: () => activeLayer.value?.text ?? "",
  set: (v: string) => { if (activeLayer.value) activeLayer.value.text = v; },
});
const textX = computed({
  get: () => activeLayer.value?.x ?? 0,
  set: (v: number) => { if (activeLayer.value) activeLayer.value.x = v; },
});
const textY = computed({
  get: () => activeLayer.value?.y ?? 0,
  set: (v: number) => { if (activeLayer.value) activeLayer.value.y = v; },
});
const textSize = computed({
  get: () => activeLayer.value?.size ?? 28,
  set: (v: number) => { if (activeLayer.value) activeLayer.value.size = v; },
});
const textColor = computed({
  get: () => activeLayer.value?.color ?? "#ffffff",
  set: (v: string) => { if (activeLayer.value) activeLayer.value.color = v; },
});
const textStrokeWidth = computed({
  get: () => activeLayer.value?.strokeWidth ?? 0,
  set: (v: number) => { if (activeLayer.value) activeLayer.value.strokeWidth = v; },
});
const textStrokeColor = computed({
  get: () => activeLayer.value?.strokeColor ?? "#000000",
  set: (v: string) => { if (activeLayer.value) activeLayer.value.strokeColor = v; },
});
const textUseBox = computed({
  get: () => activeLayer.value?.useBox ?? false,
  set: (v: boolean) => { if (activeLayer.value) activeLayer.value.useBox = v; },
});
const textBoxColor = computed({
  get: () => activeLayer.value?.boxColor ?? "#000000",
  set: (v: string) => { if (activeLayer.value) activeLayer.value.boxColor = v; },
});
const textBoxOpacity = computed({
  get: () => activeLayer.value?.boxOpacity ?? 0.6,
  set: (v: number) => { if (activeLayer.value) activeLayer.value.boxOpacity = v; },
});

const useImage = ref(false);
const imagePath = ref("");
const imageX = ref(24);
const imageY = ref(24);
const imageWidth = ref(120);

const stageRef = ref<HTMLElement | null>(null);
const timelineTrackRef = ref<HTMLElement | null>(null);
const stageSize = reactive({ width: 0, height: 0 });
const isDraggingText = ref(false);
const isDraggingImage = ref(false);

const startPercent = computed(() => {
  const duration = media.value?.durationSecs ?? 0;
  return duration > 0 ? (startSecs.value / duration) * 100 : 0;
});

const endPercent = computed(() => {
  const duration = media.value?.durationSecs ?? 0;
  return duration > 0 ? (endSecs.value / duration) * 100 : 100;
});

const rangePercent = computed(() => Math.max(0, endPercent.value - startPercent.value));

const selectedDuration = computed(() => Math.max(0, endSecs.value - startSecs.value));
const selectedDurationLabel = computed(() => `${selectedDuration.value.toFixed(2)} 秒`);

function hexToRgba(hex: string, opacity: number): string {
  const clean = hex.replace("#", "");
  const r = parseInt(clean.substring(0, 2), 16) || 0;
  const g = parseInt(clean.substring(2, 4), 16) || 0;
  const b = parseInt(clean.substring(4, 6), 16) || 0;
  return `rgba(${r}, ${g}, ${b}, ${opacity})`;
}

function getTextLayerStyle(layer: CanvasTextLayer): Record<string, string | number | undefined> {
  let left = layer.x;
  let top = layer.y;
  let fontSize = layer.size;
  let strokeW = layer.strokeWidth;

  if (media.value && stageSize.width > 0 && stageSize.height > 0) {
    const scaleX = stageSize.width / media.value.width;
    const scaleY = stageSize.height / media.value.height;
    left = Math.round(layer.x * scaleX);
    top = Math.round(layer.y * scaleY);
    fontSize = Math.max(12, Math.min(Math.round(layer.size * scaleX), 72));
    if (strokeW > 0) {
      strokeW = Math.max(1, Math.round(layer.strokeWidth * scaleX));
    }
  }

  const bg = layer.useBox ? hexToRgba(layer.boxColor, layer.boxOpacity) : "transparent";

  const styles: Record<string, string | number | undefined> = {
    left: `${left}px`,
    top: `${top}px`,
    color: layer.color,
    fontSize: `${fontSize}px`,
    backgroundColor: bg,
    paintOrder: "stroke fill",
  };
  if (strokeW > 0) {
    styles["-webkit-text-stroke"] = `${strokeW}px ${layer.strokeColor}`;
  }
  return styles;
}

const imageDisplayLeft = computed(() => {
  if (!media.value || stageSize.width <= 0) return imageX.value;
  return imageX.value * (stageSize.width / media.value.width);
});

const imageDisplayTop = computed(() => {
  if (!media.value || stageSize.height <= 0) return imageY.value;
  return imageY.value * (stageSize.height / media.value.height);
});

const imageDisplayWidth = computed(() => {
  if (!media.value || stageSize.width <= 0) return imageWidth.value;
  return Math.max(28, imageWidth.value * (stageSize.width / media.value.width));
});

function updateStageSize(): void {
  if (stageRef.value) {
    stageSize.width = stageRef.value.clientWidth;
    stageSize.height = stageRef.value.clientHeight;
  }
}

function startDragStartHandle(event: PointerEvent): void {
  if (!timelineTrackRef.value || !media.value) return;
  event.preventDefault();
  const trackRect = timelineTrackRef.value.getBoundingClientRect();
  const duration = media.value.durationSecs;

  const onPointerMove = (e: PointerEvent): void => {
    const offsetX = e.clientX - trackRect.left;
    const ratio = Math.max(0, Math.min(1, offsetX / trackRect.width));
    const target = ratio * duration;
    startSecs.value = Number(Math.max(0, Math.min(endSecs.value - 0.1, target)).toFixed(2));
  };

  const onPointerUp = (): void => {
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);
  };

  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
}

function startDragEndHandle(event: PointerEvent): void {
  if (!timelineTrackRef.value || !media.value) return;
  event.preventDefault();
  const trackRect = timelineTrackRef.value.getBoundingClientRect();
  const duration = media.value.durationSecs;

  const onPointerMove = (e: PointerEvent): void => {
    const offsetX = e.clientX - trackRect.left;
    const ratio = Math.max(0, Math.min(1, offsetX / trackRect.width));
    const target = ratio * duration;
    endSecs.value = Number(Math.min(duration, Math.max(startSecs.value + 0.1, target)).toFixed(2));
  };

  const onPointerUp = (): void => {
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);
  };

  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
}

function startDragRange(event: PointerEvent): void {
  if (!timelineTrackRef.value || !media.value) return;
  event.preventDefault();
  const trackRect = timelineTrackRef.value.getBoundingClientRect();
  const duration = media.value.durationSecs;
  const windowDuration = endSecs.value - startSecs.value;
  const startClientX = event.clientX;
  const initStart = startSecs.value;

  const onPointerMove = (e: PointerEvent): void => {
    const deltaX = e.clientX - startClientX;
    const deltaSec = (deltaX / trackRect.width) * duration;
    let newStart = initStart + deltaSec;
    if (newStart < 0) newStart = 0;
    if (newStart + windowDuration > duration) newStart = duration - windowDuration;
    startSecs.value = Number(newStart.toFixed(2));
    endSecs.value = Number((newStart + windowDuration).toFixed(2));
  };

  const onPointerUp = (): void => {
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);
  };

  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
}

function onTimelineTrackClick(event: MouseEvent): void {
  if (!timelineTrackRef.value || !media.value) return;
  const trackRect = timelineTrackRef.value.getBoundingClientRect();
  const ratio = Math.max(0, Math.min(1, (event.clientX - trackRect.left) / trackRect.width));
  const clickSec = ratio * media.value.durationSecs;

  if (Math.abs(clickSec - startSecs.value) < Math.abs(clickSec - endSecs.value)) {
    startSecs.value = Number(Math.min(endSecs.value - 0.1, Math.max(0, clickSec)).toFixed(2));
  } else {
    endSecs.value = Number(Math.max(startSecs.value + 0.1, Math.min(media.value.durationSecs, clickSec)).toFixed(2));
  }
}

function startDragText(layer: CanvasTextLayer, event: PointerEvent): void {
  if (!media.value || !stageRef.value) return;
  event.preventDefault();
  event.stopPropagation();
  activeLayerId.value = layer.id;
  isDraggingText.value = true;
  const startClientX = event.clientX;
  const startClientY = event.clientY;
  const initX = layer.x;
  const initY = layer.y;
  const scaleX = stageRef.value.clientWidth / media.value.width;
  const scaleY = stageRef.value.clientHeight / media.value.height;

  const onPointerMove = (e: PointerEvent): void => {
    const deltaX = (e.clientX - startClientX) / scaleX;
    const deltaY = (e.clientY - startClientY) / scaleY;
    const newX = Math.round(initX + deltaX);
    const newY = Math.round(initY + deltaY);
    layer.x = Math.max(0, Math.min(media.value!.width - 20, newX));
    layer.y = Math.max(0, Math.min(media.value!.height - 20, newY));
  };

  const onPointerUp = (): void => {
    isDraggingText.value = false;
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);
  };

  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
}

function startDragImage(event: PointerEvent): void {
  if (!media.value || !stageRef.value) return;
  event.preventDefault();
  event.stopPropagation();
  isDraggingImage.value = true;
  const startClientX = event.clientX;
  const startClientY = event.clientY;
  const initX = imageX.value;
  const initY = imageY.value;
  const scaleX = stageRef.value.clientWidth / media.value.width;
  const scaleY = stageRef.value.clientHeight / media.value.height;

  const onPointerMove = (e: PointerEvent): void => {
    const deltaX = (e.clientX - startClientX) / scaleX;
    const deltaY = (e.clientY - startClientY) / scaleY;
    const newX = Math.round(initX + deltaX);
    const newY = Math.round(initY + deltaY);
    imageX.value = Math.max(0, Math.min(media.value!.width - 20, newX));
    imageY.value = Math.max(0, Math.min(media.value!.height - 20, newY));
  };

  const onPointerUp = (): void => {
    isDraggingImage.value = false;
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);
  };

  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
}

const cropDisplayLeft = computed(() => {
  if (!media.value || stageSize.width <= 0) return crop.x;
  return crop.x * (stageSize.width / media.value.width);
});

const cropDisplayTop = computed(() => {
  if (!media.value || stageSize.height <= 0) return crop.y;
  return crop.y * (stageSize.height / media.value.height);
});

const cropDisplayWidth = computed(() => {
  if (!media.value || stageSize.width <= 0) return crop.width;
  return Math.max(16, crop.width * (stageSize.width / media.value.width));
});

const cropDisplayHeight = computed(() => {
  if (!media.value || stageSize.height <= 0) return crop.height;
  return Math.max(16, crop.height * (stageSize.height / media.value.height));
});

const isCropNearTop = computed(() => cropDisplayTop.value < 28);

const isDraggingCrop = ref(false);
const activeCropHandle = ref<string | null>(null);

function startDragCrop(event: PointerEvent): void {
  if (!media.value || !stageRef.value) return;
  event.preventDefault();
  event.stopPropagation();
  isDraggingCrop.value = true;
  activeCropHandle.value = "move";

  const startClientX = event.clientX;
  const startClientY = event.clientY;
  const initX = crop.x;
  const initY = crop.y;
  const scaleX = stageRef.value.clientWidth / media.value.width;
  const scaleY = stageRef.value.clientHeight / media.value.height;
  const maxX = Math.max(0, media.value.width - crop.width);
  const maxY = Math.max(0, media.value.height - crop.height);

  const onPointerMove = (e: PointerEvent): void => {
    const deltaX = (e.clientX - startClientX) / scaleX;
    const deltaY = (e.clientY - startClientY) / scaleY;
    crop.x = Math.max(0, Math.min(maxX, Math.round(initX + deltaX)));
    crop.y = Math.max(0, Math.min(maxY, Math.round(initY + deltaY)));
  };

  const onPointerUp = (): void => {
    isDraggingCrop.value = false;
    activeCropHandle.value = null;
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);
  };

  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
}

function startResizeCrop(handle: string, event: PointerEvent): void {
  if (!media.value || !stageRef.value) return;
  event.preventDefault();
  event.stopPropagation();
  isDraggingCrop.value = true;
  activeCropHandle.value = handle;

  const startClientX = event.clientX;
  const startClientY = event.clientY;
  const initX = crop.x;
  const initY = crop.y;
  const initW = crop.width;
  const initH = crop.height;
  const scaleX = stageRef.value.clientWidth / media.value.width;
  const scaleY = stageRef.value.clientHeight / media.value.height;
  const minW = 20;
  const minH = 20;
  const mediaW = media.value.width;
  const mediaH = media.value.height;

  const onPointerMove = (e: PointerEvent): void => {
    const deltaX = (e.clientX - startClientX) / scaleX;
    const deltaY = (e.clientY - startClientY) / scaleY;

    if (handle.includes("e")) {
      const maxW = mediaW - initX;
      crop.width = Math.max(minW, Math.min(maxW, Math.round(initW + deltaX)));
    } else if (handle.includes("w")) {
      const targetX = Math.round(initX + deltaX);
      const clampedX = Math.max(0, Math.min(initX + initW - minW, targetX));
      crop.x = clampedX;
      crop.width = Math.round(initW + (initX - clampedX));
    }

    if (handle.includes("s")) {
      const maxH = mediaH - initY;
      crop.height = Math.max(minH, Math.min(maxH, Math.round(initH + deltaY)));
    } else if (handle.includes("n")) {
      const targetY = Math.round(initY + deltaY);
      const clampedY = Math.max(0, Math.min(initY + initH - minH, targetY));
      crop.y = clampedY;
      crop.height = Math.round(initH + (initY - clampedY));
    }
  };

  const onPointerUp = (): void => {
    isDraggingCrop.value = false;
    activeCropHandle.value = null;
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);
  };

  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
}

function cropCenter(): void {
  if (!media.value) return;
  crop.x = Math.max(0, Math.floor((media.value.width - crop.width) / 2));
  crop.y = Math.max(0, Math.floor((media.value.height - crop.height) / 2));
}

function cropSquare(): void {
  if (!media.value) return;
  const side = Math.min(media.value.width, media.value.height, crop.width || 320);
  crop.width = side;
  crop.height = side;
  cropCenter();
}

function cropReset(): void {
  if (!media.value) return;
  crop.x = 0;
  crop.y = 0;
  crop.width = media.value.width;
  crop.height = media.value.height;
}

const isDesktop = computed(() => isDesktopRuntime());
const isBusy = computed(() => status.value === "probing" || status.value === "rendering");
const canRender = computed(
  () => Boolean(media.value && inputPath.value && outputPath.value && !isBusy.value),
);
const runtimeLabel = computed(() =>
  desktopInfo.value
    ? "已连接 Tauri"
    : "浏览器预览",
);
const mediaDuration = computed(() => media.value?.durationSecs ?? 0);
const mediaDurationLabel = computed(() =>
  media.value ? `${media.value.durationSecs.toFixed(2)} 秒` : "—",
);
const mediaFpsLabel = computed(() =>
  media.value?.fps ? `${media.value.fps.toFixed(2)} 帧/秒` : "未知帧率",
);
const statusLabel = computed(() => {
  const labels: Record<TaskStatus, string> = {
    idle: "等待导入",
    probing: "读取媒体信息",
    ready: "可以导出",
    rendering: "正在渲染",
    completed: "导出完成",
    failed: "导出失败",
    cancelled: "已取消",
  };
  return labels[status.value];
});
const renderErrorLabels: Record<string, string> = {
  FFMPEG_FAILED: "FFmpeg 导出失败，请检查输入文件、参数或 FFmpeg 配置。",
  PROCESS_STATUS_UNKNOWN: "无法读取 FFmpeg 进程状态。",
  OUTPUT_INVALID: "FFmpeg 未生成可用的 GIF 文件。",
  OUTPUT_NOT_READABLE: "导出的 GIF 无法读取。",
  TASK_NOT_FOUND: "渲染任务不存在或已经结束。",
  PROCESS_UNAVAILABLE: "无法访问 FFmpeg 进程。",
  CANCEL_FAILED: "取消 FFmpeg 任务失败。",
};
const previewStyle = computed(() => {
  if (!media.value) {
    return {};
  }
  return { aspectRatio: `${media.value.width} / ${media.value.height}` };
});

function basenameWithoutExtension(path: string): string {
  const fileName = path.split(/[\\/]/).pop() ?? "meme";
  return fileName.replace(/\.[^.]+$/, "") || "meme";
}

function defaultOutputPath(path: string, format: "gif" | "webp" = exportFormat.value): string {
  const separator = path.includes("\\") ? "\\" : "/";
  const parts = path.split(/[\\/]/);
  parts.pop();
  const parent = parts.join(separator);
  const fileName = `${basenameWithoutExtension(path)}-meme.${format}`;
  return parent ? `${parent}${separator}${fileName}` : fileName;
}

function ensureFormatExtension(path: string, format: "gif" | "webp" = exportFormat.value): string {
  const clean = path.trim();
  if (!clean) return "";
  const lower = clean.toLowerCase();
  if (lower.endsWith(`.${format}`)) {
    return clean;
  }
  if (lower.endsWith(".gif") || lower.endsWith(".webp")) {
    const idx = clean.lastIndexOf(".");
    return `${clean.substring(0, idx)}.${format}`;
  }
  return `${clean}.${format}`;
}

function setExportFormat(fmt: "gif" | "webp"): void {
  exportFormat.value = fmt;
  if (outputPath.value.trim()) {
    outputPath.value = ensureFormatExtension(outputPath.value.trim(), fmt);
  }
}

function readableError(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error && typeof error === "object" && "message" in error) {
    return String(error.message);
  }
  return "操作失败，请检查文件和 FFmpeg 配置后重试。";
}

function readableRenderError(code: string, message: string): string {
  return renderErrorLabels[code] ?? (message || "渲染失败，请检查参数后重试。");
}

function applyMediaDefaults(nextMedia: MediaInfo): void {
  const suggestedEnd = Math.min(nextMedia.durationSecs, 3);
  endSecs.value = Number(suggestedEnd.toFixed(2));
  applyPreset(PRESETS[1]); // 默认应用微信大表情 (320px, 12fps)
  crop.width = Math.min(nextMedia.width, outputWidth.value);
  crop.height = Math.min(nextMedia.height, Math.round(crop.width * 0.5625));
  crop.x = Math.max(0, Math.floor((nextMedia.width - crop.width) / 2));
  crop.y = Math.max(0, Math.floor((nextMedia.height - crop.height) / 2));
}

async function chooseVideo(): Promise<void> {
  if (isBusy.value) {
    return;
  }
  errorMessage.value = "";
  status.value = "probing";
  statusMessage.value = "正在选择视频…";

  try {
    const selected = await pickVideo();
    if (!selected) {
      status.value = media.value ? "ready" : "idle";
      statusMessage.value = media.value ? "保留当前视频。" : "选择一个视频开始制作。";
      return;
    }

    inputPath.value = selected;
    statusMessage.value = "正在读取视频信息…";
    const nextMedia = await probeMedia(selected);
    media.value = nextMedia;
    applyMediaDefaults(nextMedia);
    outputPath.value = defaultOutputPath(selected, exportFormat.value);
    status.value = "ready";
    statusMessage.value = "视频已就绪，可以调整参数并导出。";
  } catch (error) {
    status.value = "failed";
    errorMessage.value = readableError(error);
    statusMessage.value = "无法读取这个视频。";
  }
}

async function chooseOutput(): Promise<void> {
  if (!isDesktop.value) {
    errorMessage.value = "请在 Tauri 桌面模式中选择输出路径。";
    return;
  }

  try {
    const selected = await pickMediaOutput(outputPath.value || undefined, exportFormat.value);
    if (selected) {
      outputPath.value = ensureFormatExtension(selected, exportFormat.value);
      errorMessage.value = "";
    }
  } catch (error) {
    errorMessage.value = readableError(error);
  }
}

async function chooseImage(): Promise<void> {
  if (!isDesktop.value) {
    errorMessage.value = "请在 Tauri 桌面模式中选择 PNG 图案。";
    return;
  }

  try {
    const selected = await pickPng();
    if (selected) {
      imagePath.value = selected;
      useImage.value = true;
      errorMessage.value = "";
    }
  } catch (error) {
    errorMessage.value = readableError(error);
  }
}

function buildRenderPlan(): RenderPlan {
  const finalOut = ensureFormatExtension(outputPath.value.trim(), exportFormat.value);
  const activeLayers = textLayers.value
    .filter((l) => l.visible && l.text.trim().length > 0)
    .map((l) => ({
      id: l.id,
      text: l.text,
      x: Number(l.x),
      y: Number(l.y),
      size: Number(l.size),
      color: l.color,
      strokeWidth: Number(l.strokeWidth),
      strokeColor: l.strokeColor,
      useBox: Boolean(l.useBox),
      boxColor: l.boxColor,
      boxOpacity: Number(l.boxOpacity),
      visible: true,
    }));

  return {
    inputPath: inputPath.value,
    outputPath: finalOut,
    format: exportFormat.value,
    startSecs: Number(startSecs.value),
    endSecs: Number(endSecs.value),
    crop: useCrop.value ? { ...crop } : null,
    outputWidth: Number(outputWidth.value),
    fps: Number(fps.value),
    quality: currentQuality.value,
    textOverlay: useText.value && activeLayers.length > 0 ? activeLayers[0] : null,
    textLayers: useText.value && activeLayers.length > 0 ? activeLayers : null,
    imageOverlay:
      useImage.value && imagePath.value
        ? {
            path: imagePath.value,
            x: Number(imageX.value),
            y: Number(imageY.value),
            width: Number(imageWidth.value),
          }
        : null,
  };
}

async function startRender(): Promise<void> {
  if (!canRender.value) {
    errorMessage.value = "请先选择视频、输出路径，并确认参数完整。";
    return;
  }

  errorMessage.value = "";
  progress.value = 0;
  outputBytes.value = 0;
  status.value = "rendering";
  statusMessage.value = "正在启动 FFmpeg…";

  try {
    const plan = buildRenderPlan();
    outputPath.value = plan.outputPath;
    taskId.value = await renderGif(plan);
  } catch (error) {
    status.value = "failed";
    statusMessage.value = "FFmpeg 没有启动成功。";
    errorMessage.value = readableError(error);
  }
}

async function stopRender(): Promise<void> {
  if (!taskId.value) {
    return;
  }

  statusMessage.value = "正在取消…";
  try {
    await cancelRender(taskId.value);
  } catch (error) {
    errorMessage.value = readableError(error);
  }
}

function clearImage(): void {
  imagePath.value = "";
  useImage.value = false;
}

const HISTORY_STORAGE_KEY = "meme_studio_export_history";
const MAX_HISTORY = 10;
const historyList = ref<ExportRecord[]>([]);
const isDrawerOpen = ref(false);

const toastText = ref("");
const toastVisible = ref(false);
let toastTimer: ReturnType<typeof setTimeout> | null = null;

function showToast(text: string): void {
  toastText.value = text;
  toastVisible.value = true;
  if (toastTimer) {
    clearTimeout(toastTimer);
  }
  toastTimer = setTimeout(() => {
    toastVisible.value = false;
  }, 3200);
}

function formatBytes(bytes: number): string {
  if (!bytes || bytes <= 0) return "0 KB";
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

function formatRelativeTime(timestamp: number): string {
  const diffSecs = Math.floor((Date.now() - timestamp) / 1000);
  if (diffSecs < 60) return "刚刚";
  const diffMins = Math.floor(diffSecs / 60);
  if (diffMins < 60) return `${diffMins} 分钟前`;
  const diffHours = Math.floor(diffMins / 60);
  if (diffHours < 24) return `${diffHours} 小时前`;
  const diffDays = Math.floor(diffHours / 24);
  return `${diffDays} 天前`;
}

async function revealExportFile(path: string): Promise<void> {
  if (!isDesktop.value) {
    showToast("请在桌面模式中使用此功能");
    return;
  }
  try {
    await revealFileInDir(path);
    showToast("已在访达/资源管理器中定位文件");
  } catch {
    showToast("无法定位文件，可能已被移动或删除");
  }
}

async function copyExportFile(path: string): Promise<void> {
  try {
    const fileCopied = await copyFileToClipboard(path);
    if (fileCopied) {
      showToast("已复制 GIF 文件到剪贴板，可直接在微信/聊天软件中粘贴！");
    } else {
      showToast("已复制文件路径到剪贴板");
    }
  } catch {
    showToast("复制失败");
  }
}

async function loadHistoryPlan(record: ExportRecord): Promise<void> {
  const p = record.plan;
  if (!p) return;

  if (p.inputPath && (!media.value || media.value.path !== p.inputPath)) {
    if (isDesktop.value) {
      try {
        const probed = await probeMedia(p.inputPath);
        media.value = probed;
        inputPath.value = p.inputPath;
      } catch {
        showToast("源视频文件已被移动，已恢复其他参数");
      }
    } else {
      inputPath.value = p.inputPath;
    }
  }

  startSecs.value = p.startSecs;
  endSecs.value = p.endSecs;
  outputWidth.value = p.outputWidth;
  fps.value = p.fps;
  currentQuality.value = p.quality ?? "medium";
  onManualParameterChange();

  if (p.crop) {
    useCrop.value = true;
    crop.x = p.crop.x;
    crop.y = p.crop.y;
    crop.width = p.crop.width;
    crop.height = p.crop.height;
  } else {
    useCrop.value = false;
  }

  if (p.textLayers && p.textLayers.length > 0) {
    useText.value = true;
    textLayers.value = p.textLayers.map((l, idx) => ({
      id: l.id || `layer-${idx + 1}`,
      text: l.text,
      x: l.x,
      y: l.y,
      size: l.size,
      color: l.color,
      strokeWidth: l.strokeWidth ?? 2,
      strokeColor: l.strokeColor ?? "#000000",
      useBox: l.useBox ?? false,
      boxColor: l.boxColor ?? "#000000",
      boxOpacity: l.boxOpacity ?? 0.6,
      visible: l.visible ?? true,
    }));
    activeLayerId.value = textLayers.value[0].id;
  } else if (p.textOverlay) {
    useText.value = true;
    textLayers.value = [
      {
        id: "layer-1",
        text: p.textOverlay.text,
        x: p.textOverlay.x,
        y: p.textOverlay.y,
        size: p.textOverlay.size,
        color: p.textOverlay.color,
        strokeWidth: p.textOverlay.strokeWidth ?? 2,
        strokeColor: p.textOverlay.strokeColor ?? "#000000",
        useBox: p.textOverlay.useBox ?? false,
        boxColor: p.textOverlay.boxColor ?? "#000000",
        boxOpacity: p.textOverlay.boxOpacity ?? 0.6,
        visible: true,
      },
    ];
    activeLayerId.value = "layer-1";
  } else {
    useText.value = false;
  }

  if (p.imageOverlay) {
    useImage.value = true;
    imagePath.value = p.imageOverlay.path;
    imageX.value = p.imageOverlay.x;
    imageY.value = p.imageOverlay.y;
    imageWidth.value = p.imageOverlay.width;
  } else {
    useImage.value = false;
  }

  if (p.format === "webp" || record.filePath.toLowerCase().endsWith(".webp")) {
    exportFormat.value = "webp";
  } else {
    exportFormat.value = "gif";
  }

  outputPath.value = p.outputPath;
  showToast(`已恢复「${record.fileName}」的历史制作参数`);
}

function removeHistoryItem(id: string): void {
  historyList.value = historyList.value.filter((h) => h.id !== id);
  saveHistoryToStorage();
  showToast("已从历史记录移除");
}

function clearAllHistory(): void {
  historyList.value = [];
  saveHistoryToStorage();
  showToast("历史记录已清空");
}

async function addHistoryRecord(filePath: string, bytes: number): Promise<void> {
  const fileName = filePath.split(/[\\/]/).pop() ?? "meme.gif";
  const planSnapshot = buildRenderPlan();
  const id = `${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
  const record: ExportRecord = {
    id,
    fileName,
    filePath,
    outputBytes: bytes,
    timestamp: Date.now(),
    plan: planSnapshot,
  };

  historyList.value = [
    record,
    ...historyList.value.filter((h) => h.filePath !== filePath),
  ].slice(0, MAX_HISTORY);
  saveHistoryToStorage();

  try {
    const dataUrl = await readFileAsDataUrl(filePath);
    if (dataUrl) {
      const target = historyList.value.find((h) => h.id === id);
      if (target) {
        target.previewUrl = dataUrl;
      }
    }
  } catch {
    // ignore
  }
}

function saveHistoryToStorage(): void {
  try {
    const toSave = historyList.value.slice(0, MAX_HISTORY).map((item) => ({
      ...item,
      previewUrl: undefined,
    }));
    localStorage.setItem(HISTORY_STORAGE_KEY, JSON.stringify(toSave));
  } catch {
    // ignore
  }
}

function loadHistoryFromStorage(): void {
  try {
    const raw = localStorage.getItem(HISTORY_STORAGE_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw);
    if (Array.isArray(parsed)) {
      historyList.value = parsed.slice(0, MAX_HISTORY);
      for (const item of historyList.value) {
        if (!item.previewUrl && item.filePath) {
          readFileAsDataUrl(item.filePath)
            .then((dataUrl) => {
              if (dataUrl) {
                item.previewUrl = dataUrl;
              }
            })
            .catch(() => {});
        }
      }
    }
  } catch {
    // ignore
  }
}

function handleKeydown(e: KeyboardEvent): void {
  if (e.key === "Escape" && isDrawerOpen.value) {
    isDrawerOpen.value = false;
  }
}

async function connectDesktopEvents(): Promise<void> {
  if (!isDesktop.value) {
    return;
  }

  try {
    stopListening.value = await listenForRenderEvents({
      onProgress: (event) => {
        if (taskId.value && event.taskId !== taskId.value) {
          return;
        }
        taskId.value = event.taskId;
        status.value = "rendering";
        progress.value = event.progress;
        statusMessage.value = event.message;
      },
      onCompleted: async (event) => {
        if (taskId.value && event.taskId !== taskId.value) {
          return;
        }
        taskId.value = event.taskId;
        status.value = "completed";
        progress.value = 100;
        outputBytes.value = event.outputBytes;
        outputPath.value = event.outputPath;
        statusMessage.value = "GIF 已导出，可以打开文件查看。";
        await addHistoryRecord(event.outputPath, event.outputBytes);
      },
      onFailed: (event) => {
        if (taskId.value && event.taskId !== taskId.value) {
          return;
        }
        taskId.value = event.taskId;
        status.value = "failed";
        statusMessage.value = "导出没有完成。";
        errorMessage.value = readableRenderError(event.code, event.message);
      },
      onCancelled: (event) => {
        if (taskId.value && event.taskId !== taskId.value) {
          return;
        }
        taskId.value = event.taskId;
        status.value = "cancelled";
        progress.value = 0;
        statusMessage.value = event.message;
      },
    });
  } catch (error) {
    errorMessage.value = readableError(error);
  }
}

// 模板相关操作
function loadTemplates(): void {
  customTemplates.value = getCustomTemplates();
}

function openTemplatesDrawer(): void {
  drawerTab.value = "templates";
  isDrawerOpen.value = true;
}

function openSaveTemplateModal(): void {
  newTemplateName.value = "我的排版模板";
  isSaveTemplateModalOpen.value = true;
  nextTick(() => {
    templateNameInputRef.value?.focus();
  });
}

function confirmSaveTemplate(): void {
  const name = newTemplateName.value.trim();
  if (!name) return;

  let xRatio = 0.5;
  let yRatio = 0.85;
  if (media.value && media.value.width > 0 && media.value.height > 0) {
    xRatio = Number((textX.value / media.value.width).toFixed(3));
    yRatio = Number((textY.value / media.value.height).toFixed(3));
  }

  saveCustomTemplate({
    name,
    description: `自定义样式 · 字号 ${textSize.value} · ${textColor.value}`,
    badge: "我的模板",
    textStyle: {
      defaultText: text.value || "示例台词",
      size: Number(textSize.value),
      color: textColor.value,
      strokeWidth: Number(textStrokeWidth.value),
      strokeColor: textStrokeColor.value,
      useBox: Boolean(textUseBox.value),
      boxColor: textBoxColor.value,
      boxOpacity: Number(textBoxOpacity.value),
      positionRatio: { x: xRatio, y: yRatio },
    },
    suggestedPresetId: selectedPresetId.value !== "custom" ? selectedPresetId.value : undefined,
    suggestedFormat: exportFormat.value,
  });

  loadTemplates();
  isSaveTemplateModalOpen.value = false;
  showToast(`已成功保存模板「${name}」！可在模板库中查看`);
}

function handleDeleteTemplate(id: string): void {
  deleteCustomTemplate(id);
  loadTemplates();
  showToast("模板已删除");
}

function applyTemplate(tpl: MemeTemplate): void {
  useText.value = true;
  if (!text.value.trim() || text.value === "输入文字") {
    text.value = tpl.textStyle.defaultText || "输入文字";
  }
  textSize.value = tpl.textStyle.size;
  textColor.value = tpl.textStyle.color;
  textStrokeWidth.value = tpl.textStyle.strokeWidth;
  textStrokeColor.value = tpl.textStyle.strokeColor;
  textUseBox.value = tpl.textStyle.useBox;
  textBoxColor.value = tpl.textStyle.boxColor;
  textBoxOpacity.value = tpl.textStyle.boxOpacity;

  if (media.value) {
    textX.value = Math.max(0, Math.round(media.value.width * tpl.textStyle.positionRatio.x));
    textY.value = Math.max(0, Math.round(media.value.height * tpl.textStyle.positionRatio.y));
  }

  if (tpl.suggestedPresetId) {
    const p = PRESETS.find((item) => item.id === tpl.suggestedPresetId);
    if (p) applyPreset(p);
  }
  if (tpl.suggestedFormat) {
    setExportFormat(tpl.suggestedFormat);
  }

  isDrawerOpen.value = false;
  showToast(`已套用「${tpl.name}」模板到当前图层`);
}

// 草稿自动暂存与恢复
let draftAutoSaveTimer: ReturnType<typeof setTimeout> | null = null;
let isRestoringDraft = false;

function buildCurrentDraft(): StudioDraft {
  return {
    version: 1,
    updatedAt: Date.now(),
    inputPath: inputPath.value,
    outputPath: outputPath.value,
    startSecs: Number(startSecs.value),
    endSecs: Number(endSecs.value),
    useCrop: useCrop.value,
    crop: { ...crop },
    outputWidth: Number(outputWidth.value),
    fps: Number(fps.value),
    quality: currentQuality.value,
    format: exportFormat.value,
    selectedPresetId: selectedPresetId.value,
    useText: useText.value,
    text: text.value,
    textX: Number(textX.value),
    textY: Number(textY.value),
    textSize: Number(textSize.value),
    textColor: textColor.value,
    textStrokeWidth: Number(textStrokeWidth.value),
    textStrokeColor: textStrokeColor.value,
    textUseBox: Boolean(textUseBox.value),
    textBoxColor: textBoxColor.value,
    textBoxOpacity: Number(textBoxOpacity.value),
    textLayers: JSON.parse(JSON.stringify(textLayers.value)),
    activeLayerId: activeLayerId.value,
    useImage: useImage.value,
    imagePath: imagePath.value,
    imageX: Number(imageX.value),
    imageY: Number(imageY.value),
    imageWidth: Number(imageWidth.value),
  };
}

function queueDraftSave(): void {
  if (isRestoringDraft) return;
  if (draftAutoSaveTimer) {
    clearTimeout(draftAutoSaveTimer);
  }
  draftAutoSaveTimer = setTimeout(() => {
    if (!inputPath.value && !text.value && !useCrop.value) {
      return;
    }
    saveDraftToStorage(buildCurrentDraft());
  }, 400);
}

const hasActiveDraftOrMedia = computed(() => {
  return Boolean(media.value || inputPath.value || text.value || useCrop.value || useImage.value);
});

function resetWorkspace(): void {
  if (draftAutoSaveTimer) {
    clearTimeout(draftAutoSaveTimer);
  }
  clearDraftFromStorage();
  media.value = null;
  inputPath.value = "";
  outputPath.value = "";
  startSecs.value = 0;
  endSecs.value = 3;
  useCrop.value = false;
  crop.x = 0;
  crop.y = 0;
  crop.width = 480;
  crop.height = 270;
  outputWidth.value = 480;
  fps.value = 12;
  currentQuality.value = "medium";
  selectedPresetId.value = "custom";
  exportFormat.value = "gif";
  useText.value = false;
  textLayers.value = [
    {
      id: "layer-1",
      text: "输入文字",
      x: 20,
      y: 40,
      size: 28,
      color: "#ffffff",
      strokeWidth: 2,
      strokeColor: "#000000",
      useBox: false,
      boxColor: "#000000",
      boxOpacity: 0.6,
      visible: true,
    },
  ];
  activeLayerId.value = "layer-1";
  useImage.value = false;
  imagePath.value = "";
  status.value = "idle";
  statusMessage.value = "选择一个视频开始制作。";
  errorMessage.value = "";
  showToast("工作区已重置为初始状态");
}

async function restoreDraftIfAvailable(): Promise<void> {
  const draft = loadDraftFromStorage();
  if (!draft) return;

  isRestoringDraft = true;
  try {
    if (draft.inputPath) {
      try {
        const nextMedia = await probeMedia(draft.inputPath);
        media.value = nextMedia;
        inputPath.value = draft.inputPath;
        outputPath.value = draft.outputPath || defaultOutputPath(draft.inputPath, draft.format);
        startSecs.value = draft.startSecs;
        endSecs.value = draft.endSecs;
        useCrop.value = draft.useCrop;
        crop.x = draft.crop.x;
        crop.y = draft.crop.y;
        crop.width = draft.crop.width;
        crop.height = draft.crop.height;
        outputWidth.value = draft.outputWidth;
        fps.value = draft.fps;
        currentQuality.value = draft.quality;
        exportFormat.value = draft.format;
        selectedPresetId.value = draft.selectedPresetId;
        useText.value = draft.useText;
        if (draft.textLayers && draft.textLayers.length > 0) {
          textLayers.value = draft.textLayers;
          activeLayerId.value = draft.activeLayerId || draft.textLayers[0].id;
        } else if (draft.text) {
          textLayers.value = [
            {
              id: "layer-1",
              text: draft.text,
              x: draft.textX ?? 20,
              y: draft.textY ?? 40,
              size: draft.textSize ?? 28,
              color: draft.textColor ?? "#ffffff",
              strokeWidth: draft.textStrokeWidth ?? 2,
              strokeColor: draft.textStrokeColor ?? "#000000",
              useBox: draft.textUseBox ?? false,
              boxColor: draft.textBoxColor ?? "#000000",
              boxOpacity: draft.textBoxOpacity ?? 0.6,
              visible: true,
            },
          ];
          activeLayerId.value = "layer-1";
        }
        useImage.value = draft.useImage;
        imagePath.value = draft.imagePath;
        imageX.value = draft.imageX;
        imageY.value = draft.imageY;
        imageWidth.value = draft.imageWidth;
        status.value = "ready";
        statusMessage.value = "已自动还原上次编辑草稿。";
        showToast("已自动恢复上次编辑草稿");
      } catch {
        errorMessage.value = "上次编辑的视频文件已移动或无法访问，已恢复排版参数。";
        useText.value = draft.useText;
        if (draft.textLayers && draft.textLayers.length > 0) {
          textLayers.value = draft.textLayers;
          activeLayerId.value = draft.activeLayerId || draft.textLayers[0].id;
        } else if (draft.text) {
          textLayers.value = [
            {
              id: "layer-1",
              text: draft.text,
              x: draft.textX ?? 20,
              y: draft.textY ?? 40,
              size: draft.textSize ?? 28,
              color: draft.textColor ?? "#ffffff",
              strokeWidth: draft.textStrokeWidth ?? 2,
              strokeColor: draft.textStrokeColor ?? "#000000",
              useBox: draft.textUseBox ?? false,
              boxColor: draft.textBoxColor ?? "#000000",
              boxOpacity: draft.textBoxOpacity ?? 0.6,
              visible: true,
            },
          ];
          activeLayerId.value = "layer-1";
        }
      }
    }
  } finally {
    isRestoringDraft = false;
  }
}

watch(
  [
    inputPath,
    outputPath,
    startSecs,
    endSecs,
    useCrop,
    () => crop.x,
    () => crop.y,
    () => crop.width,
    () => crop.height,
    outputWidth,
    fps,
    currentQuality,
    exportFormat,
    selectedPresetId,
    useText,
    text,
    textX,
    textY,
    textSize,
    textColor,
    textStrokeWidth,
    textStrokeColor,
    textUseBox,
    textBoxColor,
    textBoxOpacity,
    useImage,
    imagePath,
    imageX,
    imageY,
    imageWidth,
  ],
  () => {
    queueDraftSave();
  },
  { deep: false },
);

watch(
  textLayers,
  () => {
    queueDraftSave();
  },
  { deep: true },
);

let resizeObserver: ResizeObserver | null = null;

onMounted(async () => {
  try {
    loadTemplates();
    loadHistoryFromStorage();
    window.addEventListener("keydown", handleKeydown);
    desktopInfo.value = await getDesktopInfo();
    await connectDesktopEvents();
    await restoreDraftIfAvailable();
    if (stageRef.value && typeof ResizeObserver !== "undefined") {
      resizeObserver = new ResizeObserver(() => {
        updateStageSize();
      });
      resizeObserver.observe(stageRef.value);
      updateStageSize();
    }
  } catch (error) {
    errorMessage.value = readableError(error);
  }
});

onBeforeUnmount(() => {
  if (draftAutoSaveTimer) {
    clearTimeout(draftAutoSaveTimer);
  }
  stopListening.value?.();
  resizeObserver?.disconnect();
  window.removeEventListener("keydown", handleKeydown);
  if (toastTimer) {
    clearTimeout(toastTimer);
  }
});
</script>

<template>
  <div class="studio-shell">
    <!-- 全局 Toast 提示 -->
    <Transition name="toast-fade">
      <div v-if="toastVisible" class="toast-notification" role="status">
        <span class="toast-icon">✓</span>
        <span class="toast-text">{{ toastText }}</span>
      </div>
    </Transition>

    <aside class="studio-sidebar">
      <div class="brand-lockup">
        <div class="brand-mark">M</div>
        <div>
          <strong>表情工坊</strong>
        </div>
      </div>

      <nav class="side-nav" aria-label="主导航">
        <a class="side-nav-item active" href="#workspace">
          <span class="nav-icon">✦</span>
          动图制作
        </a>
        <button
          type="button"
          class="side-nav-item side-nav-btn"
          title="打开排版模板库"
          @click="openTemplatesDrawer"
        >
          <span class="nav-icon">📑</span>
          模板库
          <span class="side-nav-badge">{{ allTemplatesCount }}</span>
        </button>
      </nav>

      <div class="sidebar-history-section">
        <div class="sidebar-history-header">
          <div class="sidebar-history-title">
            <span class="history-clock-icon">🕒</span>
            <strong>最近导出</strong>
            <span v-if="historyList.length > 0" class="history-count-badge">{{ historyList.length }}</span>
          </div>
          <button
            v-if="historyList.length > 0"
            type="button"
            class="history-expand-toggle"
            title="查看完整历史抽屉"
            @click="isDrawerOpen = true"
          >
            全部 ↗
          </button>
        </div>

        <div v-if="historyList.length > 0" class="sidebar-history-list">
          <div
            v-for="item in historyList.slice(0, 2)"
            :key="item.id"
            class="sidebar-history-card"
            :title="`${item.fileName}\n${item.filePath}`"
          >
            <div class="sidebar-history-thumb">
              <img v-if="item.previewUrl" :src="item.previewUrl" :alt="item.fileName" />
              <span v-else class="thumb-fallback">{{ item.fileName.toLowerCase().endsWith('.webp') ? 'WEBP' : 'GIF' }}</span>
            </div>
            <div class="sidebar-history-info">
              <div class="sidebar-history-name-row">
                <span class="sidebar-history-name">{{ item.fileName }}</span>
                <span class="history-format-badge" :class="item.fileName.toLowerCase().endsWith('.webp') ? 'badge-webp' : 'badge-gif'">
                  {{ item.fileName.toLowerCase().endsWith('.webp') ? 'WEBP' : 'GIF' }}
                </span>
              </div>
              <div class="sidebar-history-meta">
                <span class="sidebar-history-size">{{ formatBytes(item.outputBytes) }}</span>
                <span class="sidebar-history-time">{{ formatRelativeTime(item.timestamp) }}</span>
              </div>
            </div>
            <div class="sidebar-history-actions">
              <button
                type="button"
                class="sidebar-action-btn"
                title="在访达中定位"
                @click.stop="revealExportFile(item.filePath)"
              >
                🔍
              </button>
              <button
                type="button"
                class="sidebar-action-btn"
                title="复制文件"
                @click.stop="copyExportFile(item.filePath)"
              >
                📋
              </button>
              <button
                type="button"
                class="sidebar-action-btn"
                title="载入此历史参数"
                @click.stop="loadHistoryPlan(item)"
              >
                ↺
              </button>
            </div>
          </div>
        </div>
        <div v-else class="sidebar-history-empty">
          <span>暂无导出记录</span>
        </div>
      </div>

      <div class="sidebar-bottom">
        <div class="runtime-card">
          <span class="runtime-dot" :class="{ offline: !isDesktop }"></span>
          <div>
            <strong>{{ isDesktop ? "桌面端" : "预览" }}</strong>
            <small>{{ runtimeLabel }}</small>
          </div>
        </div>
      </div>
    </aside>

    <main id="workspace" class="studio-main">
      <header class="topbar">
        <div class="topbar-meta"><span class="status-mark"></span>{{ isDesktop ? "桌面端" : "预览" }}</div>
        <div class="topbar-actions">
          <button
            v-if="hasActiveDraftOrMedia"
            type="button"
            class="topbar-reset-btn"
            title="清空当前草稿并重置所有编辑参数"
            @click="resetWorkspace"
          >
            ↺ 重置工作区
          </button>
        </div>
      </header>

      <div class="page-content">
        <section class="page-intro">
          <div>
            <h1>制作 GIF</h1>
          </div>
          <div class="runtime-badge"><span></span>{{ isDesktop ? "桌面端就绪" : "需要 Tauri" }}</div>
        </section>

        <div v-if="!isDesktop" class="notice-banner">
          <span class="notice-icon">!</span>
          <div>需要桌面端 · 运行 <code>pnpm tauri:dev</code> 后才能导入本地文件。</div>
        </div>

        <section class="workspace-grid">
          <article class="preview-panel panel-card">
            <div class="panel-heading">
              <div>
                <p class="eyebrow">预览</p>
                <h2>源画面</h2>
              </div>
              <span class="panel-chip">GIF</span>
            </div>

            <div ref="stageRef" class="preview-stage" :style="previewStyle">
              <template v-if="media">
                <div class="preview-art">
                  <div class="preview-grid"></div>
                  <div class="preview-orb orb-a"></div>
                  <div class="preview-orb orb-b"></div>
                  <template v-if="useText">
                    <template v-for="layer in textLayers" :key="layer.id">
                      <div
                        v-if="layer.visible && layer.text"
                        class="preview-text interactive-drag"
                        :class="{
                          'is-dragging': isDraggingText && activeLayerId === layer.id,
                          'is-active': activeLayerId === layer.id,
                          'with-box': layer.useBox,
                        }"
                        :style="getTextLayerStyle(layer)"
                        :title="`图层 · 点击选中并拖拽`"
                        @pointerdown="startDragText(layer, $event)"
                      >
                        <span v-if="isDraggingText && activeLayerId === layer.id" class="drag-handle-pill">
                          X: {{ layer.x }} Y: {{ layer.y }}
                        </span>
                        {{ layer.text }}
                      </div>
                    </template>
                  </template>
                  <div
                    v-if="useImage && imagePath"
                    class="preview-sticker interactive-drag"
                    :class="{ 'is-dragging': isDraggingImage }"
                    :style="{
                      left: `${imageDisplayLeft}px`,
                      top: `${imageDisplayTop}px`,
                      width: `${imageDisplayWidth}px`,
                    }"
                    title="按住鼠标拖拽贴图位置"
                    @pointerdown="startDragImage"
                  >
                    <span v-if="isDraggingImage" class="drag-handle-pill">X: {{ imageX }} Y: {{ imageY }}</span>
                    PNG
                  </div>

                  <!-- 可视化矩形裁切框与九宫格遮罩层 -->
                  <div
                    v-if="useCrop"
                    class="crop-box"
                    :class="{ 'is-dragging': isDraggingCrop }"
                    :style="{
                      left: `${cropDisplayLeft}px`,
                      top: `${cropDisplayTop}px`,
                      width: `${cropDisplayWidth}px`,
                      height: `${cropDisplayHeight}px`,
                    }"
                    title="按住鼠标拖拽移动选区位置"
                    @pointerdown="startDragCrop"
                  >
                    <!-- 九宫格参考线 -->
                    <div class="crop-grid-lines" aria-hidden="true">
                      <span class="crop-line line-h-1"></span>
                      <span class="crop-line line-h-2"></span>
                      <span class="crop-line line-v-1"></span>
                      <span class="crop-line line-v-2"></span>
                    </div>

                    <!-- 8 个方向调节手柄 -->
                    <div class="crop-handle handle-nw" title="调整左上角" @pointerdown.stop="startResizeCrop('nw', $event)"></div>
                    <div class="crop-handle handle-n" title="调整顶部" @pointerdown.stop="startResizeCrop('n', $event)"></div>
                    <div class="crop-handle handle-ne" title="调整右上角" @pointerdown.stop="startResizeCrop('ne', $event)"></div>
                    <div class="crop-handle handle-e" title="调整右侧" @pointerdown.stop="startResizeCrop('e', $event)"></div>
                    <div class="crop-handle handle-se" title="调整右下角" @pointerdown.stop="startResizeCrop('se', $event)"></div>
                    <div class="crop-handle handle-s" title="调整底部" @pointerdown.stop="startResizeCrop('s', $event)"></div>
                    <div class="crop-handle handle-sw" title="调整左下角" @pointerdown.stop="startResizeCrop('sw', $event)"></div>
                    <div class="crop-handle handle-w" title="调整左侧" @pointerdown.stop="startResizeCrop('w', $event)"></div>

                    <!-- 尺寸指示标签 -->
                    <div class="crop-dimension-pill" :class="{ 'pill-inside': isCropNearTop }">
                      <span>{{ crop.width }} × {{ crop.height }}</span>
                      <small>X: {{ crop.x }} Y: {{ crop.y }}</small>
                    </div>
                  </div>

                  <div class="preview-caption">
                    {{ useCrop ? `裁切中 · ${crop.width} × ${crop.height} · 可拖拽平移或缩放选区` : `源画面 · ${media.width} × ${media.height} · 支持画布拖拽排版` }}
                  </div>
                </div>
                <div class="play-button" aria-hidden="true">▶</div>
              </template>
              <div v-else class="preview-empty">
                <div class="empty-mark">✦</div>
                <strong>暂无视频</strong>
                <span>导入视频后即可预览。</span>
              </div>
            </div>

            <div v-if="media" class="timeline-panel">
              <div class="timeline-header">
                <div class="timeline-title">
                  <span class="timeline-dot"></span>
                  <strong>时间裁切</strong>
                  <span class="timeline-badge">{{ selectedDurationLabel }}</span>
                </div>
                <div class="timeline-time-display">
                  <span>{{ startSecs.toFixed(2) }}s</span>
                  <span class="timeline-sep">至</span>
                  <span>{{ endSecs.toFixed(2) }}s</span>
                  <span class="timeline-total">/ {{ mediaDurationLabel }}</span>
                </div>
              </div>
              <div
                ref="timelineTrackRef"
                class="timeline-track"
                title="点击或拖动滑块调整时间范围"
                @click="onTimelineTrackClick"
              >
                <div class="timeline-ticks"></div>
                <div
                  class="timeline-selection"
                  :style="{ left: `${startPercent}%`, width: `${rangePercent}%` }"
                  title="拖拽可整体平移选区"
                  @pointerdown.stop="startDragRange"
                >
                  <div class="timeline-selection-glow"></div>
                </div>
                <div
                  class="timeline-handle handle-start"
                  :style="{ left: `${startPercent}%` }"
                  role="slider"
                  aria-label="开始时间"
                  title="拖拽调整开始时间"
                  @pointerdown.stop="startDragStartHandle"
                >
                  <span class="handle-bar"></span>
                </div>
                <div
                  class="timeline-handle handle-end"
                  :style="{ left: `${endPercent}%` }"
                  role="slider"
                  aria-label="结束时间"
                  title="拖拽调整结束时间"
                  @pointerdown.stop="startDragEndHandle"
                >
                  <span class="handle-bar"></span>
                </div>
              </div>
            </div>

            <div class="media-summary">
              <div class="media-file">
                <span class="file-icon">▧</span>
                <div>
                    <strong>{{ media?.fileName ?? "暂无来源" }}</strong>
                    <small>{{ media ? `${media.format} · ${media.width} × ${media.height}` : "本地视频" }}</small>
                </div>
              </div>
              <div class="media-stats">
                <span><b>{{ mediaDurationLabel }}</b> 时长</span>
                <span><b>{{ mediaFpsLabel }}</b></span>
              </div>
            </div>
          </article>

          <aside class="controls-panel panel-card">
            <div class="panel-heading compact-heading">
              <div>
                <p class="eyebrow">编辑</p>
                <h2>参数</h2>
              </div>
              <span class="step-number">02</span>
            </div>

            <button class="file-picker" type="button" :disabled="isBusy" @click="chooseVideo">
              <span class="picker-icon">↑</span>
              <span>
                <strong>{{ media ? "替换视频" : "导入视频" }}</strong>
                <small>{{ media ? "选择其他视频" : "MP4 · MOV · WEBM" }}</small>
              </span>
              <span class="picker-arrow">→</span>
            </button>

            <div class="control-section">
              <div class="section-label"><span>01</span> 时间</div>
              <div class="field-grid two-up">
                <label class="field"><span>开始 · 秒</span><input v-model.number="startSecs" type="number" min="0" step="0.1" /></label>
                <label class="field"><span>结束 · 秒</span><input v-model.number="endSecs" type="number" min="0.1" step="0.1" :max="mediaDuration || undefined" /></label>
              </div>
            </div>

            <div class="control-section">
              <div class="section-label-row">
                <div class="section-label"><span>02</span> 输出预设</div>
                <span class="preset-tag-info">{{ currentPresetInfo.name }} · {{ currentPresetInfo.badge }}</span>
              </div>
              <div class="preset-pills">
                <button
                  v-for="p in PRESETS"
                  :key="p.id"
                  type="button"
                  class="preset-pill"
                  :class="{ active: selectedPresetId === p.id }"
                  :title="p.description"
                  @click="applyPreset(p)"
                >
                  <span class="preset-name">{{ p.name }}</span>
                  <span class="preset-badge">{{ p.badge }}</span>
                </button>
                <button
                  type="button"
                  class="preset-pill"
                  :class="{ active: selectedPresetId === 'custom' }"
                  title="手动调整输出参数"
                  @click="selectedPresetId = 'custom'"
                >
                  <span class="preset-name">自定义</span>
                  <span class="preset-badge">自定</span>
                </button>
              </div>
              <div class="field-grid two-up">
                <label class="field"><span>宽度 · 像素</span><input v-model.number="outputWidth" type="number" min="32" max="1920" step="1" @input="onManualParameterChange" /></label>
                <label class="field"><span>帧率 · FPS</span><input v-model.number="fps" type="number" min="1" max="60" step="1" @input="onManualParameterChange" /></label>
              </div>
              <div class="crop-header-row">
                <label class="toggle-row"><input v-model="useCrop" type="checkbox" /><span class="toggle-ui"></span><span>裁切</span><em v-if="useCrop">{{ crop.width }} × {{ crop.height }}</em></label>
                <div v-if="useCrop" class="crop-quick-actions">
                  <button type="button" class="crop-mini-btn" title="裁切选框居中" @click="cropCenter">居中</button>
                  <button type="button" class="crop-mini-btn" title="1:1 正方形比例" @click="cropSquare">1:1</button>
                  <button type="button" class="crop-mini-btn" title="重置铺满全画面" @click="cropReset">铺满</button>
                </div>
              </div>
              <div v-if="useCrop" class="field-grid four-up crop-grid">
                <label class="field"><span>X</span><input v-model.number="crop.x" type="number" min="0" step="1" /></label>
                <label class="field"><span>Y</span><input v-model.number="crop.y" type="number" min="0" step="1" /></label>
                <label class="field"><span>W</span><input v-model.number="crop.width" type="number" min="20" step="1" /></label>
                <label class="field"><span>H</span><input v-model.number="crop.height" type="number" min="20" step="1" /></label>
              </div>
            </div>

            <div class="control-section">
              <div class="section-label-row">
                <div class="section-label"><span>03</span> 叠加</div>
                <div v-if="useText" class="template-quick-actions">
                  <button type="button" class="template-link-btn" title="浏览表情包排版模板库" @click="openTemplatesDrawer">
                    📑 套用模板
                  </button>
                  <button type="button" class="template-link-btn" title="将当前文字排版存为我的模板" @click="openSaveTemplateModal">
                    ＋ 存为模板
                  </button>
                </div>
              </div>
              <label class="toggle-row"><input v-model="useText" type="checkbox" /><span class="toggle-ui"></span><span>文字</span></label>
              <div v-if="useText" class="overlay-fields">
                <div class="layer-tabs-bar">
                  <div class="layer-tabs-list">
                    <button
                      v-for="(layer, index) in textLayers"
                      :key="layer.id"
                      type="button"
                      class="layer-tab-btn"
                      :class="{ 'is-active': activeLayerId === layer.id, 'is-hidden': !layer.visible }"
                      @click="activeLayerId = layer.id"
                    >
                      <span class="tab-tag">T{{ index + 1 }}</span>
                      <span class="tab-snippet">{{ layer.text || '未命名' }}</span>
                      <span
                        class="layer-action-icon"
                        role="button"
                        :title="layer.visible ? '隐藏此图层' : '显示此图层'"
                        @click.stop="toggleLayerVisibility(layer)"
                      >
                        {{ layer.visible ? '👁' : '🚫' }}
                      </span>
                      <span
                        v-if="textLayers.length > 1"
                        class="layer-action-icon del-icon"
                        role="button"
                        title="删除图层"
                        @click.stop="removeTextLayer(layer.id)"
                      >
                        ×
                      </span>
                    </button>
                  </div>
                  <button
                    v-if="textLayers.length < 4"
                    type="button"
                    class="add-layer-btn"
                    title="添加文字图层（最多 4 个）"
                    @click="addTextLayer"
                  >
                    ＋ 添加
                  </button>
                </div>
                <label class="field field-wide"><span>内容</span><input v-model="text" type="text" maxlength="80" placeholder="输入文字" /></label>
                <div class="field-grid four-up">
                  <label class="field"><span>X</span><input v-model.number="textX" type="number" min="0" step="1" /></label>
                  <label class="field"><span>Y</span><input v-model.number="textY" type="number" min="0" step="1" /></label>
                  <label class="field"><span>字号</span><input v-model.number="textSize" type="number" min="8" max="192" step="1" /></label>
                  <label class="field color-field"><span>颜色</span><input v-model="textColor" type="color" /></label>
                </div>
                <div class="field-grid four-up text-style-grid">
                  <label class="field"><span>描边宽</span><input v-model.number="textStrokeWidth" type="number" min="0" max="12" step="1" /></label>
                  <label class="field color-field"><span>描边色</span><input v-model="textStrokeColor" type="color" /></label>
                  <label class="field toggle-inline-field">
                    <span>底框</span>
                    <label class="mini-toggle-row">
                      <input v-model="textUseBox" type="checkbox" />
                      <span class="toggle-ui mini-ui"></span>
                      <small>{{ textUseBox ? "开启" : "关闭" }}</small>
                    </label>
                  </label>
                  <label v-if="textUseBox" class="field color-field"><span>底色</span><input v-model="textBoxColor" type="color" /></label>
                </div>
                <div v-if="textUseBox" class="box-opacity-row">
                  <span class="opacity-label">底框不透明度</span>
                  <input v-model.number="textBoxOpacity" type="range" min="0.1" max="1.0" step="0.05" class="opacity-slider" />
                  <span class="opacity-value">{{ Math.round(textBoxOpacity * 100) }}%</span>
                </div>
              </div>
              <div class="toggle-row image-row">
                <label class="toggle-label"><input v-model="useImage" type="checkbox" /><span class="toggle-ui"></span><span>PNG 图案</span></label>
                <button class="inline-button" type="button" @click="chooseImage">{{ imagePath ? "替换" : "选择 PNG" }}</button>
              </div>
              <div v-if="useImage && imagePath" class="overlay-fields">
                <div class="selected-file"><span>PNG</span><strong>{{ imagePath.split(/[\\/]/).pop() }}</strong><button type="button" aria-label="移除 PNG" @click="clearImage">×</button></div>
                <div class="field-grid three-up">
                  <label class="field"><span>X</span><input v-model.number="imageX" type="number" min="0" step="1" /></label>
                  <label class="field"><span>Y</span><input v-model.number="imageY" type="number" min="0" step="1" /></label>
                  <label class="field"><span>Width</span><input v-model.number="imageWidth" type="number" min="1" max="1920" step="1" /></label>
                </div>
              </div>
            </div>

            <div class="control-section output-section">
              <div class="section-label-row">
                <div class="section-label"><span>04</span> 格式与保存</div>
                <span class="format-hint">{{ exportFormat === 'webp' ? '小体积·真彩色' : '通用兼容' }}</span>
              </div>
              <div class="format-pills">
                <button
                  type="button"
                  class="format-pill"
                  :class="{ active: exportFormat === 'gif' }"
                  title="经典 GIF 动图格式，兼容所有聊天软件与平台"
                  @click="setExportFormat('gif')"
                >
                  <span class="format-pill-name">GIF</span>
                  <span class="format-pill-sub">经典通用</span>
                </button>
                <button
                  type="button"
                  class="format-pill"
                  :class="{ active: exportFormat === 'webp' }"
                  title="现代 WebP 动图格式，体积缩小 40%~60%，极速导出"
                  @click="setExportFormat('webp')"
                >
                  <span class="format-pill-name">WebP</span>
                  <span class="format-pill-badge">推荐·小体积</span>
                </button>
              </div>
              <div class="output-path-row">
                <input v-model="outputPath" type="text" :placeholder="exportFormat === 'webp' ? 'WebP 输出路径' : 'GIF 输出路径'" />
                <button type="button" aria-label="选择输出路径" @click="chooseOutput">…</button>
              </div>
            </div>
          </aside>
        </section>

        <section class="export-bar panel-card">
          <div class="export-status">
            <div class="status-icon" :class="`status-${status}`"><span>{{ status === "completed" ? "✓" : status === "failed" ? "!" : "✦" }}</span></div>
            <div>
              <strong>{{ statusLabel }}</strong>
              <span>{{ errorMessage || statusMessage }}</span>
            </div>
          </div>
          <div class="progress-wrap" :class="{ hidden: !isBusy && status !== 'completed' }">
            <div class="progress-track"><span :style="{ width: `${progress}%` }"></span></div>
            <small>{{ Math.round(progress) }}%</small>
          </div>
          <div class="export-actions">
            <span v-if="status === 'completed'" class="output-size">{{ formatBytes(outputBytes) }}</span>
            <span v-else-if="media" class="preset-summary-badge">{{ currentPresetInfo.name }} · {{ currentPresetInfo.badge }}</span>
            <template v-if="status === 'completed' && outputPath">
              <button
                class="secondary-action"
                type="button"
                title="在访达/资源管理器中显示此 GIF"
                @click="revealExportFile(outputPath)"
              >
                访达定位 ↗
              </button>
              <button
                class="secondary-action"
                type="button"
                title="复制文件到剪贴板，支持直接在微信等软件粘贴"
                @click="copyExportFile(outputPath)"
              >
                复制文件 📋
              </button>
            </template>
            <button
              v-if="historyList.length > 0"
              class="secondary-action"
              type="button"
              title="查看全部导出历史"
              @click="isDrawerOpen = !isDrawerOpen"
            >
              历史 ({{ historyList.length }})
            </button>
            <button v-if="isBusy" class="secondary-action danger" type="button" @click="stopRender">取消</button>
            <button v-else class="primary-action" type="button" :disabled="!canRender || !isDesktop" @click="startRender">
              {{ status === "completed" ? "再次导出" : (exportFormat === "webp" ? "导出 WebP" : "导出 GIF") }} <span>↗</span>
            </button>
          </div>
        </section>

      </div>

      <!-- 底部滑出式抽屉（导出历史与排版模板） -->
      <Transition name="drawer-slide">
        <div v-if="isDrawerOpen" class="history-drawer-backdrop" @click="isDrawerOpen = false">
          <div class="history-drawer-panel" @click.stop>
            <div class="drawer-header">
              <div class="drawer-header-left">
                <div class="drawer-tabs">
                  <button
                    type="button"
                    class="drawer-tab"
                    :class="{ active: drawerTab === 'history' }"
                    @click="drawerTab = 'history'"
                  >
                    <span>🕒 导出历史</span>
                    <span class="drawer-count-chip">{{ historyList.length }}</span>
                  </button>
                  <button
                    type="button"
                    class="drawer-tab"
                    :class="{ active: drawerTab === 'templates' }"
                    @click="drawerTab = 'templates'"
                  >
                    <span>📑 表情包模板</span>
                    <span class="drawer-count-chip">{{ allTemplatesList.length }}</span>
                  </button>
                </div>
              </div>
              <div class="drawer-header-right">
                <template v-if="drawerTab === 'history'">
                  <button
                    v-if="historyList.length > 0"
                    type="button"
                    class="drawer-clear-btn"
                    @click="clearAllHistory"
                  >
                    清空历史
                  </button>
                </template>
                <template v-else>
                  <button
                    type="button"
                    class="drawer-add-template-btn"
                    title="将当前工作区文字排版保存为新模板"
                    @click="openSaveTemplateModal"
                  >
                    ＋ 存为我的模板
                  </button>
                </template>
                <button
                  type="button"
                  class="drawer-close-btn"
                  aria-label="关闭抽屉"
                  @click="isDrawerOpen = false"
                >
                  ✕
                </button>
              </div>
            </div>

            <!-- Tab 1: 历史记录卡片横轴 -->
            <div v-if="drawerTab === 'history'" class="drawer-tab-content">
              <div v-if="historyList.length > 0" class="drawer-cards-strip">
                <div
                  v-for="item in historyList"
                  :key="item.id"
                  class="history-card"
                >
                  <div class="history-card-thumb">
                    <img v-if="item.previewUrl" :src="item.previewUrl" :alt="item.fileName" />
                    <div v-else class="card-thumb-fallback">
                      <span>{{ item.fileName.toLowerCase().endsWith('.webp') ? 'WEBP' : 'GIF' }}</span>
                    </div>
                    <span class="history-card-size-pill">{{ formatBytes(item.outputBytes) }}</span>
                  </div>

                  <div class="history-card-body">
                    <div class="history-card-title-row">
                      <strong class="history-card-filename" :title="item.fileName">{{ item.fileName }}</strong>
                      <span class="history-format-badge" :class="item.fileName.toLowerCase().endsWith('.webp') ? 'badge-webp' : 'badge-gif'">
                        {{ item.fileName.toLowerCase().endsWith('.webp') ? 'WEBP' : 'GIF' }}
                      </span>
                      <small class="history-card-time">{{ formatRelativeTime(item.timestamp) }}</small>
                    </div>
                    <div class="history-card-desc">
                      <span>{{ item.plan.outputWidth }}px · {{ item.plan.fps }} FPS</span>
                      <span v-if="item.plan.quality">· {{ item.plan.quality }}</span>
                    </div>
                    <div class="history-card-actions">
                      <button
                        type="button"
                        class="history-btn primary-subtle"
                        title="在访达/资源管理器中定位"
                        @click="revealExportFile(item.filePath)"
                      >
                        访达定位
                      </button>
                      <button
                        type="button"
                        class="history-btn"
                        title="复制文件到剪贴板"
                        @click="copyExportFile(item.filePath)"
                      >
                        复制
                      </button>
                      <button
                        type="button"
                        class="history-btn"
                        title="载入此条历史制作参数"
                        @click="loadHistoryPlan(item)"
                      >
                        载入参数
                      </button>
                      <button
                        type="button"
                        class="history-btn delete-btn"
                        title="从历史记录删除"
                        @click="removeHistoryItem(item.id)"
                      >
                        ×
                      </button>
                    </div>
                  </div>
                </div>
              </div>
              <div v-else class="drawer-empty-state">
                <div class="empty-icon">✦</div>
                <strong>暂无导出历史</strong>
                <span>在此完成的动图将保存在这里，支持一键定位、复制与参数回溯。</span>
              </div>
            </div>

            <!-- Tab 2: 表情包排版模板列表 -->
            <div v-else class="drawer-tab-content">
              <div class="drawer-cards-strip templates-strip">
                <div
                  v-for="tpl in allTemplatesList"
                  :key="tpl.id"
                  class="template-card"
                  :class="{ 'is-custom': tpl.category === 'custom' }"
                >
                  <div class="template-card-preview">
                    <div
                      class="template-preview-badge-pill"
                      :style="{
                        color: tpl.textStyle.color,
                        background: tpl.textStyle.useBox
                          ? `${tpl.textStyle.boxColor}aa`
                          : 'rgba(0,0,0,0.6)',
                        border: tpl.textStyle.strokeWidth > 0
                          ? `1px solid ${tpl.textStyle.strokeColor}`
                          : '1px solid rgba(255,255,255,0.15)',
                      }"
                    >
                      <span class="preview-mini-quote">“</span>
                      <span>{{ tpl.textStyle.defaultText || '示例排版' }}</span>
                      <span class="preview-mini-quote">”</span>
                    </div>
                  </div>

                  <div class="template-card-body">
                    <div class="template-card-title-row">
                      <strong class="template-card-name">{{ tpl.name }}</strong>
                      <span
                        class="template-badge"
                        :class="tpl.category === 'custom' ? 'badge-custom' : 'badge-builtin'"
                      >
                        {{ tpl.badge }}
                      </span>
                    </div>
                    <p class="template-card-desc">{{ tpl.description }}</p>
                    <div class="template-card-actions">
                      <button
                        type="button"
                        class="template-apply-btn"
                        title="将此排版样式一键套用至当前画面"
                        @click="applyTemplate(tpl)"
                      >
                        套用模板 ✦
                      </button>
                      <button
                        v-if="tpl.category === 'custom'"
                        type="button"
                        class="template-delete-btn"
                        title="删除该自定义模板"
                        @click.stop="handleDeleteTemplate(tpl.id)"
                      >
                        🗑
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>

          </div>
        </div>
      </Transition>
    </main>

    <!-- 存为模板 Modal 对话框 -->
    <Transition name="modal-fade">
      <div v-if="isSaveTemplateModalOpen" class="modal-backdrop" @click="isSaveTemplateModalOpen = false">
        <div class="modal-box" @click.stop>
          <div class="modal-header">
            <strong>存为我的排版模板</strong>
            <button type="button" class="modal-close-btn" @click="isSaveTemplateModalOpen = false">✕</button>
          </div>
          <div class="modal-body">
            <p class="modal-hint">将当前文字的字号、前景色、描边、底框及相对位置保存为模板，可在后续任意视频中一键套用。</p>
            <div class="modal-preview-row">
              <span class="modal-preview-label">当前排版效果：</span>
              <div
                class="modal-preview-pill"
                :style="{
                  color: textColor,
                  background: textUseBox ? `${textBoxColor}aa` : 'rgba(0,0,0,0.6)',
                  border: textStrokeWidth > 0 ? `1px solid ${textStrokeColor}` : '1px solid rgba(255,255,255,0.15)',
                }"
              >
                {{ text || '示例台词' }} ({{ textSize }}px)
              </div>
            </div>
            <label class="modal-field">
              <span>模板名称</span>
              <input
                ref="templateNameInputRef"
                v-model="newTemplateName"
                type="text"
                maxlength="20"
                placeholder="例如：吐槽大白话、影视经典字幕"
                @keydown.enter="confirmSaveTemplate"
              />
            </label>
          </div>
          <div class="modal-footer">
            <button type="button" class="secondary-action" @click="isSaveTemplateModalOpen = false">取消</button>
            <button type="button" class="primary-action" :disabled="!newTemplateName.trim()" @click="confirmSaveTemplate">
              保存模板
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>
