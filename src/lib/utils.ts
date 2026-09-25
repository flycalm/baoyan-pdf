import type { ApplicationProject, ProjectStats } from './types';

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 KB';
  const units = ['B', 'KB', 'MB', 'GB'];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / 1024 ** index;
  const digits = index === 0 ? 0 : value >= 10 ? 1 : 2;
  return `${value.toFixed(digits)} ${units[index]}`;
}

export function formatDateTime(value?: string | null): string {
  if (!value) return '尚未导出';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat('zh-CN', {
    month: 'numeric',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
}

export function formatDeadline(value?: string | null): string {
  if (!value) return '未设置截止时间';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat('zh-CN', {
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
}

export function projectDisplayName(project: ApplicationProject): string {
  return [project.school, project.department, project.program].filter(Boolean).join(' · ') || '未命名申请';
}

export function calculateProjectStats(project: ApplicationProject): ProjectStats {
  const enabledModules = project.modules.filter((module) => module.enabled);
  const enabledFiles = enabledModules.flatMap((module) => module.files);
  const missingRequired = project.modules
    .filter((module) => module.required && (!module.enabled || module.files.length === 0))
    .map((module) => module.title);
  const invalidMaterials = enabledFiles
    .filter((file) => file.material.status !== 'ready')
    .map((file) => file.material.name);

  return {
    enabledFiles: enabledFiles.length,
    enabledModules: enabledModules.length,
    totalPages: enabledFiles.reduce((total, file) => total + file.material.pageCount, 0),
    totalSourceBytes: enabledFiles.reduce((total, file) => total + file.material.sizeBytes, 0),
    missingRequired,
    invalidMaterials,
  };
}

export function safeOutputStem(project: ApplicationProject): string {
  const raw = [project.school, project.department, project.program, '申请材料'].filter(Boolean).join('-');
  const cleaned = raw
    .replace(/[<>:"/\\|?*\u0000-\u001f]/g, '-')
    .replace(/[. ]+$/g, '')
    .replace(/-+/g, '-')
    .trim();
  return cleaned.slice(0, 120) || '推免申请材料';
}

export function moveItem<T>(items: T[], from: number, to: number): T[] {
  if (from === to || from < 0 || to < 0 || from >= items.length || to >= items.length) {
    return items;
  }
  const next = [...items];
  const [item] = next.splice(from, 1);
  next.splice(to, 0, item);
  return next;
}

