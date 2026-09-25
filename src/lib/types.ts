export const MATERIAL_CATEGORIES = [
  '个人简历',
  '本科成绩单',
  '成绩排名证明',
  '英语成绩证明',
  '论文与科研成果',
  '获奖证书',
  '身份证明',
  '学籍证明',
  '推荐信',
  '申请表',
  '诚信承诺书',
  '政审材料',
  '研究计划',
  '其他材料',
] as const;

export type MaterialCategory = (typeof MATERIAL_CATEGORIES)[number] | string;
export type PageId = 'projects' | 'library' | 'templates' | 'settings';

export interface Material {
  id: string;
  name: string;
  category: MaterialCategory;
  originalName: string;
  sizeBytes: number;
  pageCount: number;
  sha256: string;
  scope: 'library' | 'project';
  scopeProjectId?: string | null;
  storedPath?: string;
  createdAt: string;
  updatedAt: string;
  usedByCount: number;
  status: 'ready' | 'missing' | 'invalid';
  warning?: string | null;
}

export interface ModuleFile {
  id: string;
  materialId: string;
  position: number;
  material: Material;
}

export interface ProjectModule {
  id: string;
  title: string;
  category: MaterialCategory;
  position: number;
  required: boolean;
  enabled: boolean;
  files: ModuleFile[];
}

export interface ApplicationProject {
  id: string;
  school: string;
  department: string;
  program: string;
  noticeUrl: string;
  deadline?: string | null;
  sizeLimitMb?: number | null;
  notes: string;
  modules: ProjectModule[];
  createdAt: string;
  updatedAt: string;
  lastExportPath?: string | null;
  lastExportedAt?: string | null;
}

export interface TemplateModule {
  id: string;
  title: string;
  category: MaterialCategory;
  position: number;
  required: boolean;
  enabled: boolean;
  materialIds: string[];
}

export interface ProjectTemplate {
  id: string;
  name: string;
  description: string;
  modules: TemplateModule[];
  createdAt: string;
  updatedAt: string;
}

export interface AppSettings {
  lastPage: PageId;
  lastProjectId?: string | null;
  defaultExportDirectory?: string | null;
  confirmBeforeOverwrite: boolean;
  aiEnabled: boolean;
  aiBaseUrl: string;
  aiApiKey: string;
  aiModel: string;
}

export interface AiConnectionConfig {
  apiKey: string;
  baseUrl: string;
  model: string;
}

export interface AiConnectionTestResult {
  ok: boolean;
  model: string;
  latencyMs: number;
  message: string;
}

export interface AiRegistrationNoticeInput {
  noticeUrl: string;
  noticeText: string;
  requestId?: string;
  config: AiConnectionConfig;
}

export interface AiRegistrationRequirement {
  name: string;
  category: MaterialCategory;
  required: boolean;
  details: string;
  format?: string | null;
  copies?: number | null;
  deadline?: string | null;
  evidenceIds: string[];
}

export interface AiRegistrationEvidence {
  id: string;
  sourceUrl: string;
  quote: string;
}

export interface AiRegistrationSource {
  url: string;
  title?: string | null;
  contentType?: string | null;
  status: string;
}

export interface AiRegistrationToolLog {
  round: number;
  url: string;
  status: string;
  contentType?: string | null;
  bytes?: number | null;
  extractedChars?: number | null;
  linksFound?: number | null;
  message?: string | null;
}

export interface AiRegistrationAnalysis {
  summary: string;
  requirements: AiRegistrationRequirement[];
  evidence: AiRegistrationEvidence[];
  source: AiRegistrationSource[];
  confidence: number;
  warnings: string[];
  toolLog: AiRegistrationToolLog[];
}

export interface AppSnapshot {
  materials: Material[];
  projects: ApplicationProject[];
  templates: ProjectTemplate[];
  settings: AppSettings;
  appVersion: string;
  dataDirectory: string;
  qpdfVersion?: string | null;
}

export interface CreateProjectInput {
  school: string;
  department: string;
  program: string;
  noticeUrl: string;
  deadline?: string | null;
  sizeLimitMb?: number | null;
  notes: string;
  templateId?: string | null;
}

export interface ImportMaterialsInput {
  paths: string[];
  category: MaterialCategory;
  scopeProjectId?: string | null;
}

export interface ImportFailure {
  path: string;
  reason: string;
}

export interface ImportResult {
  imported: Material[];
  failures: ImportFailure[];
  duplicates: Array<{ path: string; existingMaterialId: string }>;
}

export interface ProjectStats {
  enabledFiles: number;
  enabledModules: number;
  totalPages: number;
  totalSourceBytes: number;
  missingRequired: string[];
  invalidMaterials: string[];
}

export type PdfCompressionLevel = 'none' | 'lossless' | 'standard' | 'strong';

export interface ExportProjectInput {
  projectId: string;
  outputPath: string;
  overwrite: boolean;
  allowWarnings: boolean;
  compressionLevel: PdfCompressionLevel;
}

export interface ExportResult {
  outputPath: string;
  pageCount: number;
  sourceSizeBytes: number;
  sizeBytes: number;
  compressionApplied: boolean;
  exceededLimit: boolean;
  warnings: string[];
}

export interface AppErrorPayload {
  code?: string;
  message: string;
  detail?: string;
}

export interface ToastMessage {
  id: number;
  tone: 'success' | 'warning' | 'error' | 'info';
  title: string;
  message?: string;
  actionLabel?: string;
  onAction?: () => void;
}
