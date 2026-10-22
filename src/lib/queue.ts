import type { RenderPlan } from "./desktop";

export type QueueItemStatus =
  | "pending"
  | "rendering"
  | "completed"
  | "failed"
  | "cancelled";

export interface QueueItem {
  id: string;
  title: string;
  inputFileName: string;
  timeRangeLabel: string;
  format: "gif" | "webp";
  status: QueueItemStatus;
  progress: number;
  stageMessage: string;
  outputPath: string;
  outputBytes?: number;
  errorMessage?: string;
  plan: RenderPlan;
  createdAt: number;
}

/**
 * 自动生成防冲突的不重复输出文件路径
 */
export function generateQueueOutputPath(
  baseDir: string,
  inputFileName: string,
  startSecs: number,
  endSecs: number,
  format: "gif" | "webp",
  existingPaths: Set<string>
): string {
  const cleanName =
    inputFileName
      .replace(/\.[^/.]+$/, "")
      .replace(/[^a-zA-Z0-9_\u4e00-\u9fa5-]/g, "_")
      .replace(/_+/g, "_")
      .replace(/^_|_$/g, "") || "clip";

  const timeTag = `${startSecs.toFixed(1)}s-${endSecs.toFixed(1)}s`;
  const normalizedBase = baseDir.replace(/[\\/]+$/, "") || ".";
  let candidate = `${normalizedBase}/${cleanName}_${timeTag}.${format}`;
  let counter = 1;

  while (existingPaths.has(candidate)) {
    candidate = `${normalizedBase}/${cleanName}_${timeTag}_(${counter}).${format}`;
    counter++;
  }
  return candidate;
}

/**
 * 基于当前工作区 RenderPlan 构造新的队列项
 */
export function createQueueItem(
  plan: RenderPlan,
  inputFileName: string,
  existingPaths: Set<string>
): QueueItem {
  const format: "gif" | "webp" = plan.format === "webp" ? "webp" : "gif";
  const startSecs = Number(plan.startSecs) || 0;
  const endSecs = Number(plan.endSecs) || 0;
  const timeRangeLabel = `${startSecs.toFixed(2)}s ~ ${endSecs.toFixed(2)}s`;

  // 解析默认输出目录
  let baseDir = ".";
  if (plan.outputPath) {
    const lastSlash = Math.max(
      plan.outputPath.lastIndexOf("/"),
      plan.outputPath.lastIndexOf("\\")
    );
    if (lastSlash > 0) {
      baseDir = plan.outputPath.substring(0, lastSlash);
    }
  }

  // 保证每一个队列项的输出文件不重复
  const uniqueOut = generateQueueOutputPath(
    baseDir,
    inputFileName || "clip",
    startSecs,
    endSecs,
    format,
    existingPaths
  );

  const snapshotPlan: RenderPlan = {
    ...plan,
    outputPath: uniqueOut,
    format,
  };

  const id = `queue-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
  const title = `${inputFileName.replace(/\.[^/.]+$/, "") || "片段"} (${timeRangeLabel})`;

  return {
    id,
    title,
    inputFileName: inputFileName || "video",
    timeRangeLabel,
    format,
    status: "pending",
    progress: 0,
    stageMessage: "等待开始",
    outputPath: uniqueOut,
    plan: snapshotPlan,
    createdAt: Date.now(),
  };
}

export function getQueueStatusBadge(status: QueueItemStatus): {
  label: string;
  badgeClass: string;
  icon: string;
} {
  switch (status) {
    case "pending":
      return { label: "排队中", badgeClass: "badge-pending", icon: "⏳" };
    case "rendering":
      return { label: "转码中", badgeClass: "badge-rendering", icon: "⚙️" };
    case "completed":
      return { label: "已完成", badgeClass: "badge-completed", icon: "✓" };
    case "failed":
      return { label: "失败", badgeClass: "badge-failed", icon: "✗" };
    case "cancelled":
      return { label: "已取消", badgeClass: "badge-cancelled", icon: "⊘" };
  }
}

export const QUEUE_STORAGE_KEY = "meme_studio_batch_queue";

export function serializeQueue(items: QueueItem[]): string {
  const sanitized = items.map((item) => ({
    ...item,
    status: item.status === "rendering" ? ("pending" as QueueItemStatus) : item.status,
    progress: item.status === "rendering" ? 0 : item.progress,
  }));
  return JSON.stringify(sanitized);
}

export function deserializeQueue(raw: string): QueueItem[] {
  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.map((item) => ({
      ...item,
      status: item.status === "rendering" ? "pending" : item.status,
      progress: item.status === "rendering" ? 0 : (item.progress ?? 0),
    }));
  } catch {
    return [];
  }
}

