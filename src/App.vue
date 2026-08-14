<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from "vue";
import {
  cancelRender,
  getDesktopInfo,
  isDesktopRuntime,
  listenForRenderEvents,
  pickGifOutput,
  pickPng,
  pickVideo,
  probeMedia,
  renderGif,
  type DesktopInfo,
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

const useText = ref(false);
const text = ref("Meme Studio");
const textX = ref(24);
const textY = ref(24);
const textSize = ref(42);
const textColor = ref("#ffffff");

const useImage = ref(false);
const imagePath = ref("");
const imageX = ref(24);
const imageY = ref(24);
const imageWidth = ref(120);

const isDesktop = computed(() => isDesktopRuntime());
const isBusy = computed(() => status.value === "probing" || status.value === "rendering");
const canRender = computed(
  () => Boolean(media.value && inputPath.value && outputPath.value && !isBusy.value),
);
const runtimeLabel = computed(() =>
  desktopInfo.value
    ? `${desktopInfo.value.platform} · Tauri ${desktopInfo.value.version}`
    : "浏览器预览 · 未连接桌面能力",
);
const mediaDuration = computed(() => media.value?.durationSecs ?? 0);
const mediaDurationLabel = computed(() =>
  media.value ? `${media.value.durationSecs.toFixed(2)}s` : "—",
);
const mediaFpsLabel = computed(() =>
  media.value?.fps ? `${media.value.fps.toFixed(2)} fps` : "未知 fps",
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

function applyMediaDefaults(nextMedia: MediaInfo): void {
  const suggestedEnd = Math.min(nextMedia.durationSecs, 3);
  endSecs.value = Number(suggestedEnd.toFixed(2));
  outputWidth.value = Math.min(480, nextMedia.width);
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
      onCompleted: (event) => {
        if (taskId.value && event.taskId !== taskId.value) {
          return;
        }
        taskId.value = event.taskId;
        status.value = "completed";
        progress.value = 100;
        outputBytes.value = event.outputBytes;
        outputPath.value = event.outputPath;
        statusMessage.value = "GIF 已导出，可以打开文件查看。";
      },
      onFailed: (event) => {
        if (taskId.value && event.taskId !== taskId.value) {
          return;
        }
        taskId.value = event.taskId;
        status.value = "failed";
        statusMessage.value = "导出没有完成。";
        errorMessage.value = event.message;
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

onMounted(async () => {
  try {
    desktopInfo.value = await getDesktopInfo();
    await connectDesktopEvents();
  } catch (error) {
    errorMessage.value = readableError(error);
  }
});

onBeforeUnmount(() => {
  stopListening.value?.();
});
</script>

<template>
  <div class="studio-shell">
    <aside class="studio-sidebar">
      <div class="brand-lockup">
        <div class="brand-mark">M</div>
        <div>
          <strong>Meme Studio</strong>
          <span>local creator workspace</span>
        </div>
      </div>

      <div class="sidebar-kicker">MVP / 01</div>
      <nav class="side-nav" aria-label="Workspace navigation">
        <a class="side-nav-item active" href="#workspace">
          <span class="nav-icon">✦</span>
          GIF maker
          <span class="nav-state">LIVE</span>
        </a>
        <a class="side-nav-item muted" href="#notes">
          <span class="nav-icon">◷</span>
          Export history
          <span class="nav-soon">soon</span>
        </a>
        <a class="side-nav-item muted" href="#notes">
          <span class="nav-icon">⌘</span>
          Shortcuts
        </a>
      </nav>

      <div class="sidebar-bottom">
        <div class="runtime-card">
          <span class="runtime-dot" :class="{ offline: !isDesktop }"></span>
          <div>
            <strong>{{ isDesktop ? "Desktop runtime" : "Preview runtime" }}</strong>
            <small>{{ runtimeLabel }}</small>
          </div>
        </div>
        <p class="sidebar-note">
          Your media stays on this machine. FFmpeg does the heavy lifting locally.
        </p>
      </div>
    </aside>

    <main id="workspace" class="studio-main">
      <header class="topbar">
        <div class="crumbs"><span>Workspace</span><b>/</b><strong>GIF maker</strong></div>
        <div class="topbar-meta"><span class="status-mark"></span> local-first · v0.1</div>
      </header>

      <div class="page-content">
        <section class="page-intro">
          <div>
            <p class="eyebrow">FFMPEG GIF MVP</p>
            <h1>Turn a moment into a loop.</h1>
            <p class="intro-copy">
              Bring in a video, shape the frame, add a little signal, and export a shareable GIF.
            </p>
          </div>
          <div class="runtime-badge"><span></span>{{ isDesktop ? "Ready to render" : "Desktop mode required" }}</div>
        </section>

        <div v-if="!isDesktop" class="notice-banner">
          <span class="notice-icon">!</span>
          <div>
            <strong>这是浏览器预览。</strong>
            <span>请运行 <code>pnpm tauri:dev</code>，才能选择本地文件并调用 FFmpeg。</span>
          </div>
        </div>

        <section class="workspace-grid">
          <article class="preview-panel panel-card">
            <div class="panel-heading">
              <div>
                <p class="eyebrow">Canvas / 01</p>
                <h2>Output preview</h2>
              </div>
              <span class="panel-chip">GIF · loop 0</span>
            </div>

            <div class="preview-stage" :style="previewStyle">
              <template v-if="media">
                <div class="preview-art">
                  <div class="preview-grid"></div>
                  <div class="preview-orb orb-a"></div>
                  <div class="preview-orb orb-b"></div>
                  <div v-if="useText && text" class="preview-text" :style="{ color: textColor, fontSize: `${Math.min(textSize, 54)}px` }">
                    {{ text }}
                  </div>
                  <div v-if="useImage && imagePath" class="preview-sticker" :style="{ left: `${Math.min(imageX / Math.max(media.width, 1) * 100, 80)}%`, top: `${Math.min(imageY / Math.max(media.height, 1) * 100, 80)}%`, width: `${Math.min(imageWidth / Math.max(media.width, 1) * 100, 42)}%` }">
                    PNG
                  </div>
                  <div class="preview-caption">SOURCE FRAME · {{ media.width }} × {{ media.height }}</div>
                </div>
                <div class="play-button" aria-hidden="true">▶</div>
              </template>
              <div v-else class="preview-empty">
                <div class="empty-mark">✦</div>
                <strong>Your canvas is waiting.</strong>
                <span>Choose a video to see its details here.</span>
              </div>
            </div>

            <div class="media-summary">
              <div class="media-file">
                <span class="file-icon">▧</span>
                <div>
                  <strong>{{ media?.fileName ?? "No video selected" }}</strong>
                  <small>{{ media ? `${media.format} · ${media.width} × ${media.height}` : "MP4, MOV, WEBM and more" }}</small>
                </div>
              </div>
              <div class="media-stats">
                <span><b>{{ mediaDurationLabel }}</b> duration</span>
                <span><b>{{ mediaFpsLabel }}</b></span>
              </div>
            </div>
          </article>

          <aside class="controls-panel panel-card">
            <div class="panel-heading compact-heading">
              <div>
                <p class="eyebrow">Edit / 02</p>
                <h2>Shape the loop</h2>
              </div>
              <span class="step-number">02</span>
            </div>

            <button class="file-picker" type="button" :disabled="isBusy" @click="chooseVideo">
              <span class="picker-icon">↑</span>
              <span>
                <strong>{{ media ? "Replace video" : "Choose a video" }}</strong>
                <small>{{ media ? "Pick another local source" : "Start with an MP4, MOV, or WEBM" }}</small>
              </span>
              <span class="picker-arrow">→</span>
            </button>

            <div class="control-section">
              <div class="section-label"><span>01</span> Timing</div>
              <div class="field-grid two-up">
                <label class="field"><span>Start · sec</span><input v-model.number="startSecs" type="number" min="0" step="0.1" /></label>
                <label class="field"><span>End · sec</span><input v-model.number="endSecs" type="number" min="0.1" step="0.1" :max="mediaDuration || undefined" /></label>
              </div>
            </div>

            <div class="control-section">
              <div class="section-label"><span>02</span> Frame & motion</div>
              <div class="field-grid two-up">
                <label class="field"><span>Width · px</span><input v-model.number="outputWidth" type="number" min="32" max="1920" step="1" /></label>
                <label class="field"><span>Frame rate</span><input v-model.number="fps" type="number" min="1" max="60" step="1" /></label>
              </div>
              <label class="toggle-row"><input v-model="useCrop" type="checkbox" /><span class="toggle-ui"></span><span>Use crop rectangle</span><em>optional</em></label>
              <div v-if="useCrop" class="field-grid four-up crop-grid">
                <label class="field"><span>X</span><input v-model.number="crop.x" type="number" min="0" step="1" /></label>
                <label class="field"><span>Y</span><input v-model.number="crop.y" type="number" min="0" step="1" /></label>
                <label class="field"><span>W</span><input v-model.number="crop.width" type="number" min="1" step="1" /></label>
                <label class="field"><span>H</span><input v-model.number="crop.height" type="number" min="1" step="1" /></label>
              </div>
            </div>

            <div class="control-section">
              <div class="section-label"><span>03</span> Overlays</div>
              <label class="toggle-row"><input v-model="useText" type="checkbox" /><span class="toggle-ui"></span><span>Text overlay</span><em>drawtext</em></label>
              <div v-if="useText" class="overlay-fields">
                <label class="field field-wide"><span>Message</span><input v-model="text" type="text" maxlength="80" placeholder="Add a caption" /></label>
                <div class="field-grid four-up">
                  <label class="field"><span>X</span><input v-model.number="textX" type="number" min="0" step="1" /></label>
                  <label class="field"><span>Y</span><input v-model.number="textY" type="number" min="0" step="1" /></label>
                  <label class="field"><span>Size</span><input v-model.number="textSize" type="number" min="8" max="192" step="1" /></label>
                  <label class="field color-field"><span>Color</span><input v-model="textColor" type="color" /></label>
                </div>
              </div>
              <div class="toggle-row image-row">
                <label class="toggle-label"><input v-model="useImage" type="checkbox" /><span class="toggle-ui"></span><span>PNG graphic</span></label>
                <button class="inline-button" type="button" @click="chooseImage">{{ imagePath ? "Replace" : "Choose PNG" }}</button>
              </div>
              <div v-if="useImage && imagePath" class="overlay-fields">
                <div class="selected-file"><span>PNG</span><strong>{{ imagePath.split(/[\\/]/).pop() }}</strong><button type="button" aria-label="Remove PNG" @click="clearImage">×</button></div>
                <div class="field-grid three-up">
                  <label class="field"><span>X</span><input v-model.number="imageX" type="number" min="0" step="1" /></label>
                  <label class="field"><span>Y</span><input v-model.number="imageY" type="number" min="0" step="1" /></label>
                  <label class="field"><span>Width</span><input v-model.number="imageWidth" type="number" min="1" max="1920" step="1" /></label>
                </div>
              </div>
            </div>

            <div class="control-section output-section">
              <div class="section-label"><span>04</span> Destination</div>
              <div class="output-path-row">
                <input v-model="outputPath" type="text" placeholder="Choose a .gif output path" />
                <button type="button" aria-label="Choose output path" @click="chooseOutput">…</button>
              </div>
            </div>
          </aside>
        </section>

        <section class="export-bar panel-card">
          <div class="export-status">
            <div class="status-icon" :class="`status-${status}`"><span>{{ status === "completed" ? "✓" : status === "failed" ? "!" : "✦" }}</span></div>
            <div>
              <p class="eyebrow">Export / 03</p>
              <strong>{{ statusLabel }}</strong>
              <span>{{ errorMessage || statusMessage }}</span>
            </div>
          </div>
          <div class="progress-wrap" :class="{ hidden: !isBusy && status !== 'completed' }">
            <div class="progress-track"><span :style="{ width: `${progress}%` }"></span></div>
            <small>{{ Math.round(progress) }}%</small>
          </div>
          <div class="export-actions">
            <span v-if="status === 'completed'" class="output-size">{{ (outputBytes / 1024).toFixed(1) }} KB</span>
            <button v-if="isBusy" class="secondary-action danger" type="button" @click="stopRender">Cancel</button>
            <button v-else class="primary-action" type="button" :disabled="!canRender || !isDesktop" @click="startRender">
              {{ status === "completed" ? "Export another" : "Export GIF" }} <span>↗</span>
            </button>
          </div>
        </section>

        <footer id="notes" class="page-footer">
          <span>Meme Studio / FFmpeg GIF MVP</span>
          <span>Local files stay local · palettegen + paletteuse</span>
        </footer>
      </div>
    </main>
  </div>
</template>
