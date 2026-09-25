<script lang="ts">
  import { onMount } from 'svelte';
  import { open as choosePath, save as chooseSavePath } from '@tauri-apps/plugin-dialog';
  import { openPath, openUrl, revealItemInDir } from '@tauri-apps/plugin-opener';
  import { AlertTriangle, LoaderCircle } from '@lucide/svelte';
  import ConfirmDialog from './components/ConfirmDialog.svelte';
  import ImportMaterialsModal from './components/ImportMaterialsModal.svelte';
  import MaterialPickerModal from './components/MaterialPickerModal.svelte';
  import NewProjectModal from './components/NewProjectModal.svelte';
  import PdfPreviewModal from './components/PdfPreviewModal.svelte';
  import RenameMaterialModal from './components/RenameMaterialModal.svelte';
  import SaveTemplateModal from './components/SaveTemplateModal.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import ToastHost from './components/ToastHost.svelte';
  import { api, isDesktop } from './lib/api';
  import { buildAiArrangedProject, type AiRequirementMatch } from './lib/aiMatching';
  import {
    beginAiUndoTransaction,
    isAiUndoValid,
    persistAiUndoStateToStorage,
    removeAiUndoRecord,
    removeAiUndoRecordIfSame,
    restoreAiUndoState,
    rollbackAiUndoTransaction,
    shouldInvalidateAiUndoAfterSave,
    type AiUndoState,
  } from './lib/aiUndo';
  import type {
    AiConnectionConfig,
    AiConnectionTestResult,
    AiRegistrationAnalysis,
    AppSettings,
    AppSnapshot,
    ApplicationProject,
    CreateProjectInput,
    Material,
    MaterialCategory,
    PageId,
    PdfCompressionLevel,
    ProjectTemplate,
    ToastMessage,
  } from './lib/types';
  import { calculateProjectStats, formatBytes, projectDisplayName } from './lib/utils';
  import LibraryPage from './pages/LibraryPage.svelte';
  import ProjectEditor from './pages/ProjectEditor.svelte';
  import ProjectsPage from './pages/ProjectsPage.svelte';
  import SettingsPage from './pages/SettingsPage.svelte';
  import TemplatesPage from './pages/TemplatesPage.svelte';

  let snapshot: AppSnapshot | null = null;
  let loading = true;
  let fatalError = '';
  let currentPage: PageId = 'projects';
  let currentProjectId: string | null = null;
  let sidebarCollapsed = false;
  let newProjectOpen = false;
  let initialTemplateId = '';
  let importOpen = false;
  let importContext: 'library' | 'project' = 'library';
  let materialPickerOpen = false;
  let previewMaterial: Material | null = null;
  let renameMaterial: Material | null = null;
  let saveTemplateOpen = false;
  let deleteTarget: { type: 'project' | 'material' | 'template'; id: string; label: string } | null = null;
  let deleteBusy = false;
  let actionBusy = false;
  let exportBusy = false;
  let projectEditorRefreshRevision = 0;
  let exportConfirm: { projectId: string; outputPath: string; warnings: string[]; compressionLevel: PdfCompressionLevel } | null = null;
  let overwriteConfirm: { projectId: string; outputPath: string; allowWarnings: boolean; compressionLevel: PdfCompressionLevel } | null = null;
  let savingProjects = new Set<string>();
  let saveSequences = new Map<string, number>();
  let saveChains = new Map<string, Promise<void>>();
  let toastSequence = 0;
  let toasts: ToastMessage[] = [];
  let aiUndoState: AiUndoState = {};
  let aiUndoPersistenceWarningShown = false;

  const AI_UNDO_STORAGE_KEY = 'baoyan-pdf:ai-undo:v1';

  $: currentProject = snapshot?.projects.find((project) => project.id === currentProjectId) || null;
  $: currentProjectMaterialIds = currentProject
    ? currentProject.modules.flatMap((module) => module.files.map((file) => file.materialId))
    : [];
  $: aiMaterials = currentProject && snapshot
    ? snapshot.materials.filter(
        (material) => material.scope === 'library'
          || (material.scope === 'project' && material.scopeProjectId === currentProject.id),
      )
    : [];
  $: currentAiUndoRecord = currentProject ? aiUndoState[currentProject.id] || null : null;
  $: currentAiUndoValid = Boolean(
    currentProject && currentAiUndoRecord && isAiUndoValid(currentAiUndoRecord, currentProject),
  );

  onMount(() => {
    void loadSnapshot();
  });

  async function loadSnapshot() {
    loading = true;
    fatalError = '';
    try {
      snapshot = await api.getSnapshot();
      snapshot = {
        ...snapshot,
        settings: Object.assign({
          aiEnabled: false,
          aiBaseUrl: 'https://open.bigmodel.cn/api/paas/v4',
          aiApiKey: '',
          aiModel: 'glm-5.3-flash',
        }, snapshot.settings),
      };
      aiUndoState = restoreAiUndoState(readStoredAiUndo(), snapshot.projects);
      persistAiUndoState();
      currentPage = snapshot.settings.lastPage || 'projects';
      currentProjectId = snapshot.settings.lastProjectId || null;
      if (currentProjectId && !snapshot.projects.some((project) => project.id === currentProjectId)) {
        currentProjectId = null;
      }
    } catch (error) {
      fatalError = messageOf(error);
    } finally {
      loading = false;
    }
  }

  function messageOf(error: unknown): string {
    return error instanceof Error ? error.message : String(error || '操作失败，请重试。');
  }

  function readStoredAiUndo(): string | null {
    try {
      return window.localStorage.getItem(AI_UNDO_STORAGE_KEY);
    } catch {
      return null;
    }
  }

  function persistAiUndoState(): boolean {
    try {
      return persistAiUndoStateToStorage(aiUndoState, window.localStorage, AI_UNDO_STORAGE_KEY);
    } catch {
      return false;
    }
  }

  function replaceAiUndoState(next: AiUndoState, warnOnFailure = true): boolean {
    aiUndoState = next;
    const persisted = persistAiUndoState();
    if (!persisted && warnOnFailure && !aiUndoPersistenceWarningShown) {
      aiUndoPersistenceWarningShown = true;
      showToast(
        'warning',
        '撤销记录无法持久保存',
        '本次运行内仍可撤销，关闭应用后撤销记录可能失效。',
        8000,
      );
    }
    return persisted;
  }

  function invalidateAiUndo(projectId: string) {
    replaceAiUndoState(removeAiUndoRecord(aiUndoState, projectId));
  }

  function showToast(
    tone: ToastMessage['tone'],
    title: string,
    message?: string,
    timeout = 4600,
  ) {
    const toast: ToastMessage = { id: ++toastSequence, tone, title, message };
    toasts = [...toasts, toast];
    window.setTimeout(() => dismissToast(toast.id), timeout);
  }

  function dismissToast(id: number) {
    toasts = toasts.filter((toast) => toast.id !== id);
  }

  function navigate(page: PageId) {
    currentPage = page;
    currentProjectId = null;
    persistNavigation();
  }

  function openProject(projectId: string) {
    currentPage = 'projects';
    currentProjectId = projectId;
    persistNavigation();
  }

  function persistNavigation() {
    if (!snapshot) return;
    snapshot = {
      ...snapshot,
      settings: {
        ...snapshot.settings,
        lastPage: currentPage,
        lastProjectId: currentProjectId,
      },
    };
    void api.updateSettings(snapshot.settings).catch(() => undefined);
  }

  function openNewProject(templateId = '') {
    initialTemplateId = templateId;
    newProjectOpen = true;
  }

  async function createProject(input: CreateProjectInput) {
    if (!snapshot) return;
    actionBusy = true;
    try {
      const project = await api.createProject(input);
      snapshot = { ...snapshot, projects: [project, ...snapshot.projects] };
      newProjectOpen = false;
      openProject(project.id);
      showToast('success', '项目已创建', projectDisplayName(project));
    } catch (error) {
      showToast('error', '无法创建项目', messageOf(error));
    } finally {
      actionBusy = false;
    }
  }

  function updateProjectLocal(project: ApplicationProject) {
    if (!snapshot) return;
    snapshot = {
      ...snapshot,
      projects: snapshot.projects.map((item) => (item.id === project.id ? structuredClone(project) : item)),
    };
  }

  async function saveProject(
    project: ApplicationProject,
    options: { preserveAiUndo?: boolean } = {},
  ) {
    if (!snapshot) return;
    const undoRecord = aiUndoState[project.id];
    const undoToInvalidate = shouldInvalidateAiUndoAfterSave(
      undoRecord,
      project,
      Boolean(options.preserveAiUndo),
    ) ? undoRecord : null;
    updateProjectLocal(project);
    const sequence = (saveSequences.get(project.id) || 0) + 1;
    saveSequences.set(project.id, sequence);
    savingProjects = new Set(savingProjects).add(project.id);
    const previous = saveChains.get(project.id) || Promise.resolve();
    const operation = previous.catch(() => undefined).then(async () => {
      const saved = await api.updateProject(project);
      if (undoToInvalidate) {
        replaceAiUndoState(removeAiUndoRecordIfSame(aiUndoState, undoToInvalidate));
      }
      if (saveSequences.get(project.id) === sequence) updateProjectLocal(saved);
    });
    saveChains.set(project.id, operation);
    try {
      await operation;
    } catch (error) {
      showToast('error', '项目保存失败', messageOf(error), 7000);
      throw error;
    } finally {
      if (saveChains.get(project.id) === operation) {
        saveChains.delete(project.id);
        const next = new Set(savingProjects);
        next.delete(project.id);
        savingProjects = next;
      }
    }
  }

  async function analyzeRegistrationNotice(input: { noticeUrl: string; noticeText: string; requestId: string }): Promise<AiRegistrationAnalysis> {
    if (!snapshot?.settings.aiEnabled) throw new Error('请先在设置中启用 AI 整理。');
    const config: AiConnectionConfig = {
      apiKey: snapshot.settings.aiApiKey,
      baseUrl: snapshot.settings.aiBaseUrl,
      model: snapshot.settings.aiModel,
    };
    if (!config.apiKey.trim() || !config.baseUrl.trim() || !config.model.trim()) {
      throw new Error('AI 配置不完整，请先到设置中填写并测试连接。');
    }
    return api.analyzeRegistrationNotice({ ...input, config });
  }

  async function cancelAiAnalysis(requestId: string): Promise<boolean> {
    return api.cancelAiAnalysis(requestId);
  }

  async function applyAiArrangement(baseProject: ApplicationProject, matches: AiRequirementMatch[]) {
    if (!snapshot || baseProject.id !== currentProjectId) throw new Error('当前项目已经切换，请重新打开 AI 整理。');
    const previous = structuredClone(baseProject);
    const arranged = buildAiArrangedProject(baseProject, matches);
    const undoTransaction = beginAiUndoTransaction(aiUndoState, previous, arranged);
    replaceAiUndoState(undoTransaction.preparedState);
    try {
      const operation = saveProject(arranged, { preserveAiUndo: true });
      projectEditorRefreshRevision += 1;
      await operation;
      showToast('success', 'AI 编排已应用', '请检查缺失材料和最终顺序；需要时可一键撤销。');
    } catch (error) {
      replaceAiUndoState(rollbackAiUndoTransaction(undoTransaction));
      updateProjectLocal(previous);
      projectEditorRefreshRevision += 1;
      throw error;
    }
  }

  async function undoAiArrangement() {
    if (!snapshot || !currentProject) return;
    const projectId = currentProject.id;
    const undoRecord = aiUndoState[projectId];
    if (!undoRecord || !isAiUndoValid(undoRecord, currentProject)) {
      invalidateAiUndo(projectId);
      showToast('info', '无法撤销 AI 编排', '项目在 AI 编排后已经修改，为避免覆盖新内容，旧撤销记录已失效。');
      return;
    }
    const applied = structuredClone(currentProject);
    const previous = structuredClone(undoRecord.previousProject);
    try {
      const operation = saveProject(previous, { preserveAiUndo: true });
      projectEditorRefreshRevision += 1;
      await operation;
      invalidateAiUndo(projectId);
      showToast('success', '已撤销 AI 编排', '项目已恢复到应用 AI 之前的状态。');
    } catch {
      updateProjectLocal(applied);
      projectEditorRefreshRevision += 1;
      // saveProject already reports the detailed error and the undo remains available.
    }
  }

  async function updateAppSettings(settings: AppSettings) {
    if (!snapshot) return;
    const previous = snapshot.settings;
    snapshot = { ...snapshot, settings: structuredClone(settings) };
    try {
      await api.updateSettings(settings);
      showToast('success', '设置已保存');
    } catch (error) {
      snapshot = { ...snapshot, settings: previous };
      showToast('error', '无法保存设置', messageOf(error));
      throw error;
    }
  }

  async function testAiConnection(config: AiConnectionConfig): Promise<AiConnectionTestResult> {
    return api.testAiConnection(config);
  }

  async function duplicateProject(projectId: string) {
    if (!snapshot) return;
    try {
      const copy = await api.duplicateProject(projectId);
      snapshot = { ...snapshot, projects: [copy, ...snapshot.projects] };
      openProject(copy.id);
      showToast('success', '项目副本已创建');
    } catch (error) {
      showToast('error', '无法复制项目', messageOf(error));
    }
  }

  function requestDeleteProject(projectId: string) {
    const target = snapshot?.projects.find((project) => project.id === projectId);
    if (!target) return;
    deleteTarget = { type: 'project', id: projectId, label: projectDisplayName(target) };
  }

  function requestDeleteMaterial(material: Material) {
    deleteTarget = { type: 'material', id: material.id, label: material.name };
  }

  function requestDeleteTemplate(template: ProjectTemplate) {
    deleteTarget = { type: 'template', id: template.id, label: template.name };
  }

  async function confirmDelete() {
    if (!snapshot || !deleteTarget) return;
    deleteBusy = true;
    const target = deleteTarget;
    try {
      if (target.type === 'project') {
        await api.deleteProject(target.id);
        snapshot = { ...snapshot, projects: snapshot.projects.filter((item) => item.id !== target.id) };
        invalidateAiUndo(target.id);
        if (currentProjectId === target.id) navigate('projects');
      } else if (target.type === 'material') {
        await api.deleteMaterial(target.id);
        snapshot = { ...snapshot, materials: snapshot.materials.filter((item) => item.id !== target.id) };
      } else {
        await api.deleteTemplate(target.id);
        snapshot = { ...snapshot, templates: snapshot.templates.filter((item) => item.id !== target.id) };
      }
      showToast('success', '已删除', target.label);
      deleteTarget = null;
    } catch (error) {
      showToast('error', '无法删除', messageOf(error), 7000);
    } finally {
      deleteBusy = false;
    }
  }

  function startImport(context: 'library' | 'project') {
    importContext = context;
    importOpen = true;
  }

  async function choosePdfFiles(): Promise<string[]> {
    if (!isDesktop) return ['C:\\示例材料\\新导入材料.pdf'];
    const selected = await choosePath({
      multiple: true,
      directory: false,
      filters: [{ name: 'PDF 文档', extensions: ['pdf'] }],
      title: '选择要导入的 PDF',
    });
    if (!selected) return [];
    return Array.isArray(selected) ? selected : [selected];
  }

  function attachMaterials(project: ApplicationProject, materials: Material[]): ApplicationProject {
    const next = structuredClone(project);
    for (const material of materials) {
      let module = next.modules.find((entry) => entry.category === material.category);
      if (!module) {
        module = {
          id: crypto.randomUUID(),
          title: material.category,
          category: material.category,
          position: next.modules.length,
          required: false,
          enabled: true,
          files: [],
        };
        next.modules.push(module);
      }
      if (!module.files.some((file) => file.materialId === material.id)) {
        module.files.push({
          id: crypto.randomUUID(),
          materialId: material.id,
          position: module.files.length,
          material: structuredClone(material),
        });
      }
    }
    next.updatedAt = new Date().toISOString();
    return next;
  }

  async function importMaterials(paths: string[], category: MaterialCategory) {
    if (!snapshot) return;
    actionBusy = true;
    try {
      const result = await api.importMaterials({
        paths,
        category,
        scopeProjectId: importContext === 'project' ? currentProjectId : null,
      });
      snapshot = { ...snapshot, materials: [...snapshot.materials, ...result.imported] };
      if (importContext === 'project' && currentProject) {
        const saveOperation = saveProject(attachMaterials(currentProject, result.imported));
        projectEditorRefreshRevision += 1;
        await saveOperation;
      }
      importOpen = false;
      if (result.imported.length > 0) showToast('success', `已导入 ${result.imported.length} 份材料`);
      if (result.failures.length > 0) {
        showToast('warning', `${result.failures.length} 份文件未导入`, result.failures.map((item) => item.reason).join('；'), 8000);
      }
    } catch (error) {
      showToast('error', '导入失败', messageOf(error), 7000);
    } finally {
      actionBusy = false;
    }
  }

  async function addLibraryMaterials(materialIds: string[]) {
    if (!snapshot || !currentProject) return;
    actionBusy = true;
    try {
      const materials = snapshot.materials.filter((item) => materialIds.includes(item.id));
      const existingIds = new Set(currentProjectMaterialIds);
      const nextProject = attachMaterials(currentProject, materials);
      const addedCount = materials.filter((material) => !existingIds.has(material.id)).length;
      if (addedCount === 0) {
        materialPickerOpen = false;
        showToast('info', '所选材料已在当前项目中');
        return;
      }

      const saveOperation = saveProject(nextProject);
      projectEditorRefreshRevision += 1;
      materialPickerOpen = false;
      await saveOperation;
      showToast('success', `已添加 ${addedCount} 份材料`);
    } catch {
      // saveProject has already shown the detailed error message.
    } finally {
      actionBusy = false;
    }
  }

  async function renameSelectedMaterial(name: string) {
    if (!snapshot || !renameMaterial) return;
    actionBusy = true;
    try {
      const updated = await api.renameMaterial(renameMaterial.id, name);
      snapshot = {
        ...snapshot,
        materials: snapshot.materials.map((item) => (item.id === updated.id ? updated : item)),
        projects: snapshot.projects.map((project) => ({
          ...project,
          modules: project.modules.map((module) => ({
            ...module,
            files: module.files.map((file) =>
              file.materialId === updated.id ? { ...file, material: updated } : file,
            ),
          })),
        })),
      };
      renameMaterial = null;
      showToast('success', '材料名称已更新');
    } catch (error) {
      showToast('error', '无法重命名', messageOf(error));
    } finally {
      actionBusy = false;
    }
  }

  async function saveCurrentAsTemplate(name: string, description: string) {
    if (!snapshot || !currentProjectId) return;
    actionBusy = true;
    try {
      const template = await api.saveTemplate(currentProjectId, name, description);
      snapshot = { ...snapshot, templates: [template, ...snapshot.templates] };
      saveTemplateOpen = false;
      showToast('success', '模板已保存', template.name);
    } catch (error) {
      showToast('error', '无法保存模板', messageOf(error));
    } finally {
      actionBusy = false;
    }
  }

  async function requestExport(suggestedFileName: string, compressionLevel: PdfCompressionLevel) {
    if (!snapshot || !currentProject) return;
    const defaultPath = snapshot.settings.defaultExportDirectory
      ? `${snapshot.settings.defaultExportDirectory}\\${suggestedFileName}`
      : suggestedFileName;
    const outputPath = isDesktop
      ? await chooseSavePath({
          title: '保存申请材料',
          defaultPath,
          filters: [{ name: 'PDF 文档', extensions: ['pdf'] }],
        })
      : `C:\\Users\\Demo\\Documents\\${suggestedFileName}`;
    if (!outputPath) return;
    const stats = calculateProjectStats(currentProject);
    const warnings: string[] = [];
    if (stats.missingRequired.length > 0) warnings.push(`缺少必需模块：${stats.missingRequired.join('、')}`);
    if (compressionLevel === 'none' && currentProject.sizeLimitMb && stats.totalSourceBytes > currentProject.sizeLimitMb * 1024 * 1024) {
      warnings.push(`源文件合计已超过 ${currentProject.sizeLimitMb}MB 限制`);
    }
    if (warnings.length > 0) {
      exportConfirm = { projectId: currentProject.id, outputPath, warnings, compressionLevel };
    } else {
      await runExport(currentProject.id, outputPath, false, compressionLevel);
    }
  }

  async function runExport(
    projectId: string,
    outputPath: string,
    allowWarnings: boolean,
    compressionLevel: PdfCompressionLevel,
    overwrite = false,
  ) {
    if (!snapshot) return;
    exportBusy = true;
    try {
      const result = await api.exportProject({
        projectId,
        outputPath,
        overwrite,
        allowWarnings,
        compressionLevel,
      });
      const project = snapshot.projects.find((item) => item.id === projectId);
      if (project) {
        project.lastExportPath = result.outputPath;
        project.lastExportedAt = new Date().toISOString();
        project.updatedAt = project.lastExportedAt;
        updateProjectLocal(project);
      }
      exportConfirm = null;
      overwriteConfirm = null;
      const savedPercent = result.sourceSizeBytes > result.sizeBytes
        ? Math.max(1, Math.round((1 - result.sizeBytes / result.sourceSizeBytes) * 100))
        : 0;
      const sizeSummary = compressionLevel === 'none'
        ? `文件大小 ${formatBytes(result.sizeBytes)}`
        : result.compressionApplied
          ? `压缩前 ${formatBytes(result.sourceSizeBytes)}，压缩后 ${formatBytes(result.sizeBytes)}，节省 ${savedPercent}%`
          : `压缩未使文件变小，已保留较小原版（${formatBytes(result.sizeBytes)}）`;
      showToast(
        result.exceededLimit ? 'warning' : 'success',
        result.exceededLimit ? 'PDF 已生成，但超过大小限制' : 'PDF 已生成',
        `${result.pageCount} 页；${sizeSummary}；保存到 ${result.outputPath}`,
        7000,
      );
    } catch (error) {
      const message = messageOf(error);
      if (!overwrite && message.includes('目标文件已存在')) {
        exportConfirm = null;
        overwriteConfirm = { projectId, outputPath, allowWarnings, compressionLevel };
      } else {
        showToast('error', '无法生成 PDF', message, 9000);
      }
    } finally {
      exportBusy = false;
    }
  }

  async function safelyOpenUrl(url: string) {
    try {
      const parsed = new URL(url);
      if (!['http:', 'https:'].includes(parsed.protocol)) throw new Error('仅支持 http 或 https 链接。');
      if (isDesktop) await openUrl(url);
      else window.open(url, '_blank', 'noopener,noreferrer');
    } catch (error) {
      showToast('error', '无法打开链接', messageOf(error));
    }
  }

  async function openLastExport(reveal = false) {
    if (!currentProject?.lastExportPath) return;
    try {
      if (!isDesktop) return;
      if (reveal) await revealItemInDir(currentProject.lastExportPath);
      else await openPath(currentProject.lastExportPath);
    } catch (error) {
      showToast('error', '无法打开导出文件', messageOf(error));
    }
  }

  async function chooseExportDirectory() {
    if (!snapshot) return;
    const path = isDesktop
      ? await choosePath({ directory: true, multiple: false, title: '选择默认导出文件夹' })
      : 'C:\\Users\\Demo\\Documents';
    if (!path || Array.isArray(path)) return;
    snapshot = { ...snapshot, settings: { ...snapshot.settings, defaultExportDirectory: path } };
    await api.updateSettings(snapshot.settings);
  }

  async function openDataDirectory() {
    if (!snapshot || !isDesktop) return;
    try {
      await openPath(snapshot.dataDirectory);
    } catch (error) {
      showToast('error', '无法打开数据目录', messageOf(error));
    }
  }
</script>

<svelte:head><title>保研材料助手</title></svelte:head>

{#if loading}
  <div class="app-loading"><div class="app-loading-mark"><LoaderCircle class="spin" size={24} /></div><strong>正在打开保研材料助手</strong><span>加载本地材料和申请项目…</span></div>
{:else if fatalError || !snapshot}
  <div class="fatal-state"><div class="empty-icon error"><AlertTriangle size={28} /></div><h1>应用无法启动</h1><p>{fatalError}</p><button class="button primary" type="button" on:click={loadSnapshot}>重新尝试</button></div>
{:else}
  <div class="app-frame">
    <Sidebar
      {currentPage}
      {currentProjectId}
      projects={snapshot.projects}
      collapsed={sidebarCollapsed}
      onNavigate={navigate}
      onOpenProject={openProject}
      onToggle={() => (sidebarCollapsed = !sidebarCollapsed)}
    />
    <div class="main-region">
      {#if currentProject}
        <ProjectEditor
          project={currentProject}
          refreshRevision={projectEditorRefreshRevision}
          saving={savingProjects.has(currentProject.id)}
          {exportBusy}
          aiEnabled={snapshot.settings.aiEnabled}
          {aiMaterials}
          canUndoAi={currentAiUndoValid}
          aiUndoRevision={currentAiUndoRecord?.appliedRevision || null}
          onBack={() => navigate('projects')}
          onSave={saveProject}
          onAddLibrary={() => (materialPickerOpen = true)}
          onImportSpecific={() => startImport('project')}
          onPreview={(material) => (previewMaterial = material)}
          onSaveTemplate={() => (saveTemplateOpen = true)}
          onDuplicate={() => duplicateProject(currentProject.id)}
          onDelete={() => requestDeleteProject(currentProject.id)}
          onExport={requestExport}
          onOpenUrl={safelyOpenUrl}
          onOpenLastExport={() => openLastExport(false)}
          onOpenLastExportFolder={() => openLastExport(true)}
          onAnalyzeAi={analyzeRegistrationNotice}
          onCancelAi={cancelAiAnalysis}
          onApplyAi={applyAiArrangement}
          onUndoAi={undoAiArrangement}
        />
      {:else if currentPage === 'projects'}
        <ProjectsPage projects={snapshot.projects} onCreate={() => openNewProject()} onOpen={openProject} onDuplicate={duplicateProject} onDelete={requestDeleteProject} />
      {:else if currentPage === 'library'}
        <LibraryPage materials={snapshot.materials} onImport={() => startImport('library')} onPreview={(material) => (previewMaterial = material)} onRename={(material) => (renameMaterial = material)} onDelete={requestDeleteMaterial} />
      {:else if currentPage === 'templates'}
        <TemplatesPage templates={snapshot.templates} onUse={openNewProject} onDelete={requestDeleteTemplate} />
      {:else}
        <SettingsPage
          {snapshot}
          onOpenDataDirectory={openDataDirectory}
          onChooseExportDirectory={chooseExportDirectory}
          onResetExportDirectory={async () => {
            if (!snapshot) return;
            await updateAppSettings({ ...snapshot.settings, defaultExportDirectory: null });
          }}
          onUpdateSettings={updateAppSettings}
          onTestAiConnection={testAiConnection}
        />
      {/if}
    </div>
  </div>

  <NewProjectModal templates={snapshot.templates} open={newProjectOpen} {initialTemplateId} busy={actionBusy} onClose={() => (newProjectOpen = false)} onCreate={createProject} />
  <ImportMaterialsModal open={importOpen} title={importContext === 'library' ? '导入到材料库' : '导入本项目专用材料'} busy={actionBusy} onClose={() => (importOpen = false)} onChooseFiles={choosePdfFiles} onImport={importMaterials} />
  <MaterialPickerModal open={materialPickerOpen} materials={snapshot.materials} addedMaterialIds={currentProjectMaterialIds} busy={actionBusy} onClose={() => (materialPickerOpen = false)} onAdd={addLibraryMaterials} />
  <RenameMaterialModal open={Boolean(renameMaterial)} material={renameMaterial} busy={actionBusy} onClose={() => (renameMaterial = null)} onRename={renameSelectedMaterial} />
  <SaveTemplateModal open={saveTemplateOpen} suggestedName={currentProject ? `${currentProject.school || '申请'}材料模板` : ''} busy={actionBusy} onClose={() => (saveTemplateOpen = false)} onSave={saveCurrentAsTemplate} />
  <PdfPreviewModal open={Boolean(previewMaterial)} material={previewMaterial} onClose={() => (previewMaterial = null)} />
  <ConfirmDialog
    open={Boolean(deleteTarget)}
    title={`删除${deleteTarget?.type === 'project' ? '项目' : deleteTarget?.type === 'template' ? '模板' : '材料'}`}
    message={`确定删除“${deleteTarget?.label || ''}”吗？${deleteTarget?.type === 'material' ? ' 被项目引用的材料会阻止删除。' : ' 此操作不会删除已经导出的 PDF。'}`}
    confirmLabel="删除"
    destructive
    busy={deleteBusy}
    onCancel={() => (deleteTarget = null)}
    onConfirm={confirmDelete}
  />
  <ConfirmDialog
    open={Boolean(exportConfirm)}
    title="仍有材料提醒"
    message={`当前项目存在以下提醒：${exportConfirm?.warnings.join('；') || ''}。是否仍要继续生成 PDF？`}
    confirmLabel="仍然生成"
    busy={exportBusy}
    onCancel={() => (exportConfirm = null)}
    onConfirm={() => {
      if (exportConfirm) return runExport(exportConfirm.projectId, exportConfirm.outputPath, true, exportConfirm.compressionLevel);
    }}
  />
  <ConfirmDialog
    open={Boolean(overwriteConfirm)}
    title="文件已存在"
    message={`“${overwriteConfirm?.outputPath.split(/[\\/]/).pop() || '申请材料.pdf'}”已经存在。是否用本次生成的 PDF 安全替换它？`}
    confirmLabel="替换文件"
    destructive
    busy={exportBusy}
    onCancel={() => (overwriteConfirm = null)}
    onConfirm={() => {
      if (overwriteConfirm) {
        return runExport(
          overwriteConfirm.projectId,
          overwriteConfirm.outputPath,
          overwriteConfirm.allowWarnings,
          overwriteConfirm.compressionLevel,
          true,
        );
      }
    }}
  />
  <ToastHost {toasts} onDismiss={dismissToast} />
{/if}
