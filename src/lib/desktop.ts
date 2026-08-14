import { invoke } from "@tauri-apps/api/core";

export interface DesktopInfo {
  version: string;
  platform: string;
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

export async function getDesktopInfo(): Promise<DesktopInfo | null> {
  if (!isDesktopRuntime()) {
    return null;
  }

  return invoke<DesktopInfo>("get_app_info");
}

export async function greetCreator(name: string): Promise<string> {
  if (!isDesktopRuntime()) {
    return "Welcome back, " + name + ".";
  }

  return invoke<string>("greet", { name });
}
