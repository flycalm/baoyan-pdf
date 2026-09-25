import { invoke } from '@tauri-apps/api/core';
import { mockApi } from './mock';
import type {
  AiConnectionConfig,
  AiConnectionTestResult,
  AiRegistrationAnalysis,
  AiRegistrationNoticeInput,
  AppSettings,
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

export const isDesktop =
  typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);

function normalizeError(error: unknown): Error {
  if (error instanceof Error) return error;
  if (typeof error === 'string') {
    try {
      const parsed = JSON.parse(error) as { message?: string; detail?: string };
      return new Error(parsed.detail ? `${parsed.message}\n${parsed.detail}` : parsed.message || error);
    } catch {
      return new Error(error);
    }
  }
  if (error && typeof error === 'object' && 'message' in error) {
    return new Error(String((error as { message: unknown }).message));
  }
  return new Error('操作失败，请重试。');
}

async function desktopCall<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeError(error);
  }
}

export const api = {
  getSnapshot(): Promise<AppSnapshot> {
    return isDesktop ? desktopCall('get_app_snapshot') : mockApi.getSnapshot();
  },

  createProject(input: CreateProjectInput): Promise<ApplicationProject> {
    return isDesktop ? desktopCall('create_project', { input }) : mockApi.createProject(input);
  },

  updateProject(project: ApplicationProject): Promise<ApplicationProject> {
    return isDesktop ? desktopCall('update_project', { project }) : mockApi.updateProject(project);
  },

  deleteProject(projectId: string): Promise<void> {
    return isDesktop ? desktopCall('delete_project', { projectId }) : mockApi.deleteProject(projectId);
  },

  duplicateProject(projectId: string): Promise<ApplicationProject> {
    return isDesktop
      ? desktopCall('duplicate_project', { projectId })
      : mockApi.duplicateProject(projectId);
  },

  importMaterials(input: ImportMaterialsInput): Promise<ImportResult> {
    return isDesktop ? desktopCall('import_materials', { input }) : mockApi.importMaterials(input);
  },

  deleteMaterial(materialId: string): Promise<void> {
    return isDesktop
      ? desktopCall('delete_material', { materialId })
      : mockApi.deleteMaterial(materialId);
  },

  renameMaterial(materialId: string, name: string): Promise<Material> {
    return isDesktop
      ? desktopCall<Material>('rename_material', { materialId, name })
      : mockApi.renameMaterial(materialId, name);
  },

  saveTemplate(projectId: string, name: string, description: string): Promise<ProjectTemplate> {
    return isDesktop
      ? desktopCall('save_project_as_template', { projectId, name, description })
      : mockApi.saveTemplate(projectId, name, description);
  },

  deleteTemplate(templateId: string): Promise<void> {
    return isDesktop
      ? desktopCall('delete_template', { templateId })
      : mockApi.deleteTemplate(templateId);
  },

  exportProject(input: ExportProjectInput): Promise<ExportResult> {
    return isDesktop ? desktopCall('export_project', { input }) : mockApi.exportProject(input);
  },

  previewPath(materialId: string): Promise<string> {
    return isDesktop
      ? desktopCall('get_material_preview_path', { materialId })
      : mockApi.previewPath(materialId);
  },

  updateSettings(settings: AppSettings): Promise<void> {
    return isDesktop ? desktopCall('update_settings', { settings }) : mockApi.updateSettings(settings);
  },

  testAiConnection(config: AiConnectionConfig): Promise<AiConnectionTestResult> {
    return isDesktop
      ? desktopCall('test_ai_connection', { config })
      : mockApi.testAiConnection(config);
  },

  analyzeRegistrationNotice(input: AiRegistrationNoticeInput): Promise<AiRegistrationAnalysis> {
    return isDesktop
      ? desktopCall('analyze_registration_notice', { input })
      : mockApi.analyzeRegistrationNotice(input);
  },

  cancelAiAnalysis(requestId: string): Promise<boolean> {
    return isDesktop
      ? desktopCall('cancel_ai_analysis', { requestId })
      : mockApi.cancelAiAnalysis(requestId);
  },
};
