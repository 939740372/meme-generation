<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from "vue";
import {
  cancelRender,
  copyFileToClipboard,
  getDesktopInfo,
  isDesktopRuntime,
  listenForRenderEvents,
  pickGifOutput,
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
const status = ref<TaskStatus>("idle");
const progress = ref(0);
const statusMessage = ref("选择一个视频开始制作。");
const errorMessage = ref("");
const taskId = ref<string | null>(null);
const outputBytes = ref(0);
const stopListening = ref<(() => void) | null>(null);

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
const text = ref("表情工坊");
const textX = ref(24);
const textY = ref(24);
const textSize = ref(42);
const textColor = ref("#ffffff");

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

const textDisplayLeft = computed(() => {
  if (!media.value || stageSize.width <= 0) return textX.value;
  return textX.value * (stageSize.width / media.value.width);
});

const textDisplayTop = computed(() => {
  if (!media.value || stageSize.height <= 0) return textY.value;
  return textY.value * (stageSize.height / media.value.height);
});

const textDisplayFontSize = computed(() => {
  if (!media.value || stageSize.width <= 0) return textSize.value;
  return Math.max(12, Math.min(textSize.value * (stageSize.width / media.value.width), 72));
});

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

function startDragText(event: PointerEvent): void {
  if (!media.value || !stageRef.value) return;
  event.preventDefault();
  event.stopPropagation();
  isDraggingText.value = true;
  const startClientX = event.clientX;
  const startClientY = event.clientY;
  const initX = textX.value;
  const initY = textY.value;
  const scaleX = stageRef.value.clientWidth / media.value.width;
  const scaleY = stageRef.value.clientHeight / media.value.height;

  const onPointerMove = (e: PointerEvent): void => {
    const deltaX = (e.clientX - startClientX) / scaleX;
    const deltaY = (e.clientY - startClientY) / scaleY;
    const newX = Math.round(initX + deltaX);
    const newY = Math.round(initY + deltaY);
    textX.value = Math.max(0, Math.min(media.value!.width - 20, newX));
    textY.value = Math.max(0, Math.min(media.value!.height - 20, newY));
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

function defaultGifPath(path: string): string {
  const separator = path.includes("\\") ? "\\" : "/";
  const parts = path.split(/[\\/]/);
  parts.pop();
  const parent = parts.join(separator);
  const fileName = `${basenameWithoutExtension(path)}-meme.gif`;
  return parent ? `${parent}${separator}${fileName}` : fileName;
}

function ensureGifExtension(path: string): string {
  return path.toLowerCase().endsWith(".gif") ? path : `${path}.gif`;
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
    outputPath.value = defaultGifPath(selected);
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
    const selected = await pickGifOutput(outputPath.value || undefined);
    if (selected) {
      outputPath.value = ensureGifExtension(selected);
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
  return {
    inputPath: inputPath.value,
    outputPath: ensureGifExtension(outputPath.value.trim()),
    startSecs: Number(startSecs.value),
    endSecs: Number(endSecs.value),
    crop: useCrop.value ? { ...crop } : null,
    outputWidth: Number(outputWidth.value),
    fps: Number(fps.value),
    quality: currentQuality.value,
    textOverlay: useText.value
      ? {
          text: text.value,
          x: Number(textX.value),
          y: Number(textY.value),
          size: Number(textSize.value),
          color: textColor.value,
        }
      : null,
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

  if (p.textOverlay) {
    useText.value = true;
    text.value = p.textOverlay.text;
    textX.value = p.textOverlay.x;
    textY.value = p.textOverlay.y;
    textSize.value = p.textOverlay.size;
    textColor.value = p.textOverlay.color;
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

let resizeObserver: ResizeObserver | null = null;

onMounted(async () => {
  try {
    loadHistoryFromStorage();
    window.addEventListener("keydown", handleKeydown);
    desktopInfo.value = await getDesktopInfo();
    await connectDesktopEvents();
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
          GIF 制作
        </a>
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
              <span v-else class="thumb-fallback">GIF</span>
            </div>
            <div class="sidebar-history-info">
              <span class="sidebar-history-name">{{ item.fileName }}</span>
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
                  <div
                    v-if="useText && text"
                    class="preview-text interactive-drag"
                    :class="{ 'is-dragging': isDraggingText }"
                    :style="{
                      left: `${textDisplayLeft}px`,
                      top: `${textDisplayTop}px`,
                      color: textColor,
                      fontSize: `${textDisplayFontSize}px`,
                    }"
                    title="按住鼠标拖拽文字位置"
                    @pointerdown="startDragText"
                  >
                    <span v-if="isDraggingText" class="drag-handle-pill">X: {{ textX }} Y: {{ textY }}</span>
                    {{ text }}
                  </div>
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
                  <div class="preview-caption">源画面 · {{ media.width }} × {{ media.height }} · 支持画布拖拽排版</div>
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
              <label class="toggle-row"><input v-model="useCrop" type="checkbox" /><span class="toggle-ui"></span><span>裁切</span></label>
              <div v-if="useCrop" class="field-grid four-up crop-grid">
                <label class="field"><span>X</span><input v-model.number="crop.x" type="number" min="0" step="1" /></label>
                <label class="field"><span>Y</span><input v-model.number="crop.y" type="number" min="0" step="1" /></label>
                <label class="field"><span>W</span><input v-model.number="crop.width" type="number" min="1" step="1" /></label>
                <label class="field"><span>H</span><input v-model.number="crop.height" type="number" min="1" step="1" /></label>
              </div>
            </div>

            <div class="control-section">
              <div class="section-label"><span>03</span> 叠加</div>
              <label class="toggle-row"><input v-model="useText" type="checkbox" /><span class="toggle-ui"></span><span>文字</span></label>
              <div v-if="useText" class="overlay-fields">
                <label class="field field-wide"><span>内容</span><input v-model="text" type="text" maxlength="80" placeholder="输入文字" /></label>
                <div class="field-grid four-up">
                  <label class="field"><span>X</span><input v-model.number="textX" type="number" min="0" step="1" /></label>
                  <label class="field"><span>Y</span><input v-model.number="textY" type="number" min="0" step="1" /></label>
                  <label class="field"><span>字号</span><input v-model.number="textSize" type="number" min="8" max="192" step="1" /></label>
                  <label class="field color-field"><span>颜色</span><input v-model="textColor" type="color" /></label>
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
              <div class="section-label"><span>04</span> 保存到</div>
              <div class="output-path-row">
                <input v-model="outputPath" type="text" placeholder="GIF 输出路径" />
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
              {{ status === "completed" ? "再次导出" : "导出 GIF" }} <span>↗</span>
            </button>
          </div>
        </section>

      </div>

      <!-- 底部滑出式历史抽屉 -->
      <Transition name="drawer-slide">
        <div v-if="isDrawerOpen" class="history-drawer-backdrop" @click="isDrawerOpen = false">
          <div class="history-drawer-panel" @click.stop>
            <div class="drawer-header">
              <div class="drawer-header-left">
                <span class="drawer-dot"></span>
                <strong>导出历史</strong>
                <span class="drawer-count-chip">共 {{ historyList.length }} 个动图</span>
              </div>
              <div class="drawer-header-right">
                <button
                  v-if="historyList.length > 0"
                  type="button"
                  class="drawer-clear-btn"
                  @click="clearAllHistory"
                >
                  清空历史
                </button>
                <button
                  type="button"
                  class="drawer-close-btn"
                  aria-label="关闭历史抽屉"
                  @click="isDrawerOpen = false"
                >
                  ✕
                </button>
              </div>
            </div>

            <div v-if="historyList.length > 0" class="drawer-cards-strip">
              <div
                v-for="item in historyList"
                :key="item.id"
                class="history-card"
              >
                <div class="history-card-thumb">
                  <img v-if="item.previewUrl" :src="item.previewUrl" :alt="item.fileName" />
                  <div v-else class="card-thumb-fallback">
                    <span>GIF</span>
                  </div>
                  <span class="history-card-size-pill">{{ formatBytes(item.outputBytes) }}</span>
                </div>

                <div class="history-card-body">
                  <div class="history-card-title-row">
                    <strong class="history-card-filename" :title="item.fileName">{{ item.fileName }}</strong>
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
              <span>在此完成的 GIF 将保存在这里，支持一键定位、复制与参数回溯。</span>
            </div>
          </div>
        </div>
      </Transition>
    </main>
  </div>
</template>
