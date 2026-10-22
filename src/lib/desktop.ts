import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";

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
  text: string;
  x: number;
  y: number;
  size: number;
  color: string;
}

export interface ImageOverlay {
  path: string;
  x: number;
  y: number;
  width: number;
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
  imageOverlay: ImageOverlay | null;
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
  requireDesktopRuntime();

  return save({
    defaultPath,
    filters: [{ name: "GIF 动图", extensions: ["gif"] }],
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
