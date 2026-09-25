import type { ApplicationProject } from './types';

export interface AiUndoRecord {
  projectId: string;
  previousProject: ApplicationProject;
  appliedRevision: string;
  createdAt: string;
}

export type AiUndoState = Record<string, AiUndoRecord>;

export interface AiUndoTransaction {
  previousState: AiUndoState;
  preparedState: AiUndoState;
}

export interface AiUndoStorage {
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

interface StoredAiUndoState {
  version: 1;
  records: AiUndoRecord[];
}

export function projectAiRevision(project: ApplicationProject): string {
  return JSON.stringify({
    id: project.id,
    school: project.school,
    department: project.department,
    program: project.program,
    noticeUrl: project.noticeUrl,
    deadline: project.deadline ?? null,
    sizeLimitMb: project.sizeLimitMb ?? null,
    notes: project.notes,
    modules: project.modules.map((module) => ({
      id: module.id,
      title: module.title,
      category: module.category,
      position: module.position,
      required: module.required,
      enabled: module.enabled,
      files: module.files.map((file) => ({
        id: file.id,
        materialId: file.materialId,
        position: file.position,
      })),
    })),
  });
}

export function createAiUndoRecord(
  previousProject: ApplicationProject,
  appliedProject: ApplicationProject,
): AiUndoRecord {
  return {
    projectId: appliedProject.id,
    previousProject: structuredClone(previousProject),
    appliedRevision: projectAiRevision(appliedProject),
    createdAt: new Date().toISOString(),
  };
}

export function upsertAiUndoRecord(
  state: AiUndoState,
  previousProject: ApplicationProject,
  appliedProject: ApplicationProject,
): AiUndoState {
  const record = createAiUndoRecord(previousProject, appliedProject);
  return { ...state, [record.projectId]: record };
}

export function beginAiUndoTransaction(
  state: AiUndoState,
  previousProject: ApplicationProject,
  appliedProject: ApplicationProject,
): AiUndoTransaction {
  return {
    previousState: { ...state },
    preparedState: upsertAiUndoRecord(state, previousProject, appliedProject),
  };
}

export function rollbackAiUndoTransaction(transaction: AiUndoTransaction): AiUndoState {
  return transaction.previousState;
}

export function removeAiUndoRecord(state: AiUndoState, projectId: string): AiUndoState {
  if (!(projectId in state)) return state;
  const next = { ...state };
  delete next[projectId];
  return next;
}

export function shouldInvalidateAiUndoAfterSave(
  record: AiUndoRecord | null | undefined,
  project: ApplicationProject,
  preserveAiUndo = false,
): record is AiUndoRecord {
  return Boolean(!preserveAiUndo && record && !isAiUndoValid(record, project));
}

export function removeAiUndoRecordIfSame(
  state: AiUndoState,
  capturedRecord: AiUndoRecord,
): AiUndoState {
  const current = state[capturedRecord.projectId];
  if (!current
    || current.createdAt !== capturedRecord.createdAt
    || current.appliedRevision !== capturedRecord.appliedRevision) {
    return state;
  }
  return removeAiUndoRecord(state, capturedRecord.projectId);
}

export function isAiUndoValid(record: AiUndoRecord | null | undefined, project: ApplicationProject): boolean {
  return Boolean(
    record
      && record.projectId === project.id
      && record.previousProject.id === project.id
      && record.appliedRevision === projectAiRevision(project),
  );
}

export function serializeAiUndoState(state: AiUndoState): string {
  const payload: StoredAiUndoState = { version: 1, records: Object.values(state) };
  return JSON.stringify(payload);
}

export function persistAiUndoStateToStorage(
  state: AiUndoState,
  storage: AiUndoStorage,
  storageKey: string,
): boolean {
  try {
    if (Object.keys(state).length === 0) storage.removeItem(storageKey);
    else storage.setItem(storageKey, serializeAiUndoState(state));
    return true;
  } catch {
    return false;
  }
}

export function restoreAiUndoState(serialized: string | null, projects: ApplicationProject[]): AiUndoState {
  if (!serialized) return {};
  try {
    const payload = JSON.parse(serialized) as Partial<StoredAiUndoState>;
    if (payload.version !== 1 || !Array.isArray(payload.records)) return {};
    const projectsById = new Map(projects.map((project) => [project.id, project]));
    const state: AiUndoState = {};
    for (const candidate of payload.records) {
      if (!isStoredRecord(candidate)) continue;
      const project = projectsById.get(candidate.projectId);
      if (project && isAiUndoValid(candidate, project)) state[candidate.projectId] = structuredClone(candidate);
    }
    return state;
  } catch {
    return {};
  }
}

function isStoredRecord(value: unknown): value is AiUndoRecord {
  if (!value || typeof value !== 'object') return false;
  const record = value as Partial<AiUndoRecord>;
  const previous = record.previousProject as Partial<ApplicationProject> | undefined;
  return typeof record.projectId === 'string'
    && typeof record.appliedRevision === 'string'
    && typeof record.createdAt === 'string'
    && Boolean(previous)
    && previous?.id === record.projectId
    && typeof previous.school === 'string'
    && typeof previous.department === 'string'
    && typeof previous.program === 'string'
    && typeof previous.noticeUrl === 'string'
    && typeof previous.notes === 'string'
    && Array.isArray(previous.modules);
}
