import type {
  AiConnectionConfig,
  AiConnectionTestResult,
  AiRegistrationAnalysis,
  AiRegistrationNoticeInput,
  AppSnapshot,
  ApplicationProject,
  CreateProjectInput,
  ExportProjectInput,
  ExportResult,
  ImportMaterialsInput,
  ImportResult,
  Material,
  ProjectTemplate,
} from './types';

const now = new Date().toISOString();

function material(
  id: string,
  name: string,
  category: string,
  pages: number,
  sizeMb: number,
  usedByCount: number,
): Material {
  return {
    id,
    name,
    category,
    originalName: `${name}.pdf`,
    sizeBytes: Math.round(sizeMb * 1024 * 1024),
    pageCount: pages,
    sha256: id.repeat(8).slice(0, 64),
    scope: 'library',
    createdAt: now,
    updatedAt: now,
    usedByCount,
    status: 'ready',
  };
}

const seededMaterials: Material[] = [
  material('mat-resume', '个人简历（中文）', '个人简历', 2, 0.74, 3),
  material('mat-transcript', '本科成绩单（教务处盖章）', '本科成绩单', 4, 2.35, 3),
  material('mat-ranking', '专业排名证明', '成绩排名证明', 1, 0.48, 2),
  material('mat-cet6', '大学英语六级成绩单', '英语成绩证明', 1, 0.31, 3),
  material('mat-paper', '多模态检索论文', '论文与科研成果', 9, 4.72, 2),
  material('mat-award-1', '国家奖学金证书', '获奖证书', 1, 0.52, 2),
  material('mat-award-2', '数学建模竞赛证书', '获奖证书', 1, 0.61, 2),
  material('mat-enrollment', '学籍在线验证报告', '学籍证明', 2, 0.43, 1),
];

function fileRef(materialId: string, position: number) {
  const item = seededMaterials.find((entry) => entry.id === materialId)!;
  return {
    id: `ref-${materialId}-${position}`,
    materialId,
    position,
    material: structuredClone(item),
  };
}

const primaryProject: ApplicationProject = {
  id: 'project-sustech',
  school: '南方科技大学',
  department: '计算机科学与工程系',
  program: '2027年预推免',
  noticeUrl: 'https://cse.sustech.edu.cn/notices/3775.html',
  deadline: '2026-08-31T23:59:00+08:00',
  sizeLimitMb: 20,
  notes: '报名入口位于通知页二维码。提交前再次核对导师志愿顺序。',
  createdAt: now,
  updatedAt: now,
  lastExportPath: null,
  lastExportedAt: null,
  modules: [
    {
      id: 'mod-resume',
      title: '个人简历',
      category: '个人简历',
      position: 0,
      required: true,
      enabled: true,
      files: [fileRef('mat-resume', 0)],
    },
    {
      id: 'mod-transcript',
      title: '本科成绩单',
      category: '本科成绩单',
      position: 1,
      required: true,
      enabled: true,
      files: [fileRef('mat-transcript', 0)],
    },
    {
      id: 'mod-ranking',
      title: '排名证明',
      category: '成绩排名证明',
      position: 2,
      required: false,
      enabled: true,
      files: [fileRef('mat-ranking', 0)],
    },
    {
      id: 'mod-english',
      title: '英语成绩证明',
      category: '英语成绩证明',
      position: 3,
      required: true,
      enabled: true,
      files: [fileRef('mat-cet6', 0)],
    },
    {
      id: 'mod-paper',
      title: '论文与科研成果',
      category: '论文与科研成果',
      position: 4,
      required: false,
      enabled: true,
      files: [fileRef('mat-paper', 0)],
    },
    {
      id: 'mod-awards',
      title: '获奖证书',
      category: '获奖证书',
      position: 5,
      required: false,
      enabled: true,
      files: [fileRef('mat-award-1', 0), fileRef('mat-award-2', 1)],
    },
  ],
};

const secondProject: ApplicationProject = {
  ...structuredClone(primaryProject),
  id: 'project-xidian',
  school: '西安电子科技大学',
  department: '人工智能学院',
  program: '推免预报名',
  noticeUrl: 'https://sai.xidian.edu.cn/',
  deadline: '2026-09-03T17:00:00+08:00',
  sizeLimitMb: 30,
  notes: '提交前确认导师已同意接收。',
  updatedAt: new Date(Date.now() - 86_400_000).toISOString(),
};

const thirdProject: ApplicationProject = {
  ...structuredClone(primaryProject),
  id: 'project-nuaa',
  school: '南京航空航天大学',
  department: '人工智能学院',
  program: '推免生预报名',
  noticeUrl: 'https://www.graduate.nuaa.edu.cn/',
  deadline: '2026-09-05T17:00:00+08:00',
  sizeLimitMb: 20,
  notes: '成绩、排名、英语、科研及荣誉材料须按顺序合并。',
  lastExportPath: 'C:\\Users\\Demo\\Documents\\南航-人工智能学院-申请材料.pdf',
  lastExportedAt: new Date(Date.now() - 43_200_000).toISOString(),
  updatedAt: new Date(Date.now() - 43_200_000).toISOString(),
};

const seededTemplates: ProjectTemplate[] = [
  {
    id: 'template-master',
    name: '常规硕士推免',
    description: '简历、成绩单、排名、英语、科研与获奖材料',
    createdAt: now,
    updatedAt: now,
    modules: primaryProject.modules.map((module) => ({
      id: `tpl-${module.id}`,
      title: module.title,
      category: module.category,
      position: module.position,
      required: module.required,
      enabled: module.enabled,
      materialIds: module.files.map((file) => file.materialId),
    })),
  },
  {
    id: 'template-phd',
    name: '直博申请',
    description: '在常规材料基础上加入两封推荐信和研究计划占位项',
    createdAt: now,
    updatedAt: now,
    modules: [],
  },
];

let snapshot: AppSnapshot = {
  materials: seededMaterials,
  projects: [primaryProject, secondProject, thirdProject],
  templates: seededTemplates,
  settings: {
    lastPage: 'projects',
    lastProjectId: primaryProject.id,
    confirmBeforeOverwrite: true,
    aiEnabled: false,
    aiBaseUrl: 'https://open.bigmodel.cn/api/paas/v4',
    aiApiKey: '',
    aiModel: 'glm-5.3-flash',
  },
  appVersion: '0.2.0-preview',
  dataDirectory: '本地预览模式',
  qpdfVersion: 'qpdf 12.x（预览数据）',
};

const activeAiRequests = new Set<string>();
const cancelledAiRequests = new Set<string>();

function cloneSnapshot(): AppSnapshot {
  return structuredClone(snapshot);
}

function uid(prefix: string): string {
  return `${prefix}-${crypto.randomUUID()}`;
}

export const mockApi = {
  async getSnapshot(): Promise<AppSnapshot> {
    await delay(160);
    return cloneSnapshot();
  },

  async createProject(input: CreateProjectInput): Promise<ApplicationProject> {
    const template = input.templateId
      ? snapshot.templates.find((entry) => entry.id === input.templateId)
      : undefined;
    const project: ApplicationProject = {
      id: uid('project'),
      school: input.school,
      department: input.department,
      program: input.program,
      noticeUrl: input.noticeUrl,
      deadline: input.deadline,
      sizeLimitMb: input.sizeLimitMb,
      notes: input.notes,
      modules: template
        ? template.modules.map((module) => ({
            ...module,
            id: uid('module'),
            files: module.materialIds
              .map((materialId, position) => {
                const source = snapshot.materials.find((item) => item.id === materialId);
                return source
                  ? {
                      id: uid('ref'),
                      materialId,
                      position,
                      material: structuredClone(source),
                    }
                  : null;
              })
              .filter((item) => item !== null),
          }))
        : [],
      createdAt: now,
      updatedAt: now,
      lastExportPath: null,
      lastExportedAt: null,
    };
    snapshot.projects.unshift(project);
    return structuredClone(project);
  },

  async updateProject(project: ApplicationProject): Promise<ApplicationProject> {
    const index = snapshot.projects.findIndex((entry) => entry.id === project.id);
    const next = { ...structuredClone(project), updatedAt: new Date().toISOString() };
    if (index >= 0) snapshot.projects[index] = next;
    return structuredClone(next);
  },

  async deleteProject(projectId: string): Promise<void> {
    snapshot.projects = snapshot.projects.filter((entry) => entry.id !== projectId);
  },

  async duplicateProject(projectId: string): Promise<ApplicationProject> {
    const source = snapshot.projects.find((entry) => entry.id === projectId);
    if (!source) throw new Error('未找到要复制的项目。');
    const copy = structuredClone(source);
    copy.id = uid('project');
    copy.program = `${copy.program || '申请项目'}（副本）`;
    copy.modules = copy.modules.map((module) => ({
      ...module,
      id: uid('module'),
      files: module.files.map((file) => ({ ...file, id: uid('ref') })),
    }));
    copy.createdAt = now;
    copy.updatedAt = now;
    copy.lastExportPath = null;
    copy.lastExportedAt = null;
    snapshot.projects.unshift(copy);
    return structuredClone(copy);
  },

  async importMaterials(input: ImportMaterialsInput): Promise<ImportResult> {
    const imported = input.paths.map((path, index) => {
      const fileName = path.split(/[\\/]/).pop() || `材料 ${index + 1}.pdf`;
      const item: Material = {
        id: uid('material'),
        name: fileName.replace(/\.pdf$/i, ''),
        category: input.category,
        originalName: fileName,
        sizeBytes: 640_000 + index * 124_000,
        pageCount: 1 + index,
        sha256: uid('hash').replaceAll('-', '').slice(0, 64),
        scope: input.scopeProjectId ? 'project' : 'library',
        scopeProjectId: input.scopeProjectId,
        createdAt: now,
        updatedAt: now,
        usedByCount: 0,
        status: 'ready',
      };
      snapshot.materials.push(item);
      return item;
    });
    return { imported: structuredClone(imported), failures: [], duplicates: [] };
  },

  async saveProject(project: ApplicationProject): Promise<ApplicationProject> {
    return this.updateProject(project);
  },

  async deleteMaterial(materialId: string): Promise<void> {
    snapshot.materials = snapshot.materials.filter((entry) => entry.id !== materialId);
  },

  async renameMaterial(materialId: string, name: string): Promise<Material> {
    const item = snapshot.materials.find((entry) => entry.id === materialId);
    if (!item) throw new Error('未找到材料。');
    item.name = name.trim();
    item.updatedAt = new Date().toISOString();
    return structuredClone(item);
  },

  async saveTemplate(projectId: string, name: string, description: string): Promise<ProjectTemplate> {
    const project = snapshot.projects.find((entry) => entry.id === projectId);
    if (!project) throw new Error('未找到项目。');
    const template: ProjectTemplate = {
      id: uid('template'),
      name,
      description,
      createdAt: now,
      updatedAt: now,
      modules: project.modules.map((module) => ({
        id: uid('template-module'),
        title: module.title,
        category: module.category,
        position: module.position,
        required: module.required,
        enabled: module.enabled,
        materialIds: module.files
          .filter((file) => file.material.scope === 'library')
          .map((file) => file.materialId),
      })),
    };
    snapshot.templates.unshift(template);
    return structuredClone(template);
  },

  async deleteTemplate(templateId: string): Promise<void> {
    snapshot.templates = snapshot.templates.filter((entry) => entry.id !== templateId);
  },

  async exportProject(input: ExportProjectInput): Promise<ExportResult> {
    await delay(900);
    const project = snapshot.projects.find((entry) => entry.id === input.projectId);
    if (!project) throw new Error('未找到项目。');
    const files = project.modules.filter((module) => module.enabled).flatMap((module) => module.files);
    const sourceSizeBytes = files.reduce((total, file) => total + file.material.sizeBytes, 0);
    const compressionRatio = {
      none: 1,
      lossless: 0.96,
      standard: 0.78,
      strong: 0.6,
    }[input.compressionLevel];
    const sizeBytes = Math.round(sourceSizeBytes * compressionRatio);
    const pageCount = files.reduce((total, file) => total + file.material.pageCount, 0);
    project.lastExportPath = input.outputPath;
    project.lastExportedAt = new Date().toISOString();
    project.updatedAt = project.lastExportedAt;
    return {
      outputPath: input.outputPath,
      pageCount,
      sourceSizeBytes,
      sizeBytes,
      compressionApplied: sizeBytes < sourceSizeBytes,
      exceededLimit: Boolean(project.sizeLimitMb && sizeBytes > project.sizeLimitMb * 1024 * 1024),
      warnings: [],
    };
  },

  async previewPath(materialId: string): Promise<string> {
    return snapshot.materials.find((entry) => entry.id === materialId)?.storedPath || '';
  },

  async updateSettings(settings: AppSnapshot['settings']): Promise<void> {
    snapshot.settings = structuredClone(settings);
  },

  async testAiConnection(config: AiConnectionConfig): Promise<AiConnectionTestResult> {
    await delay(480);
    if (!config.baseUrl.trim() || !config.apiKey.trim() || !config.model.trim()) {
      return { ok: false, model: config.model, latencyMs: 0, message: '请填写 API 地址、API Key 和模型。' };
    }
    return { ok: true, model: config.model, latencyMs: 286, message: '连接成功，可以开始分析报名通知。' };
  },

  async analyzeRegistrationNotice(input: AiRegistrationNoticeInput): Promise<AiRegistrationAnalysis> {
    if (input.requestId) activeAiRequests.add(input.requestId);
    await delay(1_150);
    if (input.requestId && cancelledAiRequests.has(input.requestId)) {
      activeAiRequests.delete(input.requestId);
      cancelledAiRequests.delete(input.requestId);
      throw new Error('AI_ANALYSIS_CANCELLED');
    }
    if (input.requestId) activeAiRequests.delete(input.requestId);
    if (!input.noticeUrl.trim() && !input.noticeText.trim()) {
      throw new Error('请提供通知链接或粘贴通知正文。');
    }
    const sourceUrl = input.noticeUrl.trim() || 'pasted://registration-notice';
    return {
      summary: '通知要求按顺序提交个人简历、成绩单、外语水平证明，并可附科研成果和获奖证明。',
      requirements: [
        { name: '个人简历', category: '个人简历', required: true, details: '个人简历一份', evidenceIds: ['ev-1'] },
        { name: '本科成绩单', category: '本科成绩单', required: true, details: '本科阶段成绩单，建议加盖教务部门公章', evidenceIds: ['ev-2'] },
        { name: '外语水平证明', category: '英语成绩证明', required: true, details: '大学英语四六级或其他外语成绩证明', evidenceIds: ['ev-3'] },
        { name: '论文与科研成果', category: '论文与科研成果', required: false, details: '已发表论文、专利或科研项目证明，可选交', evidenceIds: ['ev-4'] },
        { name: '获奖证书', category: '获奖证书', required: false, details: '代表性获奖证书，可选交', evidenceIds: ['ev-4'] },
      ],
      evidence: [
        { id: 'ev-1', sourceUrl, quote: '申请材料包括个人简历。' },
        { id: 'ev-2', sourceUrl, quote: '本科阶段成绩单（须加盖公章）。' },
        { id: 'ev-3', sourceUrl, quote: '英语四、六级成绩或其他外语水平证明。' },
        { id: 'ev-4', sourceUrl, quote: '科研成果及获奖证明材料（如有）。' },
      ],
      source: [{ url: sourceUrl, title: input.noticeUrl ? '报名通知' : '粘贴的通知正文', contentType: 'text/html', status: 'read' }],
      confidence: 0.91,
      warnings: [],
      toolLog: [
        { round: 1, url: sourceUrl, status: 'read', contentType: 'text/html', extractedChars: 3860, linksFound: input.noticeUrl ? 2 : 0 },
        { round: 2, url: sourceUrl, status: 'verified', contentType: 'text/html', extractedChars: 1240, message: '复核材料清单与必交/选交表述' },
      ],
    };
  },

  async cancelAiAnalysis(requestId: string): Promise<boolean> {
    if (!activeAiRequests.has(requestId)) return false;
    cancelledAiRequests.add(requestId);
    return true;
  },
};

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
