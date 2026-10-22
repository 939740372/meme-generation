import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import { revealItemInDir as openerRevealItemInDir } from "@tauri-apps/plugin-opener";

export interface DesktopInfo {
  version: string;
  platform: string;
}

export interface MediaInfo {
  path: string;
  fileName: string;
  format: string;
  durationSecs: number;
  width: number;
  height: number;
  fps: number;
}

export interface CropArea {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface TextOverlay {
  id?: string;
  text: string;
  x: number;
  y: number;
  size: number;
  color: string;
  strokeWidth?: number;
  strokeColor?: string;
  useBox?: boolean;
  boxColor?: string;
  boxOpacity?: number;
  visible?: boolean;
}

export interface ImageOverlay {
  path: string;
  x: number;
  y: number;
  width: number;
}

export interface BuiltinSticker {
  id: string;
  name: string;
  category: "classic" | "emotion" | "bubble";
  path: string;
  dataUrl: string;
  defaultWidth: number;
}

export type ExportFormat = "gif" | "webp" | "avif" | "mp4";

export type HwAccelPolicy = "auto" | "disabled";

export interface HardwareAccelerationInfo {
  supported: boolean;
  engine: string;
  encoder: string;
  description: string;
}

export interface RenderPlan {
  inputPath: string;
  outputPath: string;
  startSecs: number;
  endSecs: number;
  crop: CropArea | null;
  outputWidth: number;
  fps: number;
  quality?: "low" | "medium" | "high";
  textOverlay: TextOverlay | null;
  textLayers?: TextOverlay[] | null;
  imageOverlay: ImageOverlay | null;
  format?: ExportFormat;
  videoFilter?: string;
  includeAudio?: boolean;
  hwAccel?: HwAccelPolicy;
}

export interface ExportRecord {
  id: string;
  fileName: string;
  filePath: string;
  outputBytes: number;
  timestamp: number;
  previewUrl?: string;
  plan: RenderPlan;
}

export interface RenderProgress {
  taskId: string;
  stage: "starting" | "rendering" | "finalizing";
  progress: number;
  message: string;
}

export interface RenderCompleted {
  taskId: string;
  outputPath: string;
  outputBytes: number;
}

export interface RenderFailed {
  taskId: string;
  code: string;
  message: string;
}

export interface RenderCancelled {
  taskId: string;
  message: string;
}

interface TauriWindow extends Window {
  __TAURI_INTERNALS__?: unknown;
}

export function isDesktopRuntime(): boolean {
  if (typeof window === "undefined") {
    return false;
  }

  return Boolean((window as TauriWindow).__TAURI_INTERNALS__);
}

function requireDesktopRuntime(): void {
  if (!isDesktopRuntime()) {
    throw new Error("请使用 Tauri 桌面模式访问本地文件并导出 GIF。");
  }
}

export async function getDesktopInfo(): Promise<DesktopInfo | null> {
  if (!isDesktopRuntime()) {
    return null;
  }

  return invoke<DesktopInfo>("get_app_info");
}

export async function greetCreator(name: string): Promise<string> {
  if (!isDesktopRuntime()) {
    return "欢迎回来，" + name + "。";
  }

  return invoke<string>("greet", { name });
}

export async function pickVideo(): Promise<string | null> {
  requireDesktopRuntime();

  const selected = await open({
    multiple: false,
    directory: false,
    filters: [
      {
        name: "视频",
        extensions: ["mp4", "mov", "mkv", "webm", "avi", "m4v"],
      },
    ],
  });

  return typeof selected === "string" ? selected : null;
}

export async function pickPng(): Promise<string | null> {
  requireDesktopRuntime();

  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "PNG 图案", extensions: ["png"] }],
  });

  return typeof selected === "string" ? selected : null;
}

export async function pickGifOutput(defaultPath?: string): Promise<string | null> {
  return pickMediaOutput(defaultPath, "gif");
}

export async function pickMediaOutput(
  defaultPath?: string,
  format: ExportFormat = "gif",
): Promise<string | null> {
  requireDesktopRuntime();

  const filterMap: Record<ExportFormat, { name: string; extensions: string[] }> = {
    gif: { name: "GIF 经典动图", extensions: ["gif"] },
    webp: { name: "WebP 动图", extensions: ["webp"] },
    avif: { name: "AVIF 新一代动图", extensions: ["avif"] },
    mp4: { name: "MP4 视频表情", extensions: ["mp4"] },
  };

  const prioritized = [
    filterMap[format],
    ...Object.entries(filterMap)
      .filter(([k]) => k !== format)
      .map(([, v]) => v),
  ];

  return save({
    defaultPath,
    filters: prioritized,
  });
}

export function probeMedia(path: string): Promise<MediaInfo> {
  requireDesktopRuntime();
  return invoke<MediaInfo>("media_probe", { path });
}

export function renderGif(plan: RenderPlan): Promise<string> {
  requireDesktopRuntime();
  return invoke<string>("render_gif", { plan });
}

export function cancelRender(taskId: string): Promise<void> {
  requireDesktopRuntime();
  return invoke("cancel_render", { taskId });
}

export async function listenForRenderEvents(
  handlers: {
    onProgress: (event: RenderProgress) => void;
    onCompleted: (event: RenderCompleted) => void;
    onFailed: (event: RenderFailed) => void;
    onCancelled: (event: RenderCancelled) => void;
  },
): Promise<() => void> {
  requireDesktopRuntime();

  const unlisteners: UnlistenFn[] = await Promise.all([
    listen<RenderProgress>("render_progress", (event) =>
      handlers.onProgress(event.payload),
    ),
    listen<RenderCompleted>("render_completed", (event) =>
      handlers.onCompleted(event.payload),
    ),
    listen<RenderFailed>("render_failed", (event) =>
      handlers.onFailed(event.payload),
    ),
    listen<RenderCancelled>("render_cancelled", (event) =>
      handlers.onCancelled(event.payload),
    ),
  ]);

  return () => {
    unlisteners.forEach((unlisten) => unlisten());
  };
}

export async function revealFileInDir(path: string): Promise<void> {
  requireDesktopRuntime();
  await openerRevealItemInDir(path);
}

export async function copyFileToClipboard(path: string): Promise<boolean> {
  let fileCopied = false;
  if (isDesktopRuntime()) {
    try {
      await invoke("copy_file_to_clipboard", { path });
      fileCopied = true;
    } catch {
      // ignore
    }
  }
  try {
    if (typeof navigator !== "undefined" && navigator.clipboard) {
      await navigator.clipboard.writeText(path);
    }
  } catch {
    // ignore
  }
  return fileCopied;
}

export async function readFileAsDataUrl(path: string): Promise<string | null> {
  if (!isDesktopRuntime()) {
    return null;
  }
  try {
    return await invoke<string>("read_file_as_data_url", { path });
  } catch {
    return null;
  }
}

export async function getBuiltinStickers(): Promise<BuiltinSticker[]> {
  if (!isDesktopRuntime()) {
    return [];
  }
  try {
    return await invoke<BuiltinSticker[]>("get_builtin_stickers");
  } catch {
    return [];
  }
}

export async function getHardwareAccelerationInfo(): Promise<HardwareAccelerationInfo> {
  if (!isDesktopRuntime()) {
    return {
      supported: false,
      engine: "None",
      encoder: "libx264",
      description: "非桌面环境，使用软件压制",
    };
  }
  try {
    return await invoke<HardwareAccelerationInfo>("get_hardware_acceleration_info");
  } catch {
    return {
      supported: false,
      engine: "None",
      encoder: "libx264",
      description: "探测失败，自动使用 CPU 软件压制",
    };
  }
}
