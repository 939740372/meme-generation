<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from "vue";
import {
  cancelRender,
  checkGitHubUpdate,
  copyFileToClipboard,
  extractVideoFrame,
  getBuiltinSfx,
  getBuiltinStickers,
  getDesktopInfo,
  getHardwareAccelerationInfo,
  getVideoAssetUrl,
  GITHUB_RELEASES_URL,
  isDesktopRuntime,
  listenForRenderEvents,
  openExternalUrl,
  pickAudioFile,
  pickMediaOutput,
  pickPng,
  pickVideo,
  probeMedia,
  readFileAsDataUrl,
  renderGif,
  revealFileInDir,
  type BgmOverlay,
  type BuiltinSfx,
  type BuiltinSticker,
  type DesktopInfo,
  type ExportFormat,
  type ExportRecord,
  type HardwareAccelerationInfo,
  type HwAccelPolicy,
  type ImageAdjustments,
  type MediaInfo,
  type RenderPlan,
  type UpdateInfo,
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
import {
  createQueueItem,
  deserializeQueue,
  getQueueStatusBadge,
  QUEUE_STORAGE_KEY,
  serializeQueue,
  type QueueItem,
} from "./lib/queue";

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
const exportFormat = ref<ExportFormat>("gif");
const includeAudio = ref(true);
const hwAccelInfo = ref<HardwareAccelerationInfo | null>(null);
const hwAccelPolicy = ref<HwAccelPolicy>("auto");
const status = ref<TaskStatus>("idle");
const progress = ref(0);
const statusMessage = ref("选择一个视频开始制作。");
const errorMessage = ref("");
const taskId = ref<string | null>(null);
const outputBytes = ref(0);
const stopListening = ref<(() => void) | null>(null);

const drawerTab = ref<"history" | "templates" | "stickers" | "queue" | "sfx">("history");
const customTemplates = ref<MemeTemplate[]>([]);
const isSaveTemplateModalOpen = ref(false);
const newTemplateName = ref("");
const templateNameInputRef = ref<HTMLInputElement | null>(null);

// 右侧属性检查器分栏与折叠状态
type InspectorTab = "canvas" | "text" | "media" | "export";
const activeInspectorTab = ref<InspectorTab>("canvas");
const showAdvancedText = ref(false);
const showExactTextCoords = ref(false);
const showExactCropCoords = ref(false);
const showExactStickerCoords = ref(false);

// 远程版本更新检查状态
const updateInfo = ref<UpdateInfo | null>(null);
const isCheckingUpdate = ref(false);
const isUpdateModalOpen = ref(false);
const appVersion = computed(() => desktopInfo.value?.version || "0.1.0");
const hasNewVersion = computed(() => Boolean(updateInfo.value?.hasUpdate));

async function handleCheckUpdate(silent = false): Promise<void> {
  if (isCheckingUpdate.value) return;
  isCheckingUpdate.value = true;
  try {
    const info = await checkGitHubUpdate(
      appVersion.value,
      desktopInfo.value?.platform || ""
    );
    updateInfo.value = info;
    if (info.hasUpdate) {
      if (!silent) {
        isUpdateModalOpen.value = true;
      } else {
        showToast(`🎉 发现新版本 v${info.latestVersion}，点击右上角版本号可一键更新`);
      }
    } else if (!silent) {
      isUpdateModalOpen.value = true;
      showToast(`当前已是最新版本 (v${appVersion.value})`);
    }
  } catch {
    if (!silent) {
      isUpdateModalOpen.value = true;
      showToast("无法连接 GitHub 检查更新，可直接前往 Releases 页面查看");
    }
  } finally {
    isCheckingUpdate.value = false;
  }
}

async function handleDownloadUpdate(): Promise<void> {
  const targetUrl = updateInfo.value?.downloadUrl || GITHUB_RELEASES_URL;
  await openExternalUrl(targetUrl);
  showToast("已在浏览器中打开安装包下载链接");
}

async function handleOpenReleasesPage(): Promise<void> {
  const pageUrl = updateInfo.value?.releasePageUrl || GITHUB_RELEASES_URL;
  await openExternalUrl(pageUrl);
}

function toggleDrawerTab(tab: "history" | "templates" | "stickers" | "queue" | "sfx"): void {
  if (isDrawerOpen.value && drawerTab.value === tab) {
    isDrawerOpen.value = false;
  } else {
    drawerTab.value = tab;
    isDrawerOpen.value = true;
  }
}

// 音频混流与音效状态
const builtinSfxList = ref<BuiltinSfx[]>([]);
const selectedSfxCategory = ref<"all" | "reaction" | "drama" | "funny">("all");
const bgmPath = ref("");
const bgmName = ref("");
const bgmDuration = ref(0);
const bgmVolume = ref(100);
const origAudioVolume = ref(100);
const playingSfxId = ref<string | null>(null);
const isPlayingActiveBgm = ref(false);
const localBgmDataUrl = ref<string>("");
let sfxAudioPlayer: HTMLAudioElement | null = null;

const filteredSfxList = computed(() => {
  if (selectedSfxCategory.value === "all") {
    return builtinSfxList.value;
  }
  return builtinSfxList.value.filter((s) => s.category === selectedSfxCategory.value);
});

const reactionSfxCount = computed(
  () => builtinSfxList.value.filter((s) => s.category === "reaction").length
);
const dramaSfxCount = computed(
  () => builtinSfxList.value.filter((s) => s.category === "drama").length
);
const funnySfxCount = computed(
  () => builtinSfxList.value.filter((s) => s.category === "funny").length
);

const bgmOverlay = computed<BgmOverlay | null>(() => {
  if (!bgmPath.value.trim()) {
    return null;
  }
  return {
    path: bgmPath.value,
    volume: bgmVolume.value / 100,
    origVolume: origAudioVolume.value / 100,
  };
});

function getSfxCategoryLabel(cat: string): string {
  switch (cat) {
    case "reaction":
      return "经典反响";
    case "drama":
      return "高能戏剧";
    case "funny":
      return "滑稽搞怪";
    default:
      return "音效素材";
  }
}

function togglePlaySfx(sfx: BuiltinSfx): void {
  if (playingSfxId.value === sfx.id) {
    if (sfxAudioPlayer) {
      sfxAudioPlayer.pause();
      sfxAudioPlayer.currentTime = 0;
    }
    playingSfxId.value = null;
    return;
  }
  if (sfxAudioPlayer) {
    sfxAudioPlayer.pause();
  }
  isPlayingActiveBgm.value = false;
  sfxAudioPlayer = new Audio(sfx.dataUrl);
  sfxAudioPlayer.volume = Math.min(1.0, bgmVolume.value / 100);
  playingSfxId.value = sfx.id;
  sfxAudioPlayer.play().catch(() => {});
  sfxAudioPlayer.onended = () => {
    playingSfxId.value = null;
  };
}

async function togglePlayActiveBgm(): Promise<void> {
  if (isPlayingActiveBgm.value) {
    if (sfxAudioPlayer) {
      sfxAudioPlayer.pause();
      sfxAudioPlayer.currentTime = 0;
    }
    isPlayingActiveBgm.value = false;
    return;
  }
  if (!bgmPath.value) return;
  if (sfxAudioPlayer) {
    sfxAudioPlayer.pause();
  }
  playingSfxId.value = null;
  const matched = builtinSfxList.value.find((s) => s.path === bgmPath.value);
  let src = matched ? matched.dataUrl : localBgmDataUrl.value;
  if (!src && bgmPath.value) {
    try {
      src = (await readFileAsDataUrl(bgmPath.value)) || "";
      localBgmDataUrl.value = src;
    } catch {
      src = "";
    }
  }
  if (!src) {
    showToast("无法读取音频试听");
    return;
  }
  sfxAudioPlayer = new Audio(src);
  sfxAudioPlayer.volume = Math.min(1.0, bgmVolume.value / 100);
  isPlayingActiveBgm.value = true;
  sfxAudioPlayer.play().catch(() => {
    isPlayingActiveBgm.value = false;
  });
  sfxAudioPlayer.onended = () => {
    isPlayingActiveBgm.value = false;
  };
}

function applySfx(sfx: BuiltinSfx): void {
  if (exportFormat.value !== "mp4") {
    setExportFormat("mp4");
    showToast("已自动切换为 MP4 短视频格式以承载音效");
  }
  bgmPath.value = sfx.path;
  bgmName.value = sfx.name;
  bgmDuration.value = sfx.duration;
  localBgmDataUrl.value = sfx.dataUrl;
  activeInspectorTab.value = "media";
  showToast(`已选用音效：${sfx.name}`);
}

async function handlePickAudio(): Promise<void> {
  const file = await pickAudioFile();
  if (file) {
    if (exportFormat.value !== "mp4") {
      setExportFormat("mp4");
      showToast("已自动切换为 MP4 短视频格式以承载音频");
    }
    bgmPath.value = file;
    const slashIdx = Math.max(file.lastIndexOf("/"), file.lastIndexOf("\\"));
    bgmName.value = slashIdx >= 0 ? file.slice(slashIdx + 1) : file;
    bgmDuration.value = 0;
    readFileAsDataUrl(file).then((dataUrl) => {
      if (dataUrl) localBgmDataUrl.value = dataUrl;
    }).catch(() => {});
    showToast(`已选用自定义音频：${bgmName.value}`);
  }
}

function clearBgm(): void {
  bgmPath.value = "";
  bgmName.value = "";
  bgmDuration.value = 0;
  localBgmDataUrl.value = "";
  if (sfxAudioPlayer) {
    sfxAudioPlayer.pause();
  }
  playingSfxId.value = null;
  isPlayingActiveBgm.value = false;
  showToast("已清除背景音乐 / 音效");
}

function openSfxDrawer(): void {
  drawerTab.value = "sfx";
  isDrawerOpen.value = true;
}

// 队列调度状态
const queueItems = ref<QueueItem[]>([]);
const isQueueRunning = ref(false);
const activeQueueItemId = ref<string | null>(null);
const queueTaskId = ref<string | null>(null);

const pendingQueueCount = computed(
  () => queueItems.value.filter((i) => i.status === "pending").length
);
const renderingQueueCount = computed(
  () => queueItems.value.filter((i) => i.status === "rendering").length
);
const completedQueueCount = computed(
  () => queueItems.value.filter((i) => i.status === "completed").length
);
const failedQueueCount = computed(
  () => queueItems.value.filter((i) => i.status === "failed").length
);
const hasPendingQueueItems = computed(() => pendingQueueCount.value > 0);

const builtinStickers = ref<BuiltinSticker[]>([]);
const selectedStickerCategory = ref<"all" | "classic" | "emotion" | "bubble">("all");
const imagePreviewUrl = ref<string>("");
const isResizingImage = ref(false);

const filteredStickers = computed(() => {
  if (selectedStickerCategory.value === "all") {
    return builtinStickers.value;
  }
  return builtinStickers.value.filter((s) => s.category === selectedStickerCategory.value);
});

const classicStickersCount = computed(
  () => builtinStickers.value.filter((s) => s.category === "classic").length
);
const emotionStickersCount = computed(
  () => builtinStickers.value.filter((s) => s.category === "emotion").length
);
const bubbleStickersCount = computed(
  () => builtinStickers.value.filter((s) => s.category === "bubble").length
);

function getCategoryLabel(cat: string): string {
  switch (cat) {
    case "classic":
      return "经典名梗";
    case "emotion":
      return "情绪表态";
    case "bubble":
      return "对话气泡";
    default:
      return "表情贴纸";
  }
}

function openStickersDrawer(): void {
  drawerTab.value = "stickers";
  isDrawerOpen.value = true;
}

function getMediaScaleFactor(): number {
  if (!media.value || media.value.width <= 0) return 1;
  return Math.max(0.5, Math.min(4.0, media.value.width / 480));
}

function applySticker(sticker: BuiltinSticker): void {
  useImage.value = true;
  imagePath.value = sticker.path;
  imagePreviewUrl.value = sticker.dataUrl;
  const scaleFactor = getMediaScaleFactor();
  const scaledWidth = media.value
    ? Math.max(24, Math.min(media.value.width, Math.round(sticker.defaultWidth * scaleFactor)))
    : sticker.defaultWidth;
  imageWidth.value = scaledWidth;
  if (media.value) {
    imageX.value = Math.max(0, Math.round((media.value.width - scaledWidth) / 2));
    imageY.value = Math.max(0, Math.round((media.value.height - scaledWidth) / 2));
  } else {
    imageX.value = 50;
    imageY.value = 50;
  }
  activeInspectorTab.value = "media";
  isDrawerOpen.value = false;
  showToast(`已添加贴纸「${sticker.name}」到画面`);
}

function scaleImageTo(ratio: number): void {
  useImage.value = true;
  if (!media.value) {
    imageWidth.value = Math.round(120 * ratio);
    return;
  }
  const base = Math.round(130 * getMediaScaleFactor());
  imageWidth.value = Math.max(20, Math.min(media.value.width, Math.round(base * ratio)));
}

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

// 视频画面与动态循环预览
const previewVideoRef = ref<HTMLVideoElement | null>(null);
const videoAssetUrl = ref<string>("");
const extractedFrameUrl = ref<string>("");
const isPreviewPlaying = ref<boolean>(false);
const isPreviewMuted = ref<boolean>(true);
const isNativeVideoSupported = ref<boolean>(true);
const isExtractingFrame = ref<boolean>(false);
let frameExtractTimer: ReturnType<typeof setTimeout> | null = null;

async function refreshExtractedFrame(targetSecs?: number): Promise<void> {
  if (!inputPath.value || !isDesktop.value) return;
  const time = targetSecs !== undefined ? targetSecs : startSecs.value;
  if (isExtractingFrame.value) return;
  isExtractingFrame.value = true;
  try {
    const frame = await extractVideoFrame(inputPath.value, time, 640);
    if (frame) {
      extractedFrameUrl.value = frame;
    }
  } catch {
    // ignore
  } finally {
    isExtractingFrame.value = false;
  }
}

function scheduleFrameRefresh(targetSecs?: number): void {
  if (frameExtractTimer) clearTimeout(frameExtractTimer);
  frameExtractTimer = setTimeout(() => {
    refreshExtractedFrame(targetSecs);
  }, 100);
}

function setupVideoPreview(filePath: string): void {
  if (!filePath) {
    videoAssetUrl.value = "";
    extractedFrameUrl.value = "";
    isPreviewPlaying.value = false;
    return;
  }
  isNativeVideoSupported.value = true;
  videoAssetUrl.value = getVideoAssetUrl(filePath);
  refreshExtractedFrame(startSecs.value);
}

function togglePlayPreview(): void {
  if (!previewVideoRef.value) {
    if (!isNativeVideoSupported.value) {
      showToast("当前格式已启用逐帧高清预览，拖动时间轴即可查看每一帧");
    }
    return;
  }
  if (isPreviewPlaying.value) {
    previewVideoRef.value.pause();
    isPreviewPlaying.value = false;
  } else {
    if (previewVideoRef.value.currentTime < startSecs.value || previewVideoRef.value.currentTime >= endSecs.value) {
      previewVideoRef.value.currentTime = startSecs.value;
    }
    if (currentVideoFilter.value === "fast2x") {
      previewVideoRef.value.playbackRate = 2.0;
    } else if (currentVideoFilter.value === "slow05x") {
      previewVideoRef.value.playbackRate = 0.5;
    } else {
      previewVideoRef.value.playbackRate = 1.0;
    }
    previewVideoRef.value
      .play()
      .then(() => {
        isPreviewPlaying.value = true;
      })
      .catch(() => {
        isPreviewPlaying.value = false;
      });
  }
}

function togglePreviewMute(): void {
  isPreviewMuted.value = !isPreviewMuted.value;
}

function handleVideoTimeUpdate(): void {
  if (!previewVideoRef.value) return;
  const cur = previewVideoRef.value.currentTime;
  if (cur >= endSecs.value || cur < startSecs.value - 0.05) {
    previewVideoRef.value.currentTime = startSecs.value;
    if (isPreviewPlaying.value) {
      previewVideoRef.value.play().catch(() => {});
    }
  }
}

function handleVideoLoadedMetadata(): void {
  if (previewVideoRef.value) {
    previewVideoRef.value.currentTime = startSecs.value;
  }
}

function handleVideoError(): void {
  isNativeVideoSupported.value = false;
  isPreviewPlaying.value = false;
  refreshExtractedFrame(startSecs.value);
}

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
    rotation: 0,
    arcRadius: 0,
    isItalic: false,
    shadowOffset: 0,
    shadowColor: "#000000",
  },
]);
const activeLayerId = ref<string>("layer-1");

const activeLayer = computed(() => {
  return textLayers.value.find((l) => l.id === activeLayerId.value) || textLayers.value[0];
});

function addTextLayer(): void {
  if (textLayers.value.length >= 4) return;
  useText.value = true;
  const newIndex = textLayers.value.length + 1;
  const id = `layer-${Date.now()}`;
  const scaleFactor = getMediaScaleFactor();
  const scaledSize = Math.min(512, Math.max(12, Math.round(32 * scaleFactor)));
  const scaledStroke = Math.max(1, Math.round(2 * scaleFactor));
  const stepY = Math.round(56 * scaleFactor);
  const defaultX = media.value ? Math.max(16, Math.round(media.value.width * 0.08)) : 24;
  const defaultY = media.value
    ? Math.min(media.value.height - scaledSize - 12, Math.round(24 * scaleFactor) + (newIndex - 1) * stepY)
    : 24 + (newIndex - 1) * 60;
  const newLayer: CanvasTextLayer = {
    id,
    text: newIndex === 2 ? "第二行吐槽" : `文字图层 ${newIndex}`,
    x: defaultX,
    y: defaultY,
    size: scaledSize,
    color: "#ffffff",
    strokeWidth: scaledStroke,
    strokeColor: "#000000",
    useBox: false,
    boxColor: "#000000",
    boxOpacity: 0.60,
    visible: true,
    rotation: 0,
    arcRadius: 0,
    isItalic: false,
    shadowOffset: 0,
    shadowColor: "#000000",
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
  set: (v: string) => {
    if (activeLayer.value) {
      activeLayer.value.text = v;
      if (v.trim().length > 0 && !isRestoringDraft) {
        useText.value = true;
      }
    }
  },
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
  set: (v: number) => {
    if (activeLayer.value) {
      activeLayer.value.size = v;
      if (!isRestoringDraft) useText.value = true;
    }
  },
});
const textColor = computed({
  get: () => activeLayer.value?.color ?? "#ffffff",
  set: (v: string) => {
    if (activeLayer.value) {
      activeLayer.value.color = v;
      if (!isRestoringDraft) useText.value = true;
    }
  },
});
const textStrokeWidth = computed({
  get: () => activeLayer.value?.strokeWidth ?? 0,
  set: (v: number) => {
    if (activeLayer.value) {
      activeLayer.value.strokeWidth = v;
      if (!isRestoringDraft) useText.value = true;
    }
  },
});
const textStrokeColor = computed({
  get: () => activeLayer.value?.strokeColor ?? "#000000",
  set: (v: string) => {
    if (activeLayer.value) {
      activeLayer.value.strokeColor = v;
      if (!isRestoringDraft) useText.value = true;
    }
  },
});
const textUseBox = computed({
  get: () => activeLayer.value?.useBox ?? false,
  set: (v: boolean) => {
    if (activeLayer.value) {
      activeLayer.value.useBox = v;
      if (!isRestoringDraft) useText.value = true;
    }
  },
});
const textBoxColor = computed({
  get: () => activeLayer.value?.boxColor ?? "#000000",
  set: (v: string) => { if (activeLayer.value) activeLayer.value.boxColor = v; },
});
const textBoxOpacity = computed({
  get: () => activeLayer.value?.boxOpacity ?? 0.6,
  set: (v: number) => { if (activeLayer.value) activeLayer.value.boxOpacity = v; },
});
const textRotation = computed({
  get: () => activeLayer.value?.rotation ?? 0,
  set: (v: number) => {
    if (activeLayer.value) {
      activeLayer.value.rotation = v;
      if (!isRestoringDraft) useText.value = true;
    }
  },
});
const textArcRadius = computed({
  get: () => activeLayer.value?.arcRadius ?? 0,
  set: (v: number) => {
    if (activeLayer.value) {
      activeLayer.value.arcRadius = v;
      if (!isRestoringDraft) useText.value = true;
    }
  },
});
const textIsItalic = computed({
  get: () => activeLayer.value?.isItalic ?? false,
  set: (v: boolean) => {
    if (activeLayer.value) {
      activeLayer.value.isItalic = v;
      if (!isRestoringDraft) useText.value = true;
    }
  },
});
const textShadowOffset = computed({
  get: () => activeLayer.value?.shadowOffset ?? 0,
  set: (v: number) => {
    if (activeLayer.value) {
      activeLayer.value.shadowOffset = v;
      if (!isRestoringDraft) useText.value = true;
    }
  },
});
const textShadowColor = computed({
  get: () => activeLayer.value?.shadowColor ?? "#000000",
  set: (v: string) => { if (activeLayer.value) activeLayer.value.shadowColor = v; },
});

function setRotationPreset(deg: number): void {
  textRotation.value = deg;
}

const hasAdvancedTextActive = computed(() => {
  return Boolean(
    textRotation.value !== 0 ||
    textArcRadius.value !== 0 ||
    textIsItalic.value ||
    textShadowOffset.value > 0
  );
});

const useImage = ref(false);
const imagePath = ref("");
const imageX = ref(24);
const imageY = ref(24);
const imageWidth = ref(120);

watch(useImage, (enabled) => {
  if (enabled && !imagePath.value && builtinStickers.value.length > 0 && !isRestoringDraft) {
    const first = builtinStickers.value[0];
    imagePath.value = first.path;
    imagePreviewUrl.value = first.dataUrl;
    const scaleFactor = getMediaScaleFactor();
    const scaledWidth = media.value
      ? Math.max(24, Math.min(media.value.width, Math.round(first.defaultWidth * scaleFactor)))
      : first.defaultWidth;
    imageWidth.value = scaledWidth;
    if (media.value) {
      imageX.value = Math.max(0, Math.round((media.value.width - scaledWidth) / 2));
      imageY.value = Math.max(0, Math.round((media.value.height - scaledWidth) / 2));
    }
  }
});

// 画面画风与速度特效
export type VideoFilterKey =
  | "none"
  | "grayscale"
  | "contrast"
  | "vintage"
  | "invert"
  | "fast2x"
  | "slow05x"
  | "reverse";

export interface VideoFilterOption {
  key: VideoFilterKey;
  label: string;
  icon: string;
  badge: string;
  cssFilter: string;
}

const VIDEO_FILTERS: VideoFilterOption[] = [
  { key: "none", label: "原色", icon: "✨", badge: "默认", cssFilter: "none" },
  { key: "grayscale", label: "黑白", icon: "🖤", badge: "致郁emo", cssFilter: "grayscale(100%)" },
  { key: "contrast", label: "高反差", icon: "⚡", badge: "戏剧感", cssFilter: "contrast(160%) saturate(140%)" },
  { key: "vintage", label: "复古", icon: "🎞️", badge: "港片暖色", cssFilter: "sepia(55%) contrast(110%)" },
  { key: "invert", label: "反色", icon: "🧪", badge: "精神污染", cssFilter: "invert(100%)" },
  { key: "fast2x", label: "2x加速", icon: "🚀", badge: "鬼畜魔性", cssFilter: "none" },
  { key: "slow05x", label: "0.5x慢放", icon: "🐢", badge: "慢速特写", cssFilter: "none" },
  { key: "reverse", label: "倒放", icon: "⏪", badge: "物理逆转", cssFilter: "none" },
];

const currentVideoFilter = ref<VideoFilterKey>("none");

const currentFilterOption = computed(
  () => VIDEO_FILTERS.find((f) => f.key === currentVideoFilter.value) || VIDEO_FILTERS[0]
);

function setVideoFilter(key: VideoFilterKey): void {
  currentVideoFilter.value = key;
  onManualParameterChange();
  showToast(`已应用画风特效: ${currentFilterOption.value.label} · ${currentFilterOption.value.badge}`);
}

// 全帧画质与色彩精修 (亮度、对比度、饱和度、锐化、模糊、色相、暗角)
const imageAdjustments = reactive<Required<ImageAdjustments>>({
  brightness: 0,
  contrast: 0,
  saturation: 0,
  sharpen: 0,
  blur: 0,
  hue: 0,
  vignette: false,
});

export interface AdjustmentPreset {
  id: string;
  label: string;
  icon: string;
  values: {
    brightness: number;
    contrast: number;
    saturation: number;
    sharpen: number;
    blur: number;
    hue: number;
    vignette: boolean;
  };
}

const ADJUSTMENT_PRESETS: AdjustmentPreset[] = [
  {
    id: "hd-sharpen",
    label: "高清锐化",
    icon: "✨",
    values: { brightness: 6, contrast: 15, saturation: 12, sharpen: 65, blur: 0, hue: 0, vignette: false },
  },
  {
    id: "brighten",
    label: "提亮去暗",
    icon: "☀️",
    values: { brightness: 28, contrast: 12, saturation: 10, sharpen: 25, blur: 0, hue: 0, vignette: false },
  },
  {
    id: "vivid",
    label: "浓郁鲜艳",
    icon: "🔥",
    values: { brightness: 5, contrast: 22, saturation: 48, sharpen: 30, blur: 0, hue: 0, vignette: false },
  },
  {
    id: "cyber-cool",
    label: "赛博冷调",
    icon: "❄️",
    values: { brightness: 4, contrast: 25, saturation: 25, sharpen: 35, blur: 0, hue: -28, vignette: true },
  },
  {
    id: "soft-blur",
    label: "柔焦朦胧",
    icon: "🌫️",
    values: { brightness: 8, contrast: -8, saturation: -10, sharpen: 0, blur: 32, hue: 0, vignette: true },
  },
];

const hasActiveAdjustments = computed(() => {
  return (
    imageAdjustments.brightness !== 0 ||
    imageAdjustments.contrast !== 0 ||
    imageAdjustments.saturation !== 0 ||
    imageAdjustments.sharpen > 0 ||
    imageAdjustments.blur > 0 ||
    imageAdjustments.hue !== 0 ||
    imageAdjustments.vignette
  );
});

const activeAdjustmentsSummary = computed(() => {
  const tags: string[] = [];
  if (imageAdjustments.brightness !== 0) {
    tags.push(`亮度${imageAdjustments.brightness > 0 ? `+${imageAdjustments.brightness}` : imageAdjustments.brightness}`);
  }
  if (imageAdjustments.contrast !== 0) {
    tags.push(`对比${imageAdjustments.contrast > 0 ? `+${imageAdjustments.contrast}` : imageAdjustments.contrast}`);
  }
  if (imageAdjustments.saturation !== 0) {
    tags.push(`饱和${imageAdjustments.saturation > 0 ? `+${imageAdjustments.saturation}` : imageAdjustments.saturation}`);
  }
  if (imageAdjustments.sharpen > 0) {
    tags.push(`锐化${imageAdjustments.sharpen}`);
  }
  if (imageAdjustments.blur > 0) {
    tags.push(`模糊${imageAdjustments.blur}`);
  }
  if (imageAdjustments.hue !== 0) {
    tags.push(`色相${imageAdjustments.hue > 0 ? `+${imageAdjustments.hue}°` : `${imageAdjustments.hue}°`}`);
  }
  if (imageAdjustments.vignette) {
    tags.push("暗角");
  }
  return tags.length > 0 ? tags.join(" · ") : "默认原片";
});

function resetImageAdjustments(silent = false): void {
  imageAdjustments.brightness = 0;
  imageAdjustments.contrast = 0;
  imageAdjustments.saturation = 0;
  imageAdjustments.sharpen = 0;
  imageAdjustments.blur = 0;
  imageAdjustments.hue = 0;
  imageAdjustments.vignette = false;
  if (!silent) {
    showToast("已重置全帧画质与色彩精修");
  }
}

function applyAdjustmentPreset(preset: AdjustmentPreset): void {
  imageAdjustments.brightness = preset.values.brightness;
  imageAdjustments.contrast = preset.values.contrast;
  imageAdjustments.saturation = preset.values.saturation;
  imageAdjustments.sharpen = preset.values.sharpen;
  imageAdjustments.blur = preset.values.blur;
  imageAdjustments.hue = preset.values.hue;
  imageAdjustments.vignette = preset.values.vignette;
  showToast(`已应用全帧画质预设：${preset.icon} ${preset.label}`);
}

const sharpenKernelMatrix = computed(() => {
  const k = Number(((imageAdjustments.sharpen / 100) * 1.6).toFixed(3));
  const center = Number((1 + 4 * k).toFixed(3));
  const neg = Number((-k).toFixed(3));
  return `0 ${neg} 0 ${neg} ${center} ${neg} 0 ${neg} 0`;
});

const combinedMediaFilterCss = computed(() => {
  const filters: string[] = [];
  if (currentFilterOption.value.cssFilter && currentFilterOption.value.cssFilter !== "none") {
    filters.push(currentFilterOption.value.cssFilter);
  }
  if (imageAdjustments.brightness !== 0) {
    const bPct = Math.max(20, Math.round(100 + imageAdjustments.brightness * 0.65));
    filters.push(`brightness(${bPct}%)`);
  }
  if (imageAdjustments.contrast !== 0) {
    const cPct = Math.max(10, Math.round(100 + imageAdjustments.contrast));
    filters.push(`contrast(${cPct}%)`);
  }
  if (imageAdjustments.saturation !== 0) {
    const sPct =
      imageAdjustments.saturation >= 0
        ? Math.round(100 + imageAdjustments.saturation * 1.5)
        : Math.max(0, Math.round(100 + imageAdjustments.saturation));
    filters.push(`saturate(${sPct}%)`);
  }
  if (imageAdjustments.hue !== 0) {
    filters.push(`hue-rotate(${imageAdjustments.hue}deg)`);
  }
  if (imageAdjustments.blur > 0) {
    const px = ((imageAdjustments.blur / 100) * 8).toFixed(2);
    filters.push(`blur(${px}px)`);
  }
  if (imageAdjustments.sharpen > 0) {
    filters.push("url(#studio-sharpen-filter)");
  }
  return filters.length > 0 ? filters.join(" ") : "none";
});

const stageRef = ref<HTMLElement | null>(null);
const timelineTrackRef = ref<HTMLElement | null>(null);
const stageSize = reactive({ width: 0, height: 0 });
const isDraggingText = ref(false);
const isDraggingImage = ref(false);

const viewportRect = computed(() => {
  if (!media.value || stageSize.width <= 0 || stageSize.height <= 0) {
    return {
      left: 0,
      top: 0,
      width: stageSize.width || 0,
      height: stageSize.height || 0,
      scale: 1,
    };
  }

  const mediaW = media.value.width;
  const mediaH = media.value.height;
  if (mediaW <= 0 || mediaH <= 0) {
    return {
      left: 0,
      top: 0,
      width: stageSize.width,
      height: stageSize.height,
      scale: 1,
    };
  }

  const scaleW = stageSize.width / mediaW;
  const scaleH = stageSize.height / mediaH;
  const scale = Math.min(scaleW, scaleH);

  const width = Math.round(mediaW * scale);
  const height = Math.round(mediaH * scale);
  const left = Math.round((stageSize.width - width) / 2);
  const top = Math.round((stageSize.height - height) / 2);

  return { left, top, width, height, scale };
});

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

  const scale = viewportRect.value.scale;
  if (media.value && scale > 0) {
    left = Math.round(layer.x * scale);
    top = Math.round(layer.y * scale);
    fontSize = Math.max(10, Math.min(Math.round(layer.size * scale), 240));
    if (strokeW > 0) {
      strokeW = Math.max(1, Math.round(layer.strokeWidth * scale));
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
  if (layer.shadowOffset && layer.shadowOffset > 0) {
    const sOffset = Math.max(1, Math.round(layer.shadowOffset * scale));
    const sCol = layer.shadowColor || "#000000";
    styles.textShadow = `${sOffset}px ${sOffset}px 2px ${sCol}`;
  }
  if (layer.isItalic) {
    styles.fontStyle = "italic";
  }
  if (layer.rotation && layer.rotation !== 0) {
    styles.transform = `rotate(${layer.rotation}deg)`;
    styles.transformOrigin = "center center";
  }
  return styles;
}

function getArcCharStyle(layer: CanvasTextLayer, charIdx: number, totalChars: number): Record<string, string | number | undefined> {
  if (totalChars <= 1) return {};
  const arc = layer.arcRadius ?? 0;
  const t = (charIdx - (totalChars - 1) / 2) / ((totalChars - 1) / 2);
  const maxAngle = 70 * (arc / 100);
  const deltaTheta = t * (maxAngle / 2);
  const dropAmplitude = (arc / 100) * 20;
  const deltaY = dropAmplitude * (t * t);
  return {
    display: "inline-block",
    transform: `translateY(${deltaY.toFixed(1)}px) rotate(${deltaTheta.toFixed(1)}deg)`,
    transformOrigin: "center center",
  };
}

const imageDisplayLeft = computed(() => {
  if (!media.value || viewportRect.value.scale <= 0) return imageX.value;
  return Math.round(imageX.value * viewportRect.value.scale);
});

const imageDisplayTop = computed(() => {
  if (!media.value || viewportRect.value.scale <= 0) return imageY.value;
  return Math.round(imageY.value * viewportRect.value.scale);
});

const imageDisplayWidth = computed(() => {
  if (!media.value || viewportRect.value.scale <= 0) return imageWidth.value;
  return Math.max(28, Math.round(imageWidth.value * viewportRect.value.scale));
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
  activeInspectorTab.value = "text";
  isDraggingText.value = true;
  const startClientX = event.clientX;
  const startClientY = event.clientY;
  const initX = layer.x;
  const initY = layer.y;
  const scale = viewportRect.value.scale;
  if (scale <= 0) return;

  const onPointerMove = (e: PointerEvent): void => {
    const deltaX = (e.clientX - startClientX) / scale;
    const deltaY = (e.clientY - startClientY) / scale;
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
  activeInspectorTab.value = "media";
  isDraggingImage.value = true;
  const startClientX = event.clientX;
  const startClientY = event.clientY;
  const initX = imageX.value;
  const initY = imageY.value;
  const scale = viewportRect.value.scale;
  if (scale <= 0) return;

  const onPointerMove = (e: PointerEvent): void => {
    const deltaX = (e.clientX - startClientX) / scale;
    const deltaY = (e.clientY - startClientY) / scale;
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

function startResizeImage(event: PointerEvent): void {
  if (!media.value || !stageRef.value) return;
  event.preventDefault();
  event.stopPropagation();
  activeInspectorTab.value = "media";
  isResizingImage.value = true;
  const startClientX = event.clientX;
  const initWidth = imageWidth.value;
  const scale = viewportRect.value.scale;
  if (scale <= 0) return;

  const onPointerMove = (e: PointerEvent): void => {
    const deltaX = (e.clientX - startClientX) / scale;
    const newWidth = Math.round(initWidth + deltaX);
    imageWidth.value = Math.max(20, Math.min(media.value!.width, newWidth));
  };

  const onPointerUp = (): void => {
    isResizingImage.value = false;
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);
  };

  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
}

const cropDisplayLeft = computed(() => {
  if (!media.value || viewportRect.value.scale <= 0) return crop.x;
  return Math.round(crop.x * viewportRect.value.scale);
});

const cropDisplayTop = computed(() => {
  if (!media.value || viewportRect.value.scale <= 0) return crop.y;
  return Math.round(crop.y * viewportRect.value.scale);
});

const cropDisplayWidth = computed(() => {
  if (!media.value || viewportRect.value.scale <= 0) return crop.width;
  return Math.max(16, Math.round(crop.width * viewportRect.value.scale));
});

const cropDisplayHeight = computed(() => {
  if (!media.value || viewportRect.value.scale <= 0) return crop.height;
  return Math.max(16, Math.round(crop.height * viewportRect.value.scale));
});

const isCropNearTop = computed(() => cropDisplayTop.value < 28);

const isDraggingCrop = ref(false);
const activeCropHandle = ref<string | null>(null);

function startDragCrop(event: PointerEvent): void {
  if (!media.value || !stageRef.value) return;
  event.preventDefault();
  event.stopPropagation();
  activeInspectorTab.value = "canvas";
  isDraggingCrop.value = true;
  activeCropHandle.value = "move";

  const startClientX = event.clientX;
  const startClientY = event.clientY;
  const initX = crop.x;
  const initY = crop.y;
  const scale = viewportRect.value.scale;
  if (scale <= 0) return;

  const maxX = Math.max(0, media.value.width - crop.width);
  const maxY = Math.max(0, media.value.height - crop.height);

  const onPointerMove = (e: PointerEvent): void => {
    const deltaX = (e.clientX - startClientX) / scale;
    const deltaY = (e.clientY - startClientY) / scale;
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
  activeInspectorTab.value = "canvas";
  isDraggingCrop.value = true;
  activeCropHandle.value = handle;

  const startClientX = event.clientX;
  const startClientY = event.clientY;
  const initX = crop.x;
  const initY = crop.y;
  const initW = crop.width;
  const initH = crop.height;
  const scale = viewportRect.value.scale;
  if (scale <= 0) return;

  const minW = 20;
  const minH = 20;
  const mediaW = media.value.width;
  const mediaH = media.value.height;

  const onPointerMove = (e: PointerEvent): void => {
    const deltaX = (e.clientX - startClientX) / scale;
    const deltaY = (e.clientY - startClientY) / scale;

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
  crop.width = Math.min(crop.width, media.value.width);
  crop.height = Math.min(crop.height, media.value.height);
  crop.x = Math.max(0, Math.min(media.value.width - crop.width, Math.floor((media.value.width - crop.width) / 2)));
  crop.y = Math.max(0, Math.min(media.value.height - crop.height, Math.floor((media.value.height - crop.height) / 2)));
}

function cropReset(): void {
  if (!media.value) return;
  crop.x = 0;
  crop.y = 0;
  crop.width = media.value.width;
  crop.height = media.value.height;
}

function cropSquare(): void {
  if (!media.value) return;
  const side = Math.min(media.value.width, media.value.height);
  crop.width = side;
  crop.height = side;
  cropCenter();
}

function crop16x9(): void {
  if (!media.value) return;
  const videoAspect = media.value.width / media.value.height;
  const targetAspect = 16 / 9;
  if (videoAspect >= targetAspect) {
    crop.height = media.value.height;
    crop.width = Math.round(crop.height * targetAspect);
  } else {
    crop.width = media.value.width;
    crop.height = Math.round(crop.width / targetAspect);
  }
  cropCenter();
}

function crop9x16(): void {
  if (!media.value) return;
  const videoAspect = media.value.width / media.value.height;
  const targetAspect = 9 / 16;
  if (videoAspect >= targetAspect) {
    crop.height = media.value.height;
    crop.width = Math.round(crop.height * targetAspect);
  } else {
    crop.width = media.value.width;
    crop.height = Math.round(crop.width / targetAspect);
  }
  cropCenter();
}

function crop4x3(): void {
  if (!media.value) return;
  const videoAspect = media.value.width / media.value.height;
  const targetAspect = 4 / 3;
  if (videoAspect >= targetAspect) {
    crop.height = media.value.height;
    crop.width = Math.round(crop.height * targetAspect);
  } else {
    crop.width = media.value.width;
    crop.height = Math.round(crop.width / targetAspect);
  }
  cropCenter();
}

watch(
  [() => crop.x, () => crop.y, () => crop.width, () => crop.height],
  ([x, y, w, h]) => {
    if (!media.value) return;
    const maxW = media.value.width;
    const maxH = media.value.height;
    if (w > maxW) crop.width = maxW;
    if (h > maxH) crop.height = maxH;
    if (crop.width > 0 && x + crop.width > maxW) {
      crop.x = Math.max(0, maxW - crop.width);
    }
    if (crop.height > 0 && y + crop.height > maxH) {
      crop.y = Math.max(0, maxH - crop.height);
    }
  }
);

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

function basenameWithoutExtension(path: string): string {
  const fileName = path.split(/[\\/]/).pop() ?? "meme";
  return fileName.replace(/\.[^.]+$/, "") || "meme";
}

function defaultOutputPath(path: string, format: ExportFormat = exportFormat.value): string {
  const separator = path.includes("\\") ? "\\" : "/";
  const parts = path.split(/[\\/]/);
  parts.pop();
  const parent = parts.join(separator);
  const fileName = `${basenameWithoutExtension(path)}-meme.${format}`;
  return parent ? `${parent}${separator}${fileName}` : fileName;
}

function ensureFormatExtension(path: string, format: ExportFormat = exportFormat.value): string {
  const clean = path.trim();
  if (!clean) return "";
  const lower = clean.toLowerCase();
  if (lower.endsWith(`.${format}`)) {
    return clean;
  }
  if (lower.endsWith(".gif") || lower.endsWith(".webp") || lower.endsWith(".avif") || lower.endsWith(".mp4")) {
    const idx = clean.lastIndexOf(".");
    return `${clean.substring(0, idx)}.${format}`;
  }
  return `${clean}.${format}`;
}

function setExportFormat(fmt: ExportFormat): void {
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
  crop.x = 0;
  crop.y = 0;
  crop.width = nextMedia.width;
  crop.height = nextMedia.height;

  // 若当前尚未启用文字图层且仅有默认图层，按源视频分辨率自适应设置合理的默认字号与位置
  const scaleFactor = Math.max(0.5, Math.min(4.0, nextMedia.width / 480));
  if (!useText.value && textLayers.value.length === 1) {
    const firstLayer = textLayers.value[0];
    const scaledSize = Math.min(512, Math.max(16, Math.round(38 * scaleFactor)));
    const scaledStroke = Math.max(2, Math.round(2.5 * scaleFactor));
    const estW = Math.max(1, firstLayer.text.length) * scaledSize;
    firstLayer.size = scaledSize;
    firstLayer.strokeWidth = scaledStroke;
    firstLayer.x = Math.max(16, Math.round((nextMedia.width - estW) / 2));
    firstLayer.y = Math.max(16, Math.round(nextMedia.height * 0.78));
  }

  nextTick(() => {
    updateStageSize();
  });
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
    setupVideoPreview(selected);
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
      imagePreviewUrl.value = (await readFileAsDataUrl(selected)) || "";
      showToast("已载入本地 PNG 图案");
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
      rotation: l.rotation ? Number(l.rotation) : 0,
      arcRadius: l.arcRadius ? Number(l.arcRadius) : 0,
      isItalic: Boolean(l.isItalic),
      shadowOffset: l.shadowOffset ? Number(l.shadowOffset) : 0,
      shadowColor: l.shadowColor || "#000000",
    }));

  return {
    inputPath: inputPath.value,
    outputPath: finalOut,
    format: exportFormat.value,
    includeAudio: includeAudio.value,
    hwAccel: hwAccelPolicy.value,
    startSecs: Number(startSecs.value),
    endSecs: Number(endSecs.value),
    crop: useCrop.value ? { ...crop } : null,
    outputWidth: Number(outputWidth.value),
    fps: Number(fps.value),
    quality: currentQuality.value,
    videoFilter: currentVideoFilter.value !== "none" ? currentVideoFilter.value : undefined,
    imageAdjustments: hasActiveAdjustments.value
      ? {
          brightness: Number(imageAdjustments.brightness),
          contrast: Number(imageAdjustments.contrast),
          saturation: Number(imageAdjustments.saturation),
          sharpen: Number(imageAdjustments.sharpen),
          blur: Number(imageAdjustments.blur),
          hue: Number(imageAdjustments.hue),
          vignette: Boolean(imageAdjustments.vignette),
        }
      : null,
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
    hasAudio: media.value?.hasAudio ?? true,
    bgmOverlay: bgmOverlay.value,
    sourceWidth: media.value?.width,
    sourceHeight: media.value?.height,
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
  imagePreviewUrl.value = "";
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
        setupVideoPreview(p.inputPath);
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
      rotation: l.rotation ?? 0,
      arcRadius: l.arcRadius ?? 0,
      isItalic: l.isItalic ?? false,
      shadowOffset: l.shadowOffset ?? 0,
      shadowColor: l.shadowColor ?? "#000000",
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
        rotation: p.textOverlay.rotation ?? 0,
        arcRadius: p.textOverlay.arcRadius ?? 0,
        isItalic: p.textOverlay.isItalic ?? false,
        shadowOffset: p.textOverlay.shadowOffset ?? 0,
        shadowColor: p.textOverlay.shadowColor ?? "#000000",
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
    readFileAsDataUrl(p.imageOverlay.path).then((url) => {
      if (url) imagePreviewUrl.value = url;
    });
  } else {
    useImage.value = false;
    imagePath.value = "";
    imagePreviewUrl.value = "";
  }

  if (p.format === "mp4" || record.filePath.toLowerCase().endsWith(".mp4")) {
    exportFormat.value = "mp4";
  } else if (p.format === "avif" || record.filePath.toLowerCase().endsWith(".avif")) {
    exportFormat.value = "avif";
  } else if (p.format === "webp" || record.filePath.toLowerCase().endsWith(".webp")) {
    exportFormat.value = "webp";
  } else {
    exportFormat.value = "gif";
  }

  if (p.includeAudio !== undefined) {
    includeAudio.value = p.includeAudio;
  }

  if (p.hwAccel !== undefined) {
    hwAccelPolicy.value = p.hwAccel;
  }

  if (p.videoFilter) {
    currentVideoFilter.value = p.videoFilter as VideoFilterKey;
  } else {
    currentVideoFilter.value = "none";
  }

  if (p.imageAdjustments) {
    imageAdjustments.brightness = p.imageAdjustments.brightness ?? 0;
    imageAdjustments.contrast = p.imageAdjustments.contrast ?? 0;
    imageAdjustments.saturation = p.imageAdjustments.saturation ?? 0;
    imageAdjustments.sharpen = p.imageAdjustments.sharpen ?? 0;
    imageAdjustments.blur = p.imageAdjustments.blur ?? 0;
    imageAdjustments.hue = p.imageAdjustments.hue ?? 0;
    imageAdjustments.vignette = Boolean(p.imageAdjustments.vignette);
  } else {
    resetImageAdjustments(true);
  }

  if (p.bgmOverlay) {
    bgmPath.value = p.bgmOverlay.path;
    const slashIdx = Math.max(p.bgmOverlay.path.lastIndexOf("/"), p.bgmOverlay.path.lastIndexOf("\\"));
    bgmName.value = slashIdx >= 0 ? p.bgmOverlay.path.slice(slashIdx + 1) : p.bgmOverlay.path;
    if (p.bgmOverlay.volume !== undefined) {
      bgmVolume.value = Math.round(p.bgmOverlay.volume * 100);
    }
    if (p.bgmOverlay.origVolume !== undefined) {
      origAudioVolume.value = Math.round(p.bgmOverlay.origVolume * 100);
    }
  } else {
    bgmPath.value = "";
    bgmName.value = "";
    bgmDuration.value = 0;
    bgmVolume.value = 100;
    origAudioVolume.value = 100;
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
    return;
  }
  if (e.code === "Space") {
    const active = document.activeElement;
    if (active && (active.tagName === "INPUT" || active.tagName === "TEXTAREA")) {
      return;
    }
    if (media.value) {
      e.preventDefault();
      togglePlayPreview();
    }
  }
}

watch(startSecs, (newVal) => {
  if (previewVideoRef.value && !isPreviewPlaying.value) {
    try {
      previewVideoRef.value.currentTime = newVal;
    } catch {
      // ignore
    }
  }
  scheduleFrameRefresh(newVal);
});

async function connectDesktopEvents(): Promise<void> {
  if (!isDesktop.value) {
    return;
  }

  try {
    stopListening.value = await listenForRenderEvents({
      onProgress: (event) => {
        // 队列任务事件
        if (queueTaskId.value && event.taskId === queueTaskId.value) {
          const item = queueItems.value.find((i) => i.id === activeQueueItemId.value);
          if (item) {
            item.progress = event.progress;
            item.stageMessage = event.message;
          }
          return;
        }

        if (taskId.value && event.taskId !== taskId.value) {
          return;
        }
        taskId.value = event.taskId;
        status.value = "rendering";
        progress.value = event.progress;
        statusMessage.value = event.message;
      },
      onCompleted: async (event) => {
        // 队列任务事件
        if (queueTaskId.value && event.taskId === queueTaskId.value) {
          const item = queueItems.value.find((i) => i.id === activeQueueItemId.value);
          if (item) {
            item.status = "completed";
            item.progress = 100;
            item.outputBytes = event.outputBytes;
            item.outputPath = event.outputPath;
            item.stageMessage = "已完成导出";
            saveQueueToStorage();
            await addHistoryRecord(event.outputPath, event.outputBytes);
          }
          queueTaskId.value = null;
          activeQueueItemId.value = null;
          if (isQueueRunning.value) {
            setTimeout(runQueueScheduler, 250);
          }
          return;
        }

        if (taskId.value && event.taskId !== taskId.value) {
          return;
        }
        taskId.value = event.taskId;
        status.value = "completed";
        progress.value = 100;
        outputBytes.value = event.outputBytes;
        outputPath.value = event.outputPath;
        const fmtLabel = exportFormat.value === "mp4" ? "MP4 微视频" : exportFormat.value === "avif" ? "AVIF" : exportFormat.value === "webp" ? "WebP" : "GIF";
        statusMessage.value = `${fmtLabel} 已导出，可以打开文件查看。`;
        await addHistoryRecord(event.outputPath, event.outputBytes);
      },
      onFailed: (event) => {
        // 队列任务事件
        if (queueTaskId.value && event.taskId === queueTaskId.value) {
          const item = queueItems.value.find((i) => i.id === activeQueueItemId.value);
          if (item) {
            item.status = "failed";
            item.stageMessage = "转码失败";
            item.errorMessage = readableRenderError(event.code, event.message);
            saveQueueToStorage();
          }
          queueTaskId.value = null;
          activeQueueItemId.value = null;
          if (isQueueRunning.value) {
            setTimeout(runQueueScheduler, 250);
          }
          return;
        }

        if (taskId.value && event.taskId !== taskId.value) {
          return;
        }
        taskId.value = event.taskId;
        status.value = "failed";
        statusMessage.value = "导出没有完成。";
        errorMessage.value = readableRenderError(event.code, event.message);
      },
      onCancelled: (event) => {
        // 队列任务事件
        if (queueTaskId.value && event.taskId === queueTaskId.value) {
          const item = queueItems.value.find((i) => i.id === activeQueueItemId.value);
          if (item) {
            item.status = "cancelled";
            item.stageMessage = "已取消";
            saveQueueToStorage();
          }
          queueTaskId.value = null;
          activeQueueItemId.value = null;
          if (isQueueRunning.value) {
            setTimeout(runQueueScheduler, 250);
          }
          return;
        }

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

// 队列数据持久化与调度逻辑
function saveQueueToStorage(): void {
  try {
    localStorage.setItem(QUEUE_STORAGE_KEY, serializeQueue(queueItems.value));
  } catch {
    // ignore
  }
}

function loadQueueFromStorage(): void {
  try {
    const raw = localStorage.getItem(QUEUE_STORAGE_KEY);
    if (!raw) return;
    queueItems.value = deserializeQueue(raw);
  } catch {
    // ignore
  }
}

function addToQueue(): void {
  if (!canRender.value) {
    showToast("请先选择视频并配置有效导出参数");
    return;
  }
  const plan = buildRenderPlan();
  const inputFileName = media.value?.path.split(/[\\/]/).pop() || "clip";
  const existingPaths = new Set(queueItems.value.map((i) => i.outputPath));
  const newItem = createQueueItem(plan, inputFileName, existingPaths);
  queueItems.value.push(newItem);
  saveQueueToStorage();
  showToast(`已加入导出队列 (第 ${queueItems.value.length} 个任务)`);
}

async function startBatchQueue(): Promise<void> {
  if (!isDesktop.value) {
    showToast("请在桌面模式下使用批量导出");
    return;
  }
  if (status.value === "rendering") {
    showToast("当前已有单个任务正在转码，请稍候");
    return;
  }
  if (!hasPendingQueueItems.value) {
    showToast("队列中没有待处理任务");
    return;
  }
  isQueueRunning.value = true;
  showToast(`开始批量转码，共 ${pendingQueueCount.value} 个待处理任务`);
  await runQueueScheduler();
}

async function pauseBatchQueue(): Promise<void> {
  isQueueRunning.value = false;
  showToast("已暂停队列批量转码");
  if (queueTaskId.value) {
    try {
      await cancelRender(queueTaskId.value);
    } catch {
      // ignore
    }
  }
}

async function runQueueScheduler(): Promise<void> {
  if (!isQueueRunning.value) {
    return;
  }
  if (activeQueueItemId.value) {
    return;
  }

  const nextItem = queueItems.value.find((i) => i.status === "pending");
  if (!nextItem) {
    isQueueRunning.value = false;
    activeQueueItemId.value = null;
    queueTaskId.value = null;
    showToast("🎉 批量导出任务已全部完成！");
    return;
  }

  nextItem.status = "rendering";
  nextItem.progress = 0;
  nextItem.stageMessage = "正在启动 FFmpeg…";
  nextItem.errorMessage = undefined;
  activeQueueItemId.value = nextItem.id;
  saveQueueToStorage();

  try {
    const tid = await renderGif(nextItem.plan);
    queueTaskId.value = tid;
  } catch (error) {
    nextItem.status = "failed";
    nextItem.stageMessage = "启动失败";
    nextItem.errorMessage = readableError(error);
    activeQueueItemId.value = null;
    queueTaskId.value = null;
    saveQueueToStorage();
    if (isQueueRunning.value) {
      setTimeout(runQueueScheduler, 300);
    }
  }
}

function retryQueueItem(id: string): void {
  const item = queueItems.value.find((i) => i.id === id);
  if (!item) return;
  item.status = "pending";
  item.progress = 0;
  item.errorMessage = undefined;
  item.stageMessage = "等待开始";
  saveQueueToStorage();
  showToast(`已将「${item.title}」重置为待处理`);
  if (isQueueRunning.value && !activeQueueItemId.value) {
    runQueueScheduler();
  }
}

async function removeQueueItem(id: string): Promise<void> {
  if (activeQueueItemId.value === id && queueTaskId.value) {
    try {
      await cancelRender(queueTaskId.value);
    } catch {
      // ignore
    }
    activeQueueItemId.value = null;
    queueTaskId.value = null;
  }
  queueItems.value = queueItems.value.filter((i) => i.id !== id);
  saveQueueToStorage();
  showToast("已从队列移除");
  if (isQueueRunning.value && !activeQueueItemId.value) {
    setTimeout(runQueueScheduler, 200);
  }
}

function clearCompletedQueue(): void {
  queueItems.value = queueItems.value.filter((i) => i.status !== "completed");
  saveQueueToStorage();
  showToast("已清空已完成队列项");
}

async function clearAllQueue(): Promise<void> {
  if (queueTaskId.value) {
    try {
      await cancelRender(queueTaskId.value);
    } catch {
      // ignore
    }
  }
  isQueueRunning.value = false;
  activeQueueItemId.value = null;
  queueTaskId.value = null;
  queueItems.value = [];
  saveQueueToStorage();
  showToast("已清空全部队列");
}

async function loadQueuePlan(item: QueueItem): Promise<void> {
  await loadHistoryPlan({
    id: item.id,
    fileName: item.outputPath.split(/[\\/]/).pop() || "clip",
    filePath: item.outputPath,
    outputBytes: item.outputBytes || 0,
    timestamp: item.createdAt,
    plan: item.plan,
  });
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
      rotation: Number(textRotation.value || 0),
      arcRadius: Number(textArcRadius.value || 0),
      isItalic: Boolean(textIsItalic.value),
      shadowOffset: Number(textShadowOffset.value || 0),
      shadowColor: textShadowColor.value || "#000000",
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
  const scaleFactor = getMediaScaleFactor();
  const scaledSize = Math.min(512, Math.max(12, Math.round(tpl.textStyle.size * scaleFactor)));
  const scaledStroke =
    tpl.textStyle.strokeWidth > 0 ? Math.max(1, Math.round(tpl.textStyle.strokeWidth * scaleFactor)) : 0;
  const rawShadow = tpl.textStyle.shadowOffset ?? 0;
  const scaledShadow = rawShadow > 0 ? Math.max(1, Math.round(rawShadow * scaleFactor)) : 0;

  textSize.value = scaledSize;
  textColor.value = tpl.textStyle.color;
  textStrokeWidth.value = scaledStroke;
  textStrokeColor.value = tpl.textStyle.strokeColor;
  textUseBox.value = tpl.textStyle.useBox;
  textBoxColor.value = tpl.textStyle.boxColor;
  textBoxOpacity.value = tpl.textStyle.boxOpacity;
  textRotation.value = tpl.textStyle.rotation ?? 0;
  textArcRadius.value = tpl.textStyle.arcRadius ?? 0;
  textIsItalic.value = Boolean(tpl.textStyle.isItalic);
  textShadowOffset.value = scaledShadow;
  textShadowColor.value = tpl.textStyle.shadowColor ?? "#000000";

  if (media.value) {
    const estTextW = Math.max(1, text.value.length) * scaledSize;
    const rawX = Math.round(media.value.width * tpl.textStyle.positionRatio.x - estTextW / 2);
    const rawY = Math.round(media.value.height * tpl.textStyle.positionRatio.y - scaledSize / 2);
    textX.value = Math.max(8, Math.min(Math.max(8, media.value.width - 20), rawX));
    textY.value = Math.max(8, Math.min(Math.max(8, media.value.height - scaledSize), rawY));
  }

  if (tpl.suggestedPresetId) {
    const p = PRESETS.find((item) => item.id === tpl.suggestedPresetId);
    if (p) applyPreset(p);
  }
  if (tpl.suggestedFormat) {
    setExportFormat(tpl.suggestedFormat);
  }

  activeInspectorTab.value = "text";
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
    includeAudio: includeAudio.value,
    hwAccel: hwAccelPolicy.value,
    videoFilter: currentVideoFilter.value,
    imageAdjustments: hasActiveAdjustments.value
      ? {
          brightness: Number(imageAdjustments.brightness),
          contrast: Number(imageAdjustments.contrast),
          saturation: Number(imageAdjustments.saturation),
          sharpen: Number(imageAdjustments.sharpen),
          blur: Number(imageAdjustments.blur),
          hue: Number(imageAdjustments.hue),
          vignette: Boolean(imageAdjustments.vignette),
        }
      : null,
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
    textRotation: Number(textRotation.value || 0),
    textArcRadius: Number(textArcRadius.value || 0),
    textIsItalic: Boolean(textIsItalic.value),
    textShadowOffset: Number(textShadowOffset.value || 0),
    textShadowColor: textShadowColor.value,
    textLayers: JSON.parse(JSON.stringify(textLayers.value)),
    activeLayerId: activeLayerId.value,
    useImage: useImage.value,
    imagePath: imagePath.value,
    imageX: Number(imageX.value),
    imageY: Number(imageY.value),
    imageWidth: Number(imageWidth.value),
    hasAudio: media.value?.hasAudio ?? true,
    bgmOverlay: bgmOverlay.value,
    sourceWidth: media.value?.width,
    sourceHeight: media.value?.height,
  };
}

function queueDraftSave(): void {
  if (isRestoringDraft) return;
  if (draftAutoSaveTimer) {
    clearTimeout(draftAutoSaveTimer);
  }
  draftAutoSaveTimer = setTimeout(() => {
    if (!inputPath.value && !text.value && !useCrop.value && !bgmPath.value && !hasActiveAdjustments.value) {
      return;
    }
    saveDraftToStorage(buildCurrentDraft());
  }, 400);
}

const hasActiveDraftOrMedia = computed(() => {
  return Boolean(
    media.value ||
      inputPath.value ||
      text.value ||
      useCrop.value ||
      useImage.value ||
      bgmPath.value ||
      hasActiveAdjustments.value
  );
});

function resetWorkspace(): void {
  if (draftAutoSaveTimer) {
    clearTimeout(draftAutoSaveTimer);
  }
  clearDraftFromStorage();
  media.value = null;
  inputPath.value = "";
  outputPath.value = "";
  setupVideoPreview("");
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
  includeAudio.value = true;
  hwAccelPolicy.value = "auto";
  currentVideoFilter.value = "none";
  resetImageAdjustments(true);
  bgmPath.value = "";
  bgmName.value = "";
  bgmDuration.value = 0;
  bgmVolume.value = 100;
  origAudioVolume.value = 100;
  if (sfxAudioPlayer) {
    sfxAudioPlayer.pause();
  }
  playingSfxId.value = null;
  isPlayingActiveBgm.value = false;
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
      rotation: 0,
      arcRadius: 0,
      isItalic: false,
      shadowOffset: 0,
      shadowColor: "#000000",
    },
  ];
  activeLayerId.value = "layer-1";
  useImage.value = false;
  imagePath.value = "";
  imagePreviewUrl.value = "";
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
        setupVideoPreview(draft.inputPath);
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
        if (draft.includeAudio !== undefined) {
          includeAudio.value = draft.includeAudio;
        }
        if (draft.hwAccel) {
          hwAccelPolicy.value = draft.hwAccel;
        }
        if (draft.bgmOverlay) {
          bgmPath.value = draft.bgmOverlay.path;
          const slashIdx = Math.max(draft.bgmOverlay.path.lastIndexOf("/"), draft.bgmOverlay.path.lastIndexOf("\\"));
          bgmName.value = slashIdx >= 0 ? draft.bgmOverlay.path.slice(slashIdx + 1) : draft.bgmOverlay.path;
          if (draft.bgmOverlay.volume !== undefined) {
            bgmVolume.value = Math.round(draft.bgmOverlay.volume * 100);
          }
          if (draft.bgmOverlay.origVolume !== undefined) {
            origAudioVolume.value = Math.round(draft.bgmOverlay.origVolume * 100);
          }
        }
        currentVideoFilter.value = (draft.videoFilter as VideoFilterKey) || "none";
        if (draft.imageAdjustments) {
          imageAdjustments.brightness = draft.imageAdjustments.brightness ?? 0;
          imageAdjustments.contrast = draft.imageAdjustments.contrast ?? 0;
          imageAdjustments.saturation = draft.imageAdjustments.saturation ?? 0;
          imageAdjustments.sharpen = draft.imageAdjustments.sharpen ?? 0;
          imageAdjustments.blur = draft.imageAdjustments.blur ?? 0;
          imageAdjustments.hue = draft.imageAdjustments.hue ?? 0;
          imageAdjustments.vignette = Boolean(draft.imageAdjustments.vignette);
        } else {
          resetImageAdjustments(true);
        }
        selectedPresetId.value = draft.selectedPresetId;
        useText.value = draft.useText;
        if (draft.textLayers && draft.textLayers.length > 0) {
          textLayers.value = draft.textLayers.map((l) => ({
            ...l,
            rotation: l.rotation ?? 0,
            arcRadius: l.arcRadius ?? 0,
            isItalic: Boolean(l.isItalic),
            shadowOffset: l.shadowOffset ?? 0,
            shadowColor: l.shadowColor ?? "#000000",
          }));
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
              rotation: draft.textRotation ?? 0,
              arcRadius: draft.textArcRadius ?? 0,
              isItalic: Boolean(draft.textIsItalic),
              shadowOffset: draft.textShadowOffset ?? 0,
              shadowColor: draft.textShadowColor ?? "#000000",
            },
          ];
          activeLayerId.value = "layer-1";
        }
        useImage.value = draft.useImage;
        imagePath.value = draft.imagePath;
        imageX.value = draft.imageX;
        imageY.value = draft.imageY;
        imageWidth.value = draft.imageWidth;
        if (draft.imagePath) {
          readFileAsDataUrl(draft.imagePath).then((url) => {
            if (url) imagePreviewUrl.value = url;
          });
        }
        status.value = "ready";
        statusMessage.value = "已自动还原上次编辑草稿。";
        showToast("已自动恢复上次编辑草稿");
      } catch {
        errorMessage.value = "上次编辑的视频文件已移动或无法访问，已恢复排版参数。";
        useText.value = draft.useText;
        if (draft.textLayers && draft.textLayers.length > 0) {
          textLayers.value = draft.textLayers.map((l) => ({
            ...l,
            rotation: l.rotation ?? 0,
            arcRadius: l.arcRadius ?? 0,
            isItalic: Boolean(l.isItalic),
            shadowOffset: l.shadowOffset ?? 0,
            shadowColor: l.shadowColor ?? "#000000",
          }));
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
              rotation: draft.textRotation ?? 0,
              arcRadius: draft.textArcRadius ?? 0,
              isItalic: Boolean(draft.textIsItalic),
              shadowOffset: draft.textShadowOffset ?? 0,
              shadowColor: draft.textShadowColor ?? "#000000",
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
    includeAudio,
    hwAccelPolicy,
    currentVideoFilter,
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
    bgmPath,
    bgmVolume,
    origAudioVolume,
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

watch(
  imageAdjustments,
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
    loadQueueFromStorage();
    builtinStickers.value = await getBuiltinStickers();
    builtinSfxList.value = await getBuiltinSfx();
    window.addEventListener("keydown", handleKeydown);
    desktopInfo.value = await getDesktopInfo();
    hwAccelInfo.value = await getHardwareAccelerationInfo();
    await connectDesktopEvents();
    await restoreDraftIfAvailable();
    // 启动后静默检查远程 GitHub Releases 更新
    handleCheckUpdate(true).catch(() => {});
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
  if (sfxAudioPlayer) {
    sfxAudioPlayer.pause();
    sfxAudioPlayer = null;
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

    <main id="workspace" class="studio-main">
      <!-- 1. 一体化极简顶栏 -->
      <header class="studio-topbar">
        <div class="topbar-left">
          <div class="brand-lockup">
            <div class="brand-mark">M</div>
            <strong>表情工坊</strong>
          </div>

          <span class="topbar-divider"></span>

          <div v-if="media" class="topbar-media-pill">
            <span class="media-pill-icon">🎬</span>
            <span class="media-pill-name" :title="media.fileName">{{ media.fileName }}</span>
            <span class="media-pill-meta">{{ media.width }}×{{ media.height }} · {{ mediaDurationLabel }} · {{ mediaFpsLabel }}</span>
            <button
              type="button"
              class="media-replace-btn"
              :disabled="isBusy"
              title="更换其他视频文件"
              @click="chooseVideo"
            >
              更换视频
            </button>
          </div>
          <button
            v-else
            type="button"
            class="topbar-import-btn"
            :disabled="isBusy"
            @click="chooseVideo"
          >
            <span>＋</span> 导入视频
          </button>
        </div>

        <div class="topbar-right">
          <div class="topbar-drawer-group" role="toolbar" aria-label="素材与任务抽屉">
            <button
              type="button"
              class="topbar-tool-btn"
              :class="{ active: isDrawerOpen && drawerTab === 'templates' }"
              title="打开表情包排版模板库"
              @click="toggleDrawerTab('templates')"
            >
              <span>📑 模板</span>
              <em v-if="allTemplatesCount > 0">{{ allTemplatesCount }}</em>
            </button>
            <button
              type="button"
              class="topbar-tool-btn"
              :class="{ active: isDrawerOpen && drawerTab === 'stickers' }"
              title="打开透明贴纸素材库"
              @click="toggleDrawerTab('stickers')"
            >
              <span>🎨 贴纸</span>
              <em v-if="builtinStickers.length > 0">{{ builtinStickers.length }}</em>
            </button>
            <button
              type="button"
              class="topbar-tool-btn"
              :class="{ active: isDrawerOpen && drawerTab === 'sfx' }"
              title="打开搞笑二创音效库"
              @click="toggleDrawerTab('sfx')"
            >
              <span>🎵 音效</span>
              <em v-if="builtinSfxList.length > 0">{{ builtinSfxList.length }}</em>
            </button>
            <button
              type="button"
              class="topbar-tool-btn"
              :class="{ active: isDrawerOpen && drawerTab === 'queue', 'has-running': isQueueRunning }"
              title="查看批量导出队列"
              @click="toggleDrawerTab('queue')"
            >
              <span v-if="isQueueRunning" class="queue-pulse-dot"></span>
              <span>📋 队列</span>
              <em v-if="queueItems.length > 0">{{ pendingQueueCount > 0 ? pendingQueueCount : queueItems.length }}</em>
            </button>
            <button
              type="button"
              class="topbar-tool-btn"
              :class="{ active: isDrawerOpen && drawerTab === 'history' }"
              title="查看最近导出历史"
              @click="toggleDrawerTab('history')"
            >
              <span>🕒 历史</span>
              <em v-if="historyList.length > 0">{{ historyList.length }}</em>
            </button>
          </div>

          <button
            v-if="hasActiveDraftOrMedia"
            type="button"
            class="topbar-reset-btn"
            title="清空当前草稿并重置所有编辑参数"
            @click="resetWorkspace"
          >
            ↺ 重置
          </button>

          <button
            type="button"
            class="runtime-pill version-update-btn"
            :class="{ 'has-update': hasNewVersion }"
            :title="hasNewVersion ? `发现新版本 v${updateInfo?.latestVersion}，点击查看更新` : `${runtimeLabel} · 点击检查 GitHub Releases 更新`"
            @click="hasNewVersion ? (isUpdateModalOpen = true) : handleCheckUpdate(false)"
          >
            <span class="runtime-dot" :class="{ offline: !isDesktop, 'update-dot': hasNewVersion }"></span>
            <span>v{{ appVersion }}</span>
            <em v-if="hasNewVersion" class="update-pill-tag">新版本</em>
            <small v-else-if="isCheckingUpdate" class="update-checking-text">检查中…</small>
          </button>
        </div>
      </header>

      <div class="page-content">
        <div v-if="!isDesktop" class="notice-banner">
          <span class="notice-icon">!</span>
          <div>需要桌面端 · 运行 <code>pnpm tauri:dev</code> 后才能导入本地文件。</div>
        </div>

        <!-- 2. 主工作区：左侧沉浸式画布与时间轴 + 右侧四分栏属性检查器 -->
        <section class="workspace-grid">
          <article class="preview-panel panel-card">
            <div class="stage-header-bar">
              <div class="stage-status-chips">
                <span v-if="media && isNativeVideoSupported" class="panel-chip chip-playback">
                  {{ isPreviewPlaying ? "▶ 循环播放中" : "可播放预览" }}
                </span>
                <span v-else-if="media" class="panel-chip chip-scrub">高清抽帧预览</span>
                <span v-else class="panel-chip">等待导入视频</span>

                <span v-if="media" class="panel-chip">
                  {{ useCrop ? `✂ 裁切 ${crop.width}×${crop.height}` : `${media.width}×${media.height}` }}
                </span>
                <span v-if="currentVideoFilter !== 'none'" class="panel-chip chip-filter">
                  {{ currentFilterOption.icon }} {{ currentFilterOption.label }}
                </span>
                <span v-if="hasActiveAdjustments" class="panel-chip chip-adjust" :title="activeAdjustmentsSummary">
                  ✨ {{ activeAdjustmentsSummary }}
                </span>
              </div>

              <!-- 画布快捷图层开关 -->
              <div class="stage-quick-toggles">
                <button
                  type="button"
                  class="stage-toggle-pill"
                  :class="{ 'is-on': useCrop }"
                  title="开启/关闭画面裁切框"
                  @click="useCrop = !useCrop; if (useCrop) activeInspectorTab = 'canvas'"
                >
                  ✂ 裁切
                </button>
                <button
                  type="button"
                  class="stage-toggle-pill"
                  :class="{ 'is-on': useText }"
                  title="开启/关闭文字字幕"
                  @click="useText = !useText; if (useText) activeInspectorTab = 'text'"
                >
                  🔤 文字
                </button>
                <button
                  type="button"
                  class="stage-toggle-pill"
                  :class="{ 'is-on': useImage }"
                  title="开启/关闭贴纸图案"
                  @click="useImage = !useImage; if (useImage) activeInspectorTab = 'media'"
                >
                  🎨 贴纸
                </button>
              </div>
            </div>

            <div ref="stageRef" class="preview-stage">
              <!-- 动态 SVG 锐化卷积矩阵滤镜，供画布实时预览调用 -->
              <svg width="0" height="0" style="position: absolute; pointer-events: none" aria-hidden="true">
                <defs>
                  <filter id="studio-sharpen-filter" color-interpolation-filters="sRGB">
                    <feConvolveMatrix order="3" :kernelMatrix="sharpenKernelMatrix" preserveAlpha="true" />
                  </filter>
                </defs>
              </svg>
              <template v-if="media">
                <div
                  class="preview-art"
                  :style="{
                    left: `${viewportRect.left}px`,
                    top: `${viewportRect.top}px`,
                    width: `${viewportRect.width}px`,
                    height: `${viewportRect.height}px`,
                  }"
                >
                  <!-- 视频真实画面层 (带全局滤镜与全帧画质精修) -->
                  <div class="preview-media-container" :style="{ filter: combinedMediaFilterCss }">
                    <video
                      v-if="isNativeVideoSupported && videoAssetUrl"
                      ref="previewVideoRef"
                      class="preview-video-element"
                      :src="videoAssetUrl"
                      playsinline
                      :muted="isPreviewMuted"
                      preload="auto"
                      @loadedmetadata="handleVideoLoadedMetadata"
                      @timeupdate="handleVideoTimeUpdate"
                      @error="handleVideoError"
                    ></video>
                    <img
                      v-else-if="extractedFrameUrl"
                      class="preview-frame-img"
                      :src="extractedFrameUrl"
                      alt="当前视频帧画面"
                      draggable="false"
                    />
                    <div v-else class="preview-bg-effects">
                      <div class="preview-grid"></div>
                      <div class="preview-orb orb-a"></div>
                      <div class="preview-orb orb-b"></div>
                    </div>
                  </div>
                  <div v-if="imageAdjustments.vignette" class="preview-vignette-overlay" aria-hidden="true"></div>
                  <template v-if="useText">
                    <template v-for="layer in textLayers" :key="layer.id">
                      <div
                        v-if="layer.visible && layer.text"
                        class="preview-text interactive-drag"
                        :class="{
                          'is-dragging': isDraggingText && activeLayerId === layer.id,
                          'is-active': activeLayerId === layer.id,
                          'with-box': layer.useBox,
                          'is-italic': layer.isItalic,
                        }"
                        :style="getTextLayerStyle(layer)"
                        :title="`图层 · 点击选中并拖拽`"
                        @pointerdown="startDragText(layer, $event)"
                      >
                        <span v-if="isDraggingText && activeLayerId === layer.id" class="drag-handle-pill">
                          X: {{ layer.x }} Y: {{ layer.y }}{{ layer.rotation ? ` · ${layer.rotation}°` : '' }}
                        </span>
                        <template v-if="layer.arcRadius && layer.arcRadius !== 0">
                          <span
                            v-for="(char, charIdx) in layer.text.split('')"
                            :key="charIdx"
                            class="arc-char-span"
                            :style="getArcCharStyle(layer, charIdx, layer.text.length)"
                          >
                            {{ char }}
                          </span>
                        </template>
                        <template v-else>
                          {{ layer.text }}
                        </template>
                      </div>
                    </template>
                  </template>
                  <div
                    v-if="useImage && imagePath"
                    class="preview-sticker interactive-drag"
                    :class="{ 'is-dragging': isDraggingImage, 'is-resizing': isResizingImage }"
                    :style="{
                      left: `${imageDisplayLeft}px`,
                      top: `${imageDisplayTop}px`,
                      width: `${imageDisplayWidth}px`,
                    }"
                    title="按住鼠标拖拽位置，按住右下角手柄缩放尺寸"
                    @pointerdown="startDragImage"
                  >
                    <span v-if="isDraggingImage || isResizingImage" class="drag-handle-pill">
                      X: {{ imageX }} Y: {{ imageY }} · W: {{ imageWidth }}px
                    </span>
                    <img
                      v-if="imagePreviewUrl"
                      :src="imagePreviewUrl"
                      class="sticker-canvas-img"
                      alt="贴纸图案"
                      draggable="false"
                    />
                    <span v-else class="sticker-fallback-text">PNG</span>
                    <div
                      class="sticker-resize-handle"
                      title="按住拖拽缩放尺寸"
                      @pointerdown.stop="startResizeImage"
                    >
                      ⤡
                    </div>
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
                </div>

                <!-- 真实可点击的播放控制浮层 -->
                <div
                  v-if="isNativeVideoSupported"
                  class="preview-play-control"
                  :class="{ 'is-playing': isPreviewPlaying }"
                  :title="isPreviewPlaying ? '暂停循环预览 (空格键)' : '开始循环预览所选片段 (空格键)'"
                  @click="togglePlayPreview"
                >
                  <span class="play-control-icon">{{ isPreviewPlaying ? "⏸" : "▶" }}</span>
                </div>
              </template>
              <div v-else class="preview-empty">
                <div class="empty-mark">✦</div>
                <strong>导入视频开始创作</strong>
                <span>支持拖拽画面裁切、添加多行字幕、透明贴纸与音效混流</span>
                <button
                  type="button"
                  class="empty-import-cta"
                  :disabled="isBusy"
                  @click="chooseVideo"
                >
                  ＋ 选择本地视频文件
                </button>
                <small class="empty-formats-hint">支持 MP4 · MOV · WEBM · MKV 等常见视频格式</small>
              </div>
            </div>

            <!-- 一体化播控与时间裁切栏 (合并播放按钮、精确秒数输入与时间轴滑块) -->
            <div v-if="media" class="unified-timeline-bar">
              <div class="timeline-controls-row">
                <div class="timeline-play-group">
                  <button
                    type="button"
                    class="preview-toolbar-btn"
                    :class="{ 'is-active': isPreviewPlaying }"
                    :disabled="!isNativeVideoSupported"
                    :title="isNativeVideoSupported ? '播放/暂停选定片段 (空格键)' : '当前格式仅支持抽帧预览'"
                    @click="togglePlayPreview"
                  >
                    {{ isPreviewPlaying ? "⏸ 暂停" : "▶ 播放" }}
                  </button>
                  <button
                    v-if="media.hasAudio && isNativeVideoSupported"
                    type="button"
                    class="preview-toolbar-btn btn-mute"
                    :class="{ 'is-muted': isPreviewMuted }"
                    :title="isPreviewMuted ? '点击开启原声音频试听' : '点击静音'"
                    @click="togglePreviewMute"
                  >
                    {{ isPreviewMuted ? "🔇 静音" : "🔊 原声" }}
                  </button>
                </div>

                <div class="timeline-inline-inputs">
                  <label class="inline-time-field" title="片段开始时间（秒）">
                    <span>起</span>
                    <input v-model.number="startSecs" type="number" min="0" step="0.1" />
                    <small>s</small>
                  </label>
                  <span class="timeline-sep">→</span>
                  <label class="inline-time-field" title="片段结束时间（秒）">
                    <span>止</span>
                    <input v-model.number="endSecs" type="number" min="0.1" step="0.1" :max="mediaDuration || undefined" />
                    <small>s</small>
                  </label>
                  <span class="timeline-total">/ {{ mediaDurationLabel }}</span>
                  <span class="timeline-badge">截取 {{ selectedDurationLabel }}</span>
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
          </article>

          <!-- 右侧：四分栏模块化属性检查器 -->
          <aside class="controls-panel panel-card">
            <nav class="inspector-tabs" aria-label="编辑属性分类">
              <button
                type="button"
                class="inspector-tab-btn"
                :class="{ active: activeInspectorTab === 'canvas' }"
                @click="activeInspectorTab = 'canvas'"
              >
                <span>📐 画面</span>
                <i v-if="useCrop || currentVideoFilter !== 'none' || hasActiveAdjustments" class="tab-active-dot"></i>
              </button>
              <button
                type="button"
                class="inspector-tab-btn"
                :class="{ active: activeInspectorTab === 'text' }"
                @click="activeInspectorTab = 'text'"
              >
                <span>🔤 文字</span>
                <i v-if="useText" class="tab-active-dot"></i>
              </button>
              <button
                type="button"
                class="inspector-tab-btn"
                :class="{ active: activeInspectorTab === 'media' }"
                @click="activeInspectorTab = 'media'"
              >
                <span>🎨 贴纸·音效</span>
                <i v-if="(useImage && imagePath) || bgmPath" class="tab-active-dot"></i>
              </button>
              <button
                type="button"
                class="inspector-tab-btn"
                :class="{ active: activeInspectorTab === 'export' }"
                @click="activeInspectorTab = 'export'"
              >
                <span>🚀 导出</span>
                <small class="tab-format-tag">{{ exportFormat.toUpperCase() }}</small>
              </button>
            </nav>

            <div class="inspector-body">
              <!-- ==================== Tab 1: 画面 (构图裁切 & 滤镜调色) ==================== -->
              <div v-if="activeInspectorTab === 'canvas'" class="inspector-pane">
                <div class="inspector-card">
                  <div class="inspector-card-header">
                    <label class="toggle-row">
                      <input v-model="useCrop" type="checkbox" />
                      <span class="toggle-ui"></span>
                      <strong>画面区域裁切</strong>
                    </label>
                    <span v-if="useCrop" class="card-status-pill">{{ crop.width }} × {{ crop.height }}</span>
                  </div>

                  <template v-if="useCrop">
                    <p class="inspector-hint">可在左侧画布直接拖拽九宫格选框，或选用常用比例：</p>
                    <div class="crop-ratio-grid">
                      <button type="button" class="crop-mini-btn" @click="cropSquare">1:1 方形</button>
                      <button type="button" class="crop-mini-btn" @click="crop4x3">4:3 经典</button>
                      <button type="button" class="crop-mini-btn" @click="crop16x9">16:9 横屏</button>
                      <button type="button" class="crop-mini-btn" @click="crop9x16">9:16 竖屏</button>
                      <button type="button" class="crop-mini-btn" @click="cropCenter">居中对齐</button>
                      <button type="button" class="crop-mini-btn" @click="cropReset">铺满原画</button>
                    </div>

                    <button
                      type="button"
                      class="collapsible-trigger"
                      @click="showExactCropCoords = !showExactCropCoords"
                    >
                      <span>📍 精确像素坐标 (X / Y / 宽 / 高)</span>
                      <span class="chevron">{{ showExactCropCoords ? "▾" : "▸" }}</span>
                    </button>
                    <div v-if="showExactCropCoords" class="field-grid four-up collapsible-content">
                      <label class="field"><span>X</span><input v-model.number="crop.x" type="number" min="0" step="1" /></label>
                      <label class="field"><span>Y</span><input v-model.number="crop.y" type="number" min="0" step="1" /></label>
                      <label class="field"><span>宽 W</span><input v-model.number="crop.width" type="number" min="20" step="1" /></label>
                      <label class="field"><span>高 H</span><input v-model.number="crop.height" type="number" min="20" step="1" /></label>
                    </div>
                  </template>
                  <p v-else class="inspector-empty-note">保持源视频完整构图，开启后可自由框选局部画面。</p>
                </div>

                <div class="inspector-card">
                  <div class="inspector-card-header">
                    <strong>全局画风与变速</strong>
                    <span class="filter-current-tag" :class="currentVideoFilter">
                      {{ currentFilterOption.icon }} {{ currentFilterOption.label }} · {{ currentFilterOption.badge }}
                    </span>
                  </div>
                  <div class="filter-pills-grid">
                    <button
                      v-for="f in VIDEO_FILTERS"
                      :key="f.key"
                      type="button"
                      class="video-filter-pill"
                      :class="{ active: currentVideoFilter === f.key, [`filter-${f.key}`]: true }"
                      :title="`${f.label} (${f.badge})`"
                      @click="setVideoFilter(f.key)"
                    >
                      <span class="filter-pill-icon">{{ f.icon }}</span>
                      <span class="filter-pill-label">{{ f.label }}</span>
                    </button>
                  </div>
                </div>

                <!-- 全帧画质与色彩精修卡片 (亮度/对比度/饱和度/锐化/模糊/色相/暗角) -->
                <div class="inspector-card">
                  <div class="inspector-card-header">
                    <strong>✨ 全帧画质与色彩精修</strong>
                    <div class="adjust-header-actions">
                      <span
                        class="card-status-pill"
                        :class="{ 'is-active': hasActiveAdjustments }"
                        :title="activeAdjustmentsSummary"
                      >
                        {{ activeAdjustmentsSummary }}
                      </span>
                      <button
                        v-if="hasActiveAdjustments"
                        type="button"
                        class="mini-reset-btn"
                        title="重置全部画质与色彩参数"
                        @click="resetImageAdjustments()"
                      >
                        ↺ 重置
                      </button>
                    </div>
                  </div>

                  <p class="inspector-hint">一键套用画质增强预设，或拖动滑块逐项微调（双击滑块可单项归零）：</p>
                  <div class="adjust-preset-row">
                    <button
                      v-for="preset in ADJUSTMENT_PRESETS"
                      :key="preset.id"
                      type="button"
                      class="adjust-preset-pill"
                      :title="`一键应用：${preset.label}`"
                      @click="applyAdjustmentPreset(preset)"
                    >
                      <span>{{ preset.icon }}</span>
                      <span>{{ preset.label }}</span>
                    </button>
                  </div>

                  <div class="adjust-sliders-grid">
                    <!-- 亮度 -->
                    <div class="adjust-slider-item">
                      <div class="adjust-slider-label">
                        <span>☀️ 亮度</span>
                        <em :class="{ 'is-modified': imageAdjustments.brightness !== 0 }">
                          {{ imageAdjustments.brightness > 0 ? `+${imageAdjustments.brightness}` : imageAdjustments.brightness }}
                        </em>
                      </div>
                      <input
                        v-model.number="imageAdjustments.brightness"
                        type="range"
                        min="-100"
                        max="100"
                        step="1"
                        class="effect-range-slider"
                        title="双击归零"
                        @dblclick="imageAdjustments.brightness = 0"
                      />
                    </div>

                    <!-- 对比度 -->
                    <div class="adjust-slider-item">
                      <div class="adjust-slider-label">
                        <span>◐ 对比度</span>
                        <em :class="{ 'is-modified': imageAdjustments.contrast !== 0 }">
                          {{ imageAdjustments.contrast > 0 ? `+${imageAdjustments.contrast}` : imageAdjustments.contrast }}
                        </em>
                      </div>
                      <input
                        v-model.number="imageAdjustments.contrast"
                        type="range"
                        min="-100"
                        max="100"
                        step="1"
                        class="effect-range-slider"
                        title="双击归零"
                        @dblclick="imageAdjustments.contrast = 0"
                      />
                    </div>

                    <!-- 饱和度 -->
                    <div class="adjust-slider-item">
                      <div class="adjust-slider-label">
                        <span>🎨 饱和度</span>
                        <em :class="{ 'is-modified': imageAdjustments.saturation !== 0 }">
                          {{ imageAdjustments.saturation > 0 ? `+${imageAdjustments.saturation}` : imageAdjustments.saturation }}
                        </em>
                      </div>
                      <input
                        v-model.number="imageAdjustments.saturation"
                        type="range"
                        min="-100"
                        max="100"
                        step="1"
                        class="effect-range-slider"
                        title="双击归零"
                        @dblclick="imageAdjustments.saturation = 0"
                      />
                    </div>

                    <!-- 画面锐化 -->
                    <div class="adjust-slider-item">
                      <div class="adjust-slider-label">
                        <span>💎 画面锐化</span>
                        <em :class="{ 'is-modified': imageAdjustments.sharpen > 0 }">
                          {{ imageAdjustments.sharpen }}%
                        </em>
                      </div>
                      <input
                        v-model.number="imageAdjustments.sharpen"
                        type="range"
                        min="0"
                        max="100"
                        step="1"
                        class="effect-range-slider"
                        title="双击归零 (FFmpeg unsharp 5×5 矩阵锐化)"
                        @dblclick="imageAdjustments.sharpen = 0"
                      />
                    </div>

                    <!-- 高斯模糊 -->
                    <div class="adjust-slider-item">
                      <div class="adjust-slider-label">
                        <span>🌫️ 高斯模糊</span>
                        <em :class="{ 'is-modified': imageAdjustments.blur > 0 }">
                          {{ imageAdjustments.blur }}%
                        </em>
                      </div>
                      <input
                        v-model.number="imageAdjustments.blur"
                        type="range"
                        min="0"
                        max="100"
                        step="1"
                        class="effect-range-slider"
                        title="双击归零 (FFmpeg gblur 柔焦模糊)"
                        @dblclick="imageAdjustments.blur = 0"
                      />
                    </div>

                    <!-- 色相偏移 -->
                    <div class="adjust-slider-item">
                      <div class="adjust-slider-label">
                        <span>🌈 色相偏移</span>
                        <em :class="{ 'is-modified': imageAdjustments.hue !== 0 }">
                          {{ imageAdjustments.hue > 0 ? `+${imageAdjustments.hue}°` : `${imageAdjustments.hue}°` }}
                        </em>
                      </div>
                      <input
                        v-model.number="imageAdjustments.hue"
                        type="range"
                        min="-180"
                        max="180"
                        step="5"
                        class="effect-range-slider"
                        title="双击归零 (FFmpeg hue 色相旋转)"
                        @dblclick="imageAdjustments.hue = 0"
                      />
                    </div>
                  </div>

                  <!-- 电影感暗角开关 -->
                  <div class="vignette-toggle-bar">
                    <label class="mini-toggle-row">
                      <input v-model="imageAdjustments.vignette" type="checkbox" />
                      <span class="toggle-ui mini-ui"></span>
                      <span>🎬 电影感光学暗角 (Vignette 四周柔化聚光)</span>
                    </label>
                  </div>
                </div>
              </div>

              <!-- ==================== Tab 2: 文字 (多图层字幕 & 模板排版) ==================== -->
              <div v-else-if="activeInspectorTab === 'text'" class="inspector-pane">
                <div class="inspector-card">
                  <div class="inspector-card-header">
                    <label class="toggle-row">
                      <input v-model="useText" type="checkbox" />
                      <span class="toggle-ui"></span>
                      <strong>文字字幕叠加</strong>
                    </label>
                    <div class="template-quick-actions">
                      <button type="button" class="template-link-btn" title="浏览表情包排版模板库" @click="openTemplatesDrawer">
                        📑 模板库
                      </button>
                      <button type="button" class="template-link-btn" title="将当前文字样式存为模板" @click="openSaveTemplateModal">
                        ＋ 存为模板
                      </button>
                    </div>
                  </div>

                  <p v-if="!useText" class="inspector-empty-note" style="margin-bottom: 10px">
                    💡 输入或修改下方台词将自动开启画布字幕，也可点击右上角「模板库」一键套用经典字幕。
                  </p>

                  <div class="text-editor-stack">
                    <!-- 多图层切换栏 -->
                    <div class="layer-tabs-bar">
                      <div class="layer-tabs-list">
                        <button
                          v-for="(layer, index) in textLayers"
                          :key="layer.id"
                          type="button"
                          class="layer-tab-btn"
                          :class="{ 'is-active': activeLayerId === layer.id, 'is-hidden': !layer.visible }"
                          @click="activeLayerId = layer.id; useText = true"
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
                        ＋ 图层
                      </button>
                    </div>

                    <!-- 台词内容输入 -->
                    <label class="field">
                      <span>台词内容（可在画布直接拖拽位置）</span>
                      <input v-model="text" type="text" maxlength="80" placeholder="输入表情包台词…" @focus="useText = true" />
                    </label>

                    <!-- 核心外观：字号、文字色、描边宽、描边色 -->
                    <div class="field-grid four-up">
                      <label class="field"><span>字号</span><input v-model.number="textSize" type="number" min="8" max="512" step="1" /></label>
                      <label class="field color-field"><span>文字色</span><input v-model="textColor" type="color" /></label>
                      <label class="field"><span>描边宽</span><input v-model.number="textStrokeWidth" type="number" min="0" max="24" step="1" /></label>
                      <label class="field color-field"><span>描边色</span><input v-model="textStrokeColor" type="color" /></label>
                    </div>

                    <!-- 字幕背景底框 -->
                    <div class="box-style-bar">
                      <label class="mini-toggle-row">
                        <input v-model="textUseBox" type="checkbox" />
                        <span class="toggle-ui mini-ui"></span>
                        <span>字幕背景底框</span>
                      </label>
                      <template v-if="textUseBox">
                        <input v-model="textBoxColor" type="color" class="inline-color-input" title="底框颜色" />
                        <input v-model.number="textBoxOpacity" type="range" min="0.1" max="1.0" step="0.05" class="opacity-slider" title="底框不透明度" />
                        <span class="opacity-value">{{ Math.round(textBoxOpacity * 100) }}%</span>
                      </template>
                    </div>

                    <!-- 折叠面板 1：高级变形与立体阴影 -->
                    <button
                      type="button"
                      class="collapsible-trigger"
                      :class="{ 'has-active': hasAdvancedTextActive }"
                      @click="showAdvancedText = !showAdvancedText"
                    >
                      <span>✨ 高级变形与阴影 (旋转 / 弧形 / 斜体 / 阴影)</span>
                      <span class="trigger-right">
                        <em v-if="hasAdvancedTextActive" class="active-tag">已启用</em>
                        <span class="chevron">{{ showAdvancedText ? "▾" : "▸" }}</span>
                      </span>
                    </button>

                    <div v-if="showAdvancedText" class="advanced-effects-panel">
                      <div class="effect-control-row">
                        <div class="effect-label-row">
                          <span class="effect-title">自由旋转</span>
                          <span class="effect-value">{{ textRotation }}°</span>
                        </div>
                        <div class="rotation-slider-bar">
                          <input
                            v-model.number="textRotation"
                            type="range"
                            min="-180"
                            max="180"
                            step="1"
                            class="effect-range-slider"
                          />
                          <div class="rotation-preset-pills">
                            <button type="button" class="rot-pill" :class="{ active: textRotation === 0 }" @click="setRotationPreset(0)">0°</button>
                            <button type="button" class="rot-pill" :class="{ active: textRotation === -45 }" @click="setRotationPreset(-45)">-45°</button>
                            <button type="button" class="rot-pill" :class="{ active: textRotation === -15 }" @click="setRotationPreset(-15)">-15°</button>
                            <button type="button" class="rot-pill" :class="{ active: textRotation === 15 }" @click="setRotationPreset(15)">+15°</button>
                            <button type="button" class="rot-pill" :class="{ active: textRotation === 45 }" @click="setRotationPreset(45)">+45°</button>
                            <button type="button" class="rot-pill" :class="{ active: textRotation === 90 }" @click="setRotationPreset(90)">90°</button>
                          </div>
                        </div>
                      </div>

                      <div class="advanced-style-grid">
                        <div class="effect-control-col">
                          <div class="effect-label-row">
                            <span class="effect-title">弧形弯曲</span>
                            <span class="effect-value">
                              {{ textArcRadius === 0 ? "平直" : textArcRadius > 0 ? `上拱 ${textArcRadius}` : `下凹 ${textArcRadius}` }}
                            </span>
                          </div>
                          <div class="arc-slider-wrap">
                            <input
                              v-model.number="textArcRadius"
                              type="range"
                              min="-100"
                              max="100"
                              step="5"
                              class="effect-range-slider"
                              title="双击归零"
                              @dblclick="textArcRadius = 0"
                            />
                            <button v-if="textArcRadius !== 0" type="button" class="mini-reset-btn" @click="textArcRadius = 0">重置</button>
                          </div>
                        </div>

                        <div class="effect-control-col">
                          <div class="effect-label-row">
                            <span class="effect-title">斜体与阴影</span>
                          </div>
                          <div class="style-toggle-buttons">
                            <button
                              type="button"
                              class="italic-toggle-btn"
                              :class="{ 'is-active': textIsItalic }"
                              @click="textIsItalic = !textIsItalic"
                            >
                              <span class="italic-glyph">𝐼</span> 斜体
                            </button>
                            <div class="shadow-controls-inline">
                              <label class="field color-field shadow-color-field" title="阴影颜色">
                                <input v-model="textShadowColor" type="color" />
                              </label>
                              <label class="shadow-offset-label" title="阴影偏移距离">
                                <input
                                  v-model.number="textShadowOffset"
                                  type="number"
                                  min="0"
                                  max="40"
                                  step="1"
                                  class="shadow-offset-input"
                                  placeholder="0"
                                />
                                <small>px</small>
                              </label>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>

                    <!-- 折叠面板 2：精确坐标 -->
                    <button
                      type="button"
                      class="collapsible-trigger"
                      @click="showExactTextCoords = !showExactTextCoords"
                    >
                      <span>📍 精确位置坐标 (X: {{ textX }}, Y: {{ textY }})</span>
                      <span class="chevron">{{ showExactTextCoords ? "▾" : "▸" }}</span>
                    </button>
                    <div v-if="showExactTextCoords" class="field-grid two-up collapsible-content">
                      <label class="field"><span>横坐标 X</span><input v-model.number="textX" type="number" min="0" step="1" /></label>
                      <label class="field"><span>纵坐标 Y</span><input v-model.number="textY" type="number" min="0" step="1" /></label>
                    </div>
                  </div>
                </div>
              </div>

              <!-- ==================== Tab 3: 贴纸与音效 (素材叠加) ==================== -->
              <div v-else-if="activeInspectorTab === 'media'" class="inspector-pane">
                <!-- 贴纸卡片 -->
                <div class="inspector-card">
                  <div class="inspector-card-header">
                    <label class="toggle-row">
                      <input v-model="useImage" type="checkbox" />
                      <span class="toggle-ui"></span>
                      <strong>透明表情贴纸</strong>
                    </label>
                    <div class="sticker-header-actions">
                      <button class="inline-button sticker-lib-btn" type="button" @click="openStickersDrawer">🎨 全部贴纸</button>
                      <button class="inline-button" type="button" @click="chooseImage">{{ imagePath ? "换 PNG" : "本地 PNG" }}</button>
                    </div>
                  </div>

                  <div v-if="useImage && imagePath" class="sticker-editor-stack">
                    <div class="selected-file sticker-selected-chip">
                      <img v-if="imagePreviewUrl" :src="imagePreviewUrl" class="sticker-chip-icon" alt="贴纸" />
                      <span v-else class="sticker-chip-badge">PNG</span>
                      <strong class="sticker-chip-filename">{{ imagePath.split(/[\\/]/).pop() }}</strong>
                      <button type="button" aria-label="移除贴图" @click="clearImage">×</button>
                    </div>

                    <div class="sticker-scale-presets-bar">
                      <span class="scale-preset-label">尺寸 ({{ imageWidth }}px)</span>
                      <div class="scale-preset-buttons">
                        <button type="button" class="scale-preset-btn" @click="scaleImageTo(0.6)">60%</button>
                        <button type="button" class="scale-preset-btn" @click="scaleImageTo(1.0)">100%</button>
                        <button type="button" class="scale-preset-btn" @click="scaleImageTo(1.5)">150%</button>
                      </div>
                    </div>

                    <button
                      type="button"
                      class="collapsible-trigger"
                      @click="showExactStickerCoords = !showExactStickerCoords"
                    >
                      <span>📍 精确位置与宽度 (X: {{ imageX }}, Y: {{ imageY }})</span>
                      <span class="chevron">{{ showExactStickerCoords ? "▾" : "▸" }}</span>
                    </button>
                    <div v-if="showExactStickerCoords" class="field-grid three-up collapsible-content">
                      <label class="field"><span>X</span><input v-model.number="imageX" type="number" min="0" step="1" /></label>
                      <label class="field"><span>Y</span><input v-model.number="imageY" type="number" min="0" step="1" /></label>
                      <label class="field"><span>宽 W</span><input v-model.number="imageWidth" type="number" min="20" max="3840" step="1" /></label>
                    </div>
                  </div>
                  <p v-else class="inspector-empty-note">点击下方热门贴纸即可直接叠加到画面中央，或导入本地透明 PNG：</p>

                  <!-- 内嵌贴纸快捷点选网格 -->
                  <div v-if="builtinStickers.length > 0" class="inline-sticker-picker">
                    <button
                      v-for="st in builtinStickers.slice(0, 8)"
                      :key="st.id"
                      type="button"
                      class="inline-sticker-item"
                      :class="{ active: useImage && imagePath === st.path }"
                      :title="`点击叠加「${st.name}」`"
                      @click="applySticker(st)"
                    >
                      <img :src="st.dataUrl" :alt="st.name" />
                      <span>{{ st.name }}</span>
                    </button>
                  </div>
                </div>

                <!-- 声音与音效混流卡片 -->
                <div class="inspector-card">
                  <div class="inspector-card-header">
                    <strong>🎵 配乐与音效混流</strong>
                    <button
                      v-if="exportFormat !== 'mp4'"
                      type="button"
                      class="mp4-switch-pill"
                      title="声音需要 MP4 短视频格式承载，点击一键切换"
                      @click="setExportFormat('mp4')"
                    >
                      切换为 MP4 启用声音 ↗
                    </button>
                    <span v-else class="card-status-pill mp4-pill">MP4 声画同步</span>
                  </div>

                  <div class="audio-mixing-panel">
                    <div class="audio-switch-row">
                      <label class="audio-switch-label">
                        <input type="checkbox" v-model="includeAudio" class="audio-switch-checkbox" />
                        <span class="audio-switch-text">
                          <span class="audio-switch-title">🔊 保留原视频声音</span>
                        </span>
                      </label>
                      <span v-if="includeAudio" class="audio-vol-badge">{{ origAudioVolume }}%</span>
                    </div>

                    <div v-if="includeAudio" class="audio-slider-row">
                      <span class="audio-slider-label">原声音量</span>
                      <input
                        type="range"
                        v-model.number="origAudioVolume"
                        min="0"
                        max="200"
                        step="5"
                        class="audio-slider"
                      />
                    </div>

                    <div class="bgm-control-section">
                      <div class="bgm-header-row">
                        <span class="bgm-section-title">叠加搞笑音效 / BGM</span>
                        <div class="bgm-action-btns">
                          <button type="button" class="bgm-lib-btn" @click="openSfxDrawer">🎵 音效库</button>
                          <button type="button" class="bgm-pick-btn" @click="handlePickAudio">本地音频…</button>
                        </div>
                      </div>

                      <div v-if="bgmPath" class="bgm-active-card">
                        <div class="bgm-active-info">
                          <button
                            type="button"
                            class="bgm-play-btn"
                            :title="isPlayingActiveBgm ? '暂停试听' : '试听当前音效'"
                            @click="togglePlayActiveBgm"
                          >
                            {{ isPlayingActiveBgm ? '⏸' : '▶' }}
                          </button>
                          <div class="bgm-active-meta">
                            <span class="bgm-active-name" :title="bgmName">{{ bgmName }}</span>
                            <span v-if="bgmDuration > 0" class="bgm-active-dur">{{ bgmDuration }}s</span>
                          </div>
                        </div>
                        <button type="button" class="bgm-remove-btn" title="清除已选音效" @click="clearBgm">✕</button>
                      </div>

                      <div v-if="bgmPath" class="audio-slider-row">
                        <span class="audio-slider-label">音效音量</span>
                        <input
                          type="range"
                          v-model.number="bgmVolume"
                          min="0"
                          max="200"
                          step="5"
                          class="audio-slider"
                        />
                        <span class="audio-vol-badge">{{ bgmVolume }}%</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- ==================== Tab 4: 导出 (格式、预设、路径 & 硬件加速) ==================== -->
              <div v-else-if="activeInspectorTab === 'export'" class="inspector-pane">
                <!-- 输出格式 -->
                <div class="inspector-card">
                  <div class="inspector-card-header">
                    <strong>导出格式</strong>
                    <span class="format-hint">{{ exportFormat === 'mp4' ? '有声短视频' : exportFormat === 'avif' ? '超高压缩比' : exportFormat === 'webp' ? '高清小体积' : '全平台通用' }}</span>
                  </div>
                  <div class="format-pills">
                    <button
                      type="button"
                      class="format-pill"
                      :class="{ active: exportFormat === 'gif' }"
                      @click="setExportFormat('gif')"
                    >
                      <span class="format-pill-name">GIF</span>
                      <span class="format-pill-sub">经典通用</span>
                    </button>
                    <button
                      type="button"
                      class="format-pill"
                      :class="{ active: exportFormat === 'webp' }"
                      @click="setExportFormat('webp')"
                    >
                      <span class="format-pill-name">WebP</span>
                      <span class="format-pill-badge">小体积</span>
                    </button>
                    <button
                      type="button"
                      class="format-pill pill-avif"
                      :class="{ active: exportFormat === 'avif' }"
                      @click="setExportFormat('avif')"
                    >
                      <span class="format-pill-name">AVIF</span>
                      <span class="format-pill-badge badge-avif">超压缩</span>
                    </button>
                    <button
                      type="button"
                      class="format-pill pill-mp4"
                      :class="{ active: exportFormat === 'mp4' }"
                      @click="setExportFormat('mp4')"
                    >
                      <span class="format-pill-name">MP4</span>
                      <span class="format-pill-badge badge-mp4">带原声</span>
                    </button>
                  </div>
                </div>

                <!-- 尺寸与帧率预设 -->
                <div class="inspector-card">
                  <div class="inspector-card-header">
                    <strong>尺寸与帧率预设</strong>
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
                    <label class="field">
                      <span>输出宽度 · px</span>
                      <input v-model.number="outputWidth" type="number" min="32" max="1920" step="1" @input="onManualParameterChange" />
                    </label>
                    <label class="field">
                      <span>帧率 · FPS</span>
                      <input v-model.number="fps" type="number" min="1" max="60" step="1" @input="onManualParameterChange" />
                    </label>
                  </div>
                </div>

                <!-- 保存位置与硬件加速 -->
                <div class="inspector-card">
                  <div class="inspector-card-header">
                    <strong>保存路径与编码引擎</strong>
                    <small class="inspector-sub-hint">同名自动顺延序号</small>
                  </div>
                  <div class="output-path-row">
                    <input
                      v-model="outputPath"
                      type="text"
                      :placeholder="`${exportFormat.toUpperCase()} 输出保存路径`"
                    />
                    <button type="button" aria-label="选择输出路径" title="浏览保存位置" @click="chooseOutput">…</button>
                  </div>

                  <div class="hw-accel-row" :class="{ 'is-disabled': hwAccelPolicy === 'disabled' }">
                    <div class="hw-accel-info">
                      <span class="hw-accel-icon" :class="{ 'is-active': hwAccelPolicy === 'auto' && hwAccelInfo?.supported }">
                        {{ hwAccelPolicy === 'auto' && hwAccelInfo?.supported ? '⚡' : '⚙' }}
                      </span>
                      <div class="hw-accel-text">
                        <span class="hw-accel-title">
                          {{ hwAccelPolicy === 'auto' && hwAccelInfo?.supported ? `GPU 加速 · ${hwAccelInfo.engine}` : 'CPU 软件编码' }}
                        </span>
                        <small class="hw-accel-desc">
                          {{ hwAccelPolicy === 'auto' && hwAccelInfo?.supported ? hwAccelInfo.description : '纯软件逐帧运算，兼容性高' }}
                        </small>
                      </div>
                    </div>
                    <button
                      type="button"
                      class="hw-toggle-btn"
                      :class="{ 'is-on': hwAccelPolicy === 'auto' }"
                      @click="hwAccelPolicy = hwAccelPolicy === 'auto' ? 'disabled' : 'auto'"
                    >
                      {{ hwAccelPolicy === 'auto' ? 'ON' : 'OFF' }}
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </aside>
        </section>

        <!-- 3. 底部导出状态与主操作栏 -->
        <section class="export-bar panel-card">
          <div class="export-status">
            <div class="status-icon" :class="`status-${status}`">
              <span>{{ status === "completed" ? "✓" : status === "failed" ? "!" : "✦" }}</span>
            </div>
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
            <!-- 当前输出规格摘要标签（不论处于哪个 Tab 都一目了然） -->
            <span v-if="status === 'completed'" class="output-size">{{ formatBytes(outputBytes) }}</span>
            <button
              v-if="media"
              type="button"
              class="preset-summary-badge clickable-spec-badge"
              title="点击前往「导出」设置修改格式与分辨率"
              @click="activeInspectorTab = 'export'"
            >
              {{ exportFormat.toUpperCase() }} · {{ outputWidth }}px · {{ fps }}FPS
            </button>
            <span v-if="hwAccelPolicy === 'auto' && hwAccelInfo?.supported" class="hw-accel-badge" title="已启用 GPU 硬件加速">
              ⚡ {{ hwAccelInfo.engine }}
            </span>

            <template v-if="status === 'completed' && outputPath">
              <button
                class="secondary-action"
                type="button"
                title="在访达/资源管理器中显示此文件"
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
              class="secondary-action queue-add-btn"
              type="button"
              :disabled="!canRender || !isDesktop"
              title="将当前片段与排版加入批量导出队列"
              @click="addToQueue"
            >
              ＋ 加入队列
            </button>
            <button v-if="isBusy" class="secondary-action danger" type="button" @click="stopRender">取消</button>
            <button v-else class="primary-action" type="button" :disabled="!canRender || !isDesktop" @click="startRender">
              {{ status === "completed" ? "再次导出" : `导出 ${exportFormat.toUpperCase()}` }} <span>↗</span>
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
                  <button
                    type="button"
                    class="drawer-tab"
                    :class="{ active: drawerTab === 'stickers' }"
                    @click="drawerTab = 'stickers'"
                  >
                    <span>🎨 贴纸素材</span>
                    <span class="drawer-count-chip">{{ builtinStickers.length }}</span>
                  </button>
                  <button
                    type="button"
                    class="drawer-tab"
                    :class="{ active: drawerTab === 'sfx' }"
                    @click="drawerTab = 'sfx'"
                  >
                    <span>🎵 音效库</span>
                    <span class="drawer-count-chip">{{ builtinSfxList.length }}</span>
                  </button>
                  <button
                    type="button"
                    class="drawer-tab"
                    :class="{ active: drawerTab === 'queue' }"
                    @click="drawerTab = 'queue'"
                  >
                    <span v-if="isQueueRunning" class="queue-tab-indicator"></span>
                    <span>📋 导出队列</span>
                    <span class="drawer-count-chip" :class="{ 'chip-active': isQueueRunning }">{{ queueItems.length }}</span>
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
                <template v-else-if="drawerTab === 'templates'">
                  <button
                    type="button"
                    class="drawer-add-template-btn"
                    title="将当前工作区文字排版保存为新模板"
                    @click="openSaveTemplateModal"
                  >
                    ＋ 存为我的模板
                  </button>
                </template>
                <template v-else-if="drawerTab === 'stickers'">
                  <span class="drawer-stickers-tip">透明免抠图 · 点击一键置入画布</span>
                </template>
                <template v-else-if="drawerTab === 'sfx'">
                  <span class="drawer-stickers-tip">免版权搞笑二创音效 · 即选即用</span>
                </template>
                <template v-else-if="drawerTab === 'queue'">
                  <button
                    v-if="!isQueueRunning && hasPendingQueueItems"
                    type="button"
                    class="drawer-queue-start-btn"
                    title="依序串行转码所有排队任务"
                    @click="startBatchQueue"
                  >
                    ▶ 开始批量导出 ({{ pendingQueueCount }})
                  </button>
                  <button
                    v-else-if="isQueueRunning"
                    type="button"
                    class="drawer-queue-pause-btn"
                    title="暂停队列转码"
                    @click="pauseBatchQueue"
                  >
                    ⏸ 暂停队列
                  </button>
                  <button
                    v-if="completedQueueCount > 0"
                    type="button"
                    class="drawer-clear-btn"
                    @click="clearCompletedQueue"
                  >
                    清空已完成
                  </button>
                  <button
                    v-if="queueItems.length > 0"
                    type="button"
                    class="drawer-clear-btn danger-hover"
                    @click="clearAllQueue"
                  >
                    清空全部
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
                    <video
                      v-if="item.previewUrl && item.fileName.toLowerCase().endsWith('.mp4')"
                      :src="item.previewUrl"
                      autoplay
                      loop
                      muted
                      playsinline
                      class="history-thumb-video"
                    ></video>
                    <img v-else-if="item.previewUrl" :src="item.previewUrl" :alt="item.fileName" />
                    <div v-else class="card-thumb-fallback">
                      <span>{{ item.fileName.toLowerCase().endsWith('.mp4') ? 'MP4' : item.fileName.toLowerCase().endsWith('.avif') ? 'AVIF' : item.fileName.toLowerCase().endsWith('.webp') ? 'WEBP' : 'GIF' }}</span>
                    </div>
                    <span class="history-card-size-pill">{{ formatBytes(item.outputBytes) }}</span>
                  </div>

                  <div class="history-card-body">
                    <div class="history-card-title-row">
                      <strong class="history-card-filename" :title="item.fileName">{{ item.fileName }}</strong>
                      <span
                        class="history-format-badge"
                        :class="item.fileName.toLowerCase().endsWith('.mp4') ? 'badge-mp4' : item.fileName.toLowerCase().endsWith('.avif') ? 'badge-avif' : item.fileName.toLowerCase().endsWith('.webp') ? 'badge-webp' : 'badge-gif'"
                      >
                        {{ item.fileName.toLowerCase().endsWith('.mp4') ? 'MP4' : item.fileName.toLowerCase().endsWith('.avif') ? 'AVIF' : item.fileName.toLowerCase().endsWith('.webp') ? 'WEBP' : 'GIF' }}
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
            <div v-else-if="drawerTab === 'templates'" class="drawer-tab-content">
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

            <!-- Tab 3: 透明表情贴纸素材库 -->
            <div v-else-if="drawerTab === 'stickers'" class="drawer-tab-content stickers-tab-content">
              <div class="stickers-filter-bar">
                <div class="stickers-category-pills">
                  <button
                    type="button"
                    class="category-pill-btn"
                    :class="{ active: selectedStickerCategory === 'all' }"
                    @click="selectedStickerCategory = 'all'"
                  >
                    全部 ({{ builtinStickers.length }})
                  </button>
                  <button
                    type="button"
                    class="category-pill-btn"
                    :class="{ active: selectedStickerCategory === 'classic' }"
                    @click="selectedStickerCategory = 'classic'"
                  >
                    经典名梗 ({{ classicStickersCount }})
                  </button>
                  <button
                    type="button"
                    class="category-pill-btn"
                    :class="{ active: selectedStickerCategory === 'emotion' }"
                    @click="selectedStickerCategory = 'emotion'"
                  >
                    情绪表态 ({{ emotionStickersCount }})
                  </button>
                  <button
                    type="button"
                    class="category-pill-btn"
                    :class="{ active: selectedStickerCategory === 'bubble' }"
                    @click="selectedStickerCategory = 'bubble'"
                  >
                    对话气泡 ({{ bubbleStickersCount }})
                  </button>
                </div>
              </div>

              <div class="drawer-cards-strip stickers-strip">
                <div
                  v-for="stk in filteredStickers"
                  :key="stk.id"
                  class="sticker-card"
                  :class="{ 'is-active-sticker': useImage && imagePath === stk.path }"
                  title="点击将贴纸一键置入画面中央"
                  @click="applySticker(stk)"
                >
                  <div class="sticker-card-thumb-wrap checker-bg">
                    <img :src="stk.dataUrl" :alt="stk.name" class="sticker-card-img" draggable="false" />
                  </div>
                  <div class="sticker-card-meta">
                    <strong class="sticker-card-name">{{ stk.name }}</strong>
                    <span class="sticker-card-cat">{{ getCategoryLabel(stk.category) }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Tab 4: 导出任务队列 -->
            <div v-else-if="drawerTab === 'queue'" class="drawer-tab-content queue-tab-content">
              <div v-if="queueItems.length > 0" class="queue-summary-bar">
                <div class="queue-summary-stats">
                  <span>总计 <strong>{{ queueItems.length }}</strong> 个任务</span>
                  <span class="stat-pending">⏳ 待处理 {{ pendingQueueCount }}</span>
                  <span v-if="renderingQueueCount > 0" class="stat-rendering">⚙️ 转码中 {{ renderingQueueCount }}</span>
                  <span class="stat-completed">✓ 已完成 {{ completedQueueCount }}</span>
                  <span v-if="failedQueueCount > 0" class="stat-failed">✗ 失败 {{ failedQueueCount }}</span>
                </div>
                <div class="queue-summary-tip">串行调度 · 防止卡顿 · 防同名自动递增</div>
              </div>

              <div v-if="queueItems.length > 0" class="drawer-cards-strip queue-cards-strip">
                <div
                  v-for="item in queueItems"
                  :key="item.id"
                  class="drawer-card queue-card"
                  :class="[`card-status-${item.status}`, { 'is-active': item.id === activeQueueItemId }]"
                >
                  <div class="queue-card-top">
                    <div class="queue-card-title-row">
                      <strong class="queue-card-title" :title="item.title">{{ item.title }}</strong>
                      <div class="queue-badges">
                        <span class="format-badge" :class="item.format">{{ item.format.toUpperCase() }}</span>
                        <span class="queue-status-chip" :class="getQueueStatusBadge(item.status).badgeClass">
                          {{ getQueueStatusBadge(item.status).icon }} {{ getQueueStatusBadge(item.status).label }}
                        </span>
                      </div>
                    </div>
                    <div class="queue-card-sub">
                      <span class="queue-time-range">{{ item.timeRangeLabel }}</span>
                      <span v-if="item.outputBytes" class="queue-file-size">{{ formatBytes(item.outputBytes) }}</span>
                      <span v-else class="queue-dest-name">{{ item.outputPath.split(/[\\/]/).pop() }}</span>
                    </div>
                  </div>

                  <!-- 进度指示 -->
                  <div v-if="item.status === 'rendering' || item.status === 'completed'" class="queue-progress-track">
                    <div class="queue-progress-fill" :style="{ width: `${item.progress}%` }"></div>
                  </div>

                  <!-- 阶段信息 / 错误信息 -->
                  <div class="queue-card-msg-row">
                    <span class="queue-card-msg" :class="{ 'msg-error': item.status === 'failed' }">
                      {{ item.errorMessage || item.stageMessage || (item.status === 'pending' ? '排队等待转码' : '') }}
                    </span>
                    <span v-if="item.status === 'rendering'" class="queue-percent-text">{{ Math.round(item.progress) }}%</span>
                  </div>

                  <!-- 操作按钮栏 -->
                  <div class="queue-card-actions">
                    <template v-if="item.status === 'completed'">
                      <button
                        class="card-btn"
                        type="button"
                        title="在访达/资源管理器中查看"
                        @click="revealExportFile(item.outputPath)"
                      >
                        访达 ↗
                      </button>
                      <button
                        class="card-btn"
                        type="button"
                        title="复制文件到剪贴板"
                        @click="copyExportFile(item.outputPath)"
                      >
                        复制 📋
                      </button>
                    </template>
                    <template v-else-if="item.status === 'failed' || item.status === 'cancelled'">
                      <button
                        class="card-btn highlight-btn"
                        type="button"
                        title="重新加入待处理并执行"
                        @click="retryQueueItem(item.id)"
                      >
                        重试 ↻
                      </button>
                    </template>
                    <button
                      class="card-btn"
                      type="button"
                      title="将此任务的所有参数恢复到当前画布与面板"
                      @click="loadQueuePlan(item)"
                    >
                      恢复参数 ↺
                    </button>
                    <button
                      class="card-btn danger-btn"
                      type="button"
                      title="从队列中删除"
                      @click="removeQueueItem(item.id)"
                    >
                      删除 ✕
                    </button>
                  </div>
                </div>
              </div>

              <div v-else class="drawer-empty-state">
                <div class="drawer-empty-icon">📋</div>
                <div class="drawer-empty-title">导出队列为空</div>
                <div class="drawer-empty-desc">
                  调整好视频片段截取时间、字幕、贴纸后，点击底部「＋ 加入队列」即可将任务添加至此处，稍后一键串行批量导出！
                </div>
              </div>
            </div>

            <!-- Tab 5: 内置二创音效素材库 -->
            <div v-else-if="drawerTab === 'sfx'" class="drawer-tab-content sfx-tab-content">
              <div class="sfx-filter-bar">
                <div class="sfx-category-pills">
                  <button
                    type="button"
                    class="category-pill-btn"
                    :class="{ active: selectedSfxCategory === 'all' }"
                    @click="selectedSfxCategory = 'all'"
                  >
                    全部 ({{ builtinSfxList.length }})
                  </button>
                  <button
                    type="button"
                    class="category-pill-btn"
                    :class="{ active: selectedSfxCategory === 'reaction' }"
                    @click="selectedSfxCategory = 'reaction'"
                  >
                    经典反响 ({{ reactionSfxCount }})
                  </button>
                  <button
                    type="button"
                    class="category-pill-btn"
                    :class="{ active: selectedSfxCategory === 'drama' }"
                    @click="selectedSfxCategory = 'drama'"
                  >
                    高能戏剧 ({{ dramaSfxCount }})
                  </button>
                  <button
                    type="button"
                    class="category-pill-btn"
                    :class="{ active: selectedSfxCategory === 'funny' }"
                    @click="selectedSfxCategory = 'funny'"
                  >
                    滑稽搞怪 ({{ funnySfxCount }})
                  </button>
                </div>
              </div>

              <div class="sfx-cards-grid">
                <div
                  v-for="sfx in filteredSfxList"
                  :key="sfx.id"
                  class="sfx-card"
                  :class="{ 'is-selected': bgmPath === sfx.path }"
                >
                  <div class="sfx-card-top">
                    <span class="sfx-badge">{{ getSfxCategoryLabel(sfx.category) }}</span>
                    <span class="sfx-duration">{{ sfx.duration }}s</span>
                  </div>
                  <div class="sfx-card-body">
                    <button
                      type="button"
                      class="sfx-play-btn"
                      :title="playingSfxId === sfx.id ? '停止试听' : '试听音效'"
                      @click="togglePlaySfx(sfx)"
                    >
                      <span class="sfx-play-icon">{{ playingSfxId === sfx.id ? '⏸' : '▶' }}</span>
                    </button>
                    <span class="sfx-name">{{ sfx.name }}</span>
                  </div>
                  <div class="sfx-card-footer">
                    <button
                      type="button"
                      class="sfx-apply-btn"
                      :class="{ 'is-applied': bgmPath === sfx.path }"
                      @click="applySfx(sfx)"
                    >
                      {{ bgmPath === sfx.path ? '✓ 已选用' : '选用此音效' }}
                    </button>
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

    <!-- 版本与远程更新检查 Modal 对话框 -->
    <Transition name="modal-fade">
      <div v-if="isUpdateModalOpen" class="modal-backdrop" @click="isUpdateModalOpen = false">
        <div class="modal-box update-modal-box" @click.stop>
          <div class="modal-header">
            <strong>{{ hasNewVersion ? "🎉 发现新版本" : "📦 版本与更新检查" }}</strong>
            <button type="button" class="modal-close-btn" @click="isUpdateModalOpen = false">✕</button>
          </div>
          <div class="modal-body">
            <div class="update-version-compare">
              <div class="version-card">
                <span>当前版本</span>
                <strong>v{{ appVersion }}</strong>
                <small>{{ desktopInfo?.platform || "Desktop" }}</small>
              </div>
              <span class="version-arrow">→</span>
              <div class="version-card" :class="{ 'is-latest': hasNewVersion }">
                <span>GitHub 最新版本</span>
                <strong>v{{ updateInfo?.latestVersion || appVersion }}</strong>
                <small>{{ hasNewVersion ? "可立即更新" : "已是最新" }}</small>
              </div>
            </div>

            <template v-if="hasNewVersion && updateInfo">
              <div class="update-notes-box">
                <div class="update-notes-title">
                  <span>{{ updateInfo.releaseName }}</span>
                  <small v-if="updateInfo.assetName">{{ updateInfo.assetName }}</small>
                </div>
                <pre class="update-notes-content">{{ updateInfo.releaseNotes }}</pre>
              </div>
            </template>
            <p v-else class="modal-hint">
              当前安装的 <strong>表情工坊 v{{ appVersion }}</strong> 已是最新版本（内置免配置 FFmpeg / FFprobe 引擎）。你也可以前往 GitHub Releases 页面查看历史版本或下载 Windows / macOS 安装包。
            </p>
          </div>
          <div class="modal-footer">
            <button type="button" class="secondary-action" @click="handleOpenReleasesPage">
              查看 GitHub Releases
            </button>
            <button
              v-if="hasNewVersion"
              type="button"
              class="primary-action"
              @click="handleDownloadUpdate"
            >
              🚀 下载最新安装包
            </button>
            <button
              v-else
              type="button"
              class="primary-action"
              :disabled="isCheckingUpdate"
              @click="handleCheckUpdate(false)"
            >
              {{ isCheckingUpdate ? "正在检查…" : "🔄 重新检查" }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>
