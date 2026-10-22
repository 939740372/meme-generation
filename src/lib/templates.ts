export interface TextStyleConfig {
  defaultText?: string;
  size: number;
  color: string;
  strokeWidth: number;
  strokeColor: string;
  useBox: boolean;
  boxColor: string;
  boxOpacity: number;
  positionRatio: { x: number; y: number }; // 0.0 ~ 1.0 水平/垂直相对比例
  rotation?: number;
  arcRadius?: number;
  isItalic?: boolean;
  shadowOffset?: number;
  shadowColor?: string;
}

export interface MemeTemplate {
  id: string;
  name: string;
  category: "builtin" | "custom";
  description: string;
  badge: string;
  textStyle: TextStyleConfig;
  suggestedPresetId?: string;
  suggestedFormat?: "gif" | "webp" | "avif" | "mp4";
  createdAt?: number;
}

export interface CanvasTextLayer {
  id: string;
  text: string;
  x: number;
  y: number;
  size: number;
  color: string;
  strokeWidth: number;
  strokeColor: string;
  useBox: boolean;
  boxColor: string;
  boxOpacity: number;
  visible: boolean;
  rotation?: number;
  arcRadius?: number;
  isItalic?: boolean;
  shadowOffset?: number;
  shadowColor?: string;
}

export interface StudioDraft {
  version: 1;
  updatedAt: number;
  inputPath: string;
  outputPath: string;
  startSecs: number;
  endSecs: number;
  useCrop: boolean;
  crop: { x: number; y: number; width: number; height: number };
  outputWidth: number;
  fps: number;
  quality: "low" | "medium" | "high";
  format: "gif" | "webp" | "avif" | "mp4";
  videoFilter?: string;
  includeAudio?: boolean;
  hwAccel?: "auto" | "disabled";
  selectedPresetId: string;
  useText: boolean;
  text: string;
  textX: number;
  textY: number;
  textSize: number;
  textColor: string;
  textStrokeWidth: number;
  textStrokeColor: string;
  textUseBox: boolean;
  textBoxColor: string;
  textBoxOpacity: number;
  textRotation?: number;
  textArcRadius?: number;
  textIsItalic?: boolean;
  textShadowOffset?: number;
  textShadowColor?: string;
  textLayers?: CanvasTextLayer[];
  activeLayerId?: string;
  useImage: boolean;
  imagePath: string;
  imageX: number;
  imageY: number;
  imageWidth: number;
  hasAudio?: boolean;
  bgmOverlay?: {
    path: string;
    volume?: number;
    origVolume?: number;
  } | null;
  sourceWidth?: number;
  sourceHeight?: number;
}

export const BUILTIN_TEMPLATES: MemeTemplate[] = [
  {
    id: "builtin-subtitles-bar",
    name: "经典影视字幕条",
    category: "builtin",
    description: "居中白字 + 黑色半透明底条，院线名场面台词质感",
    badge: "影视字幕",
    textStyle: {
      defaultText: "你不要过来啊！",
      size: 26,
      color: "#ffffff",
      strokeWidth: 2,
      strokeColor: "#000000",
      useBox: true,
      boxColor: "#000000",
      boxOpacity: 0.65,
      positionRatio: { x: 0.5, y: 0.88 },
    },
    suggestedPresetId: "large",
  },
  {
    id: "builtin-top-banner",
    name: "顶部大字吐槽",
    category: "builtin",
    description: "醒目黄色大字 + 纯黑描边，搞笑直观神吐槽",
    badge: "吐槽带",
    textStyle: {
      defaultText: "此时一位网友路过",
      size: 32,
      color: "#ffe600",
      strokeWidth: 3,
      strokeColor: "#000000",
      useBox: false,
      boxColor: "#000000",
      boxOpacity: 0.6,
      positionRatio: { x: 0.5, y: 0.12 },
    },
    suggestedPresetId: "large",
  },
  {
    id: "builtin-impact-headline",
    name: "名场面震撼大字报",
    category: "builtin",
    description: "超粗黑边大号白字，视觉冲击力拉满",
    badge: "大字报",
    textStyle: {
      defaultText: "这波操作绝了",
      size: 42,
      color: "#ffffff",
      strokeWidth: 4,
      strokeColor: "#000000",
      useBox: false,
      boxColor: "#000000",
      boxOpacity: 0.6,
      positionRatio: { x: 0.5, y: 0.78 },
    },
    suggestedPresetId: "hd",
  },
];

const CUSTOM_TEMPLATES_KEY = "meme_studio_custom_templates";
const ACTIVE_DRAFT_KEY = "meme_studio_active_draft";

export function getCustomTemplates(): MemeTemplate[] {
  if (typeof window === "undefined" || !window.localStorage) {
    return [];
  }
  try {
    const raw = localStorage.getItem(CUSTOM_TEMPLATES_KEY);
    if (!raw) return [];
    const list = JSON.parse(raw);
    return Array.isArray(list) ? list : [];
  } catch {
    return [];
  }
}

export function saveCustomTemplate(
  data: Omit<MemeTemplate, "id" | "category" | "createdAt">,
): MemeTemplate {
  const current = getCustomTemplates();
  const id = `custom-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
  const template: MemeTemplate = {
    ...data,
    id,
    category: "custom",
    createdAt: Date.now(),
  };
  const nextList = [template, ...current].slice(0, 30);
  try {
    localStorage.setItem(CUSTOM_TEMPLATES_KEY, JSON.stringify(nextList));
  } catch {
    // ignore
  }
  return template;
}

export function deleteCustomTemplate(id: string): void {
  const current = getCustomTemplates();
  const nextList = current.filter((t) => t.id !== id);
  try {
    localStorage.setItem(CUSTOM_TEMPLATES_KEY, JSON.stringify(nextList));
  } catch {
    // ignore
  }
}

export function getAllTemplates(): MemeTemplate[] {
  return [...BUILTIN_TEMPLATES, ...getCustomTemplates()];
}

export function saveDraftToStorage(draft: StudioDraft): void {
  if (typeof window === "undefined" || !window.localStorage) {
    return;
  }
  try {
    localStorage.setItem(ACTIVE_DRAFT_KEY, JSON.stringify(draft));
  } catch {
    // ignore
  }
}

export function loadDraftFromStorage(): StudioDraft | null {
  if (typeof window === "undefined" || !window.localStorage) {
    return null;
  }
  try {
    const raw = localStorage.getItem(ACTIVE_DRAFT_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (parsed && parsed.version === 1) {
      return parsed as StudioDraft;
    }
    return null;
  } catch {
    return null;
  }
}

export function clearDraftFromStorage(): void {
  if (typeof window === "undefined" || !window.localStorage) {
    return;
  }
  try {
    localStorage.removeItem(ACTIVE_DRAFT_KEY);
  } catch {
    // ignore
  }
}
