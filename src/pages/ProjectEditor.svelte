<script lang="ts">
  import { onDestroy } from 'svelte';
  import {
    AlertCircle,
    ArrowDown,
    ArrowLeft,
    ArrowUp,
    CalendarDays,
    Check,
    ChevronDown,
    ChevronRight,
    CircleHelp,
    Copy,
    Download,
    ExternalLink,
    Eye,
    FileOutput,
    FilePlus2,
    FileText,
    FolderOpen,
    GripVertical,
    LayoutTemplate,
    Link2,
    LoaderCircle,
    MoreHorizontal,
    Plus,
    Save,
    Sparkles,
    Trash2,
    Undo2,
  } from '@lucide/svelte';
  import AiAssistantModal from '../components/AiAssistantModal.svelte';
  import NewModuleModal from '../components/NewModuleModal.svelte';
  import type { AiRegistrationAnalysis, ApplicationProject, Material, PdfCompressionLevel, ProjectModule } from '../lib/types';
  import type { AiRequirementMatch } from '../lib/aiMatching';
  import { projectAiRevision } from '../lib/aiUndo';
  import {
    calculateProjectStats,
    formatBytes,
    formatDateTime,
    moveItem,
    projectDisplayName,
    safeOutputStem,
  } from '../lib/utils';

  export let project: ApplicationProject;
  export let refreshRevision = 0;
  export let saving = false;
  export let exportBusy = false;
  export let aiEnabled = false;
  export let aiMaterials: Material[] = [];
  export let canUndoAi = false;
  export let aiUndoRevision: string | null = null;
  export let onBack: () => void = () => undefined;
  export let onSave: (project: ApplicationProject) => void | Promise<void> = () => undefined;
  export let onAddLibrary: () => void = () => undefined;
  export let onImportSpecific: () => void = () => undefined;
  export let onPreview: (material: Material) => void = () => undefined;
  export let onSaveTemplate: () => void = () => undefined;
  export let onDuplicate: () => void = () => undefined;
  export let onDelete: () => void = () => undefined;
  export let onExport: (suggestedFileName: string, compressionLevel: PdfCompressionLevel) => void | Promise<void> = () => undefined;
  export let onOpenUrl: (url: string) => void = () => undefined;
  export let onOpenLastExport: () => void = () => undefined;
  export let onOpenLastExportFolder: () => void = () => undefined;
  export let onAnalyzeAi: (input: { noticeUrl: string; noticeText: string; requestId: string }) => Promise<AiRegistrationAnalysis> = async () => {
    throw new Error('AI 整理暂不可用。');
  };
  export let onCancelAi: (requestId: string) => Promise<boolean> = async () => false;
  export let onApplyAi: (project: ApplicationProject, matches: AiRequirementMatch[]) => void | Promise<void> = () => undefined;
  export let onUndoAi: () => void | Promise<void> = () => undefined;

  let draft = structuredClone(project);
  let loadedProjectId = project.id;
  let loadedRefreshRevision = refreshRevision;
  let expandedModules = new Set<string>(project.modules.map((module) => module.id));
  let infoExpanded = true;
  let moduleMenuId: string | null = null;
  let draggedModuleIndex: number | null = null;
  let moduleDropIndex: number | null = null;
  let draggedFile: { moduleId: string; fileId: string; index: number } | null = null;
  let fileDropTarget: { moduleId: string; index: number } | null = null;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let saveChain: Promise<void> = Promise.resolve();
  let lastLocalSave = project.updatedAt;
  let newModuleOpen = false;
  let newModuleBusy = false;
  let exportPreparing = false;
  let compressionLevel: PdfCompressionLevel = 'none';
  let aiAssistantOpen = false;
  let aiActionBusy = false;

  $: if (project.id !== loadedProjectId || refreshRevision !== loadedRefreshRevision) {
    const switchingProject = project.id !== loadedProjectId;
    const previousModules = new Map(draft.modules.map((module) => [module.id, module.files.length]));
    if (switchingProject && saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
      void enqueueSave(structuredClone(draft)).catch(() => undefined);
    }
    loadedProjectId = project.id;
    loadedRefreshRevision = refreshRevision;
    draft = structuredClone(project);
    expandedModules = switchingProject
      ? new Set(project.modules.map((module) => module.id))
      : new Set([
          ...expandedModules,
          ...project.modules
            .filter((module) => (previousModules.get(module.id) ?? -1) !== module.files.length)
            .map((module) => module.id),
        ]);
    lastLocalSave = project.updatedAt;
  }
  $: stats = calculateProjectStats(draft);
  $: sizeLimitBytes = draft.sizeLimitMb ? draft.sizeLimitMb * 1024 * 1024 : null;
  $: sizeRatio = sizeLimitBytes ? Math.min(1.25, stats.totalSourceBytes / sizeLimitBytes) : 0;
  $: estimatedOverLimit = Boolean(sizeLimitBytes && stats.totalSourceBytes > sizeLimitBytes);
  $: outputName = `${safeOutputStem(draft)}.pdf`;
  $: compressionHint = {
    none: '速度最快，不改变图片质量。',
    lossless: '整理 PDF 内部结构，不降低图片质量。',
    standard: '适合大多数报名材料，兼顾清晰度和体积。',
    strong: '更小体积，扫描件和图片画质可能下降。',
  }[compressionLevel];
  $: aiUndoAvailable = Boolean(
    canUndoAi && aiUndoRevision && projectAiRevision(draft) === aiUndoRevision,
  );

  function normalizePositions(target: ApplicationProject): ApplicationProject {
    target.modules = target.modules.map((module, moduleIndex) => ({
      ...module,
      position: moduleIndex,
      files: module.files.map((file, fileIndex) => ({ ...file, position: fileIndex })),
    }));
    return target;
  }

  function enqueueSave(snapshot: ApplicationProject): Promise<void> {
    const persist = async () => {
      await onSave(snapshot);
    };
    saveChain = saveChain.then(persist, persist);
    return saveChain;
  }

  function queueSave() {
    draft = normalizePositions({ ...draft, updatedAt: new Date().toISOString() });
    lastLocalSave = draft.updatedAt;
    if (saveTimer) clearTimeout(saveTimer);
    const snapshot = structuredClone(draft);
    saveTimer = setTimeout(() => {
      saveTimer = null;
      void enqueueSave(snapshot).catch(() => undefined);
    }, 450);
  }

  async function saveNow(): Promise<boolean> {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = null;
    draft = normalizePositions({ ...draft, updatedAt: new Date().toISOString() });
    lastLocalSave = draft.updatedAt;
    try {
      await enqueueSave(structuredClone(draft));
      return true;
    } catch {
      return false;
    }
  }

  onDestroy(() => {
    if (!saveTimer) return;
    clearTimeout(saveTimer);
    saveTimer = null;
    void enqueueSave(structuredClone(draft)).catch(() => undefined);
  });

  function toggleExpanded(moduleId: string) {
    const next = new Set(expandedModules);
    if (next.has(moduleId)) next.delete(moduleId);
    else next.add(moduleId);
    expandedModules = next;
  }

  function moveModule(from: number, to: number) {
    if (to < 0 || to >= draft.modules.length) return;
    draft = { ...draft, modules: moveItem(draft.modules, from, to) };
    saveNow();
  }

  function toggleModule(moduleId: string) {
    draft = {
      ...draft,
      modules: draft.modules.map((module) =>
        module.id === moduleId ? { ...module, enabled: !module.enabled } : module,
      ),
    };
    saveNow();
  }

  function toggleRequired(moduleId: string) {
    draft = {
      ...draft,
      modules: draft.modules.map((module) =>
        module.id === moduleId ? { ...module, required: !module.required } : module,
      ),
    };
    moduleMenuId = null;
    saveNow();
  }

  function removeModule(moduleId: string) {
    draft = { ...draft, modules: draft.modules.filter((module) => module.id !== moduleId) };
    moduleMenuId = null;
    saveNow();
  }

  function moveFile(moduleId: string, from: number, to: number) {
    const module = draft.modules.find((entry) => entry.id === moduleId);
    if (!module || to < 0 || to >= module.files.length) return;
    draft = {
      ...draft,
      modules: draft.modules.map((entry) =>
        entry.id === moduleId ? { ...entry, files: moveItem(entry.files, from, to) } : entry,
      ),
    };
    saveNow();
  }

  function removeFile(moduleId: string, fileId: string) {
    draft = {
      ...draft,
      modules: draft.modules.map((module) =>
        module.id === moduleId
          ? { ...module, files: module.files.filter((file) => file.id !== fileId) }
          : module,
      ),
    };
    saveNow();
  }

  async function openMaterialAction(action: () => void) {
    if (saveTimer && !(await saveNow())) return;
    action();
  }

  async function openAiAssistant() {
    if (!aiEnabled || saving || aiActionBusy) return;
    if (saveTimer && !(await saveNow())) return;
    aiAssistantOpen = true;
  }

  async function applyAiArrangement(matches: AiRequirementMatch[]) {
    aiActionBusy = true;
    try {
      if (!(await saveNow())) throw new Error('当前项目尚未保存，暂时无法应用 AI 编排。');
      await onApplyAi(structuredClone(draft), matches);
    } finally {
      aiActionBusy = false;
    }
  }

  async function undoAiArrangement() {
    if (saving || aiActionBusy) return;
    aiActionBusy = true;
    try {
      if (saveTimer && !(await saveNow())) return;
      await onUndoAi();
    } finally {
      aiActionBusy = false;
    }
  }

  function startModulePointerDrag(index: number, event: PointerEvent) {
    if (event.button > 0) return;
    event.preventDefault();
    draggedModuleIndex = index;
    moduleDropIndex = index;
    (event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);
  }

  function updateModulePointerDrag(event: PointerEvent) {
    if (draggedModuleIndex === null) return;
    event.preventDefault();
    autoScrollForPointer(event.clientY);
    const moduleList = document.querySelector<HTMLElement>('[data-module-list]');
    if (!moduleList) return;
    const cards = Array.from(moduleList.querySelectorAll<HTMLElement>('.module-card'));
    let insertionIndex = cards.length;
    for (let index = 0; index < cards.length; index += 1) {
      const bounds = cards[index].getBoundingClientRect();
      if (event.clientY < bounds.top + bounds.height / 2) {
        insertionIndex = index;
        break;
      }
    }
    moduleDropIndex = insertionIndex;
  }

  function finishModulePointerDrag(event: PointerEvent) {
    if (draggedModuleIndex === null) return;
    updateModulePointerDrag(event);
    const sourceIndex = draggedModuleIndex;
    const insertionIndex = moduleDropIndex ?? sourceIndex;
    const destination = sourceIndex < insertionIndex ? insertionIndex - 1 : insertionIndex;
    (event.currentTarget as HTMLElement).releasePointerCapture?.(event.pointerId);
    draggedModuleIndex = null;
    moduleDropIndex = null;
    if (destination !== sourceIndex) moveModule(sourceIndex, destination);
  }

  function cancelModulePointerDrag(event: PointerEvent) {
    (event.currentTarget as HTMLElement).releasePointerCapture?.(event.pointerId);
    draggedModuleIndex = null;
    moduleDropIndex = null;
  }

  function autoScrollForPointer(clientY: number) {
    const scroller = document.querySelector<HTMLElement>('.main-region');
    if (!scroller) return;
    const bounds = scroller.getBoundingClientRect();
    const edge = 72;
    if (clientY < bounds.top + edge) scroller.scrollTop -= 18;
    else if (clientY > bounds.bottom - edge) scroller.scrollTop += 18;
  }

  function startFilePointerDrag(moduleId: string, fileId: string, index: number, event: PointerEvent) {
    if (event.button > 0) return;
    event.preventDefault();
    draggedFile = { moduleId, fileId, index };
    fileDropTarget = { moduleId, index };
    (event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);
  }

  function updateFilePointerDrag(moduleId: string, event: PointerEvent) {
    if (!draggedFile || draggedFile.moduleId !== moduleId) return;
    event.preventDefault();
    autoScrollForPointer(event.clientY);
    const moduleFiles = Array.from(document.querySelectorAll<HTMLElement>('[data-module-files]'))
      .find((element) => element.dataset.moduleFiles === moduleId);
    if (!moduleFiles) return;
    const rows = Array.from(moduleFiles.querySelectorAll<HTMLElement>('.module-file-row'));
    let insertionIndex = rows.length;
    for (let index = 0; index < rows.length; index += 1) {
      const bounds = rows[index].getBoundingClientRect();
      if (event.clientY < bounds.top + bounds.height / 2) {
        insertionIndex = index;
        break;
      }
    }
    fileDropTarget = { moduleId, index: insertionIndex };
  }

  function finishFilePointerDrag(moduleId: string, event: PointerEvent) {
    if (!draggedFile || draggedFile.moduleId !== moduleId) return;
    updateFilePointerDrag(moduleId, event);
    const sourceIndex = draggedFile.index;
    const insertionIndex = fileDropTarget?.moduleId === moduleId ? fileDropTarget.index : sourceIndex;
    const destination = sourceIndex < insertionIndex ? insertionIndex - 1 : insertionIndex;
    (event.currentTarget as HTMLElement).releasePointerCapture?.(event.pointerId);
    draggedFile = null;
    fileDropTarget = null;
    if (destination !== sourceIndex) moveFile(moduleId, sourceIndex, destination);
  }

  function cancelFilePointerDrag(event: PointerEvent) {
    (event.currentTarget as HTMLElement).releasePointerCapture?.(event.pointerId);
    draggedFile = null;
    fileDropTarget = null;
  }

  function moduleTotals(module: ProjectModule) {
    return {
      pages: module.files.reduce((total, file) => total + file.material.pageCount, 0),
      size: module.files.reduce((total, file) => total + file.material.sizeBytes, 0),
    };
  }

  async function createEmptyModule(title: string) {
    newModuleBusy = true;
    const moduleId = crypto.randomUUID();
    draft = {
      ...draft,
      modules: [
        ...draft.modules,
        {
          id: moduleId,
          title,
          category: title,
          position: draft.modules.length,
          required: false,
          enabled: true,
          files: [],
        },
      ],
    };
    expandedModules = new Set(expandedModules).add(moduleId);
    try {
      if (await saveNow()) newModuleOpen = false;
    } finally {
      newModuleBusy = false;
    }
  }

  async function requestExport() {
    if (saving || exportBusy || exportPreparing) return;
    exportPreparing = true;
    try {
      if (!(await saveNow())) return;
      await onExport(outputName, compressionLevel);
    } finally {
      exportPreparing = false;
    }
  }
</script>

<main class="editor-shell">
  <header class="editor-header">
    <div class="editor-title-row">
      <button class="icon-button" type="button" aria-label="返回申请项目" title="返回申请项目" on:click={onBack}><ArrowLeft size={18} /></button>
      <div class="editor-title">
        <div class="breadcrumbs">申请项目 <span>/</span> {draft.school || '未命名申请'}</div>
        <h1>{projectDisplayName(draft)}</h1>
      </div>
      <div class="autosave-status" class:saving>
        {#if saving}<LoaderCircle class="spin" size={14} /> 正在保存…{:else}<Check size={14} /> 已自动保存{/if}
      </div>
    </div>
    <div class="editor-header-actions">
      {#if draft.noticeUrl}
        <button class="button quiet" type="button" on:click={() => onOpenUrl(draft.noticeUrl)}><ExternalLink size={15} /> 官方通知</button>
      {/if}
      <button class="button secondary" type="button" on:click={onSaveTemplate}><LayoutTemplate size={16} /> 保存为模板</button>
      <div class="card-menu-wrap">
        <button class="icon-button" type="button" aria-label="项目操作" title="项目操作" on:click={() => (moduleMenuId = moduleMenuId === '__project' ? null : '__project')}><MoreHorizontal size={18} /></button>
        {#if moduleMenuId === '__project'}
          <div class="context-menu right-aligned">
            <button type="button" on:click={() => { moduleMenuId = null; onDuplicate(); }}><Copy size={15} /> 复制项目</button>
            <button class="danger-text" type="button" on:click={() => { moduleMenuId = null; onDelete(); }}><Trash2 size={15} /> 删除项目</button>
          </div>
        {/if}
      </div>
    </div>
  </header>

  <div class="editor-content">
    <section class="composer-column">
      <div class="project-info-card">
        <button class="project-info-summary" type="button" on:click={() => (infoExpanded = !infoExpanded)}>
          <span>{#if infoExpanded}<ChevronDown size={17} />{:else}<ChevronRight size={17} />{/if}</span>
          <span><strong>申请信息</strong><small>截止时间、通知链接和备注</small></span>
          <span class="last-saved">本地更新于 {formatDateTime(lastLocalSave)}</span>
        </button>
        {#if infoExpanded}
          <div class="project-info-form">
            <label class="field"><span>学校</span><input bind:value={draft.school} on:input={queueSave} placeholder="学校名称" /></label>
            <label class="field"><span>学院 / 院系</span><input bind:value={draft.department} on:input={queueSave} placeholder="学院或院系" /></label>
            <label class="field"><span>项目名称</span><input bind:value={draft.program} on:input={queueSave} placeholder="预推免、夏令营等" /></label>
            <label class="field"><span><CalendarDays size={13} /> 截止时间</span><input type="datetime-local" value={draft.deadline ? new Date(draft.deadline).toISOString().slice(0, 16) : ''} on:change={(event) => { draft.deadline = (event.currentTarget as HTMLInputElement).value ? new Date((event.currentTarget as HTMLInputElement).value).toISOString() : null; saveNow(); }} /></label>
            <label class="field"><span>大小限制（MB，可选）</span><input type="number" min="1" max="1024" step="0.1" bind:value={draft.sizeLimitMb} on:input={queueSave} placeholder="留空表示不限" /></label>
            <label class="field info-url"><span><Link2 size={13} /> 官方通知</span><input type="url" bind:value={draft.noticeUrl} on:input={queueSave} placeholder="https://..." /></label>
            <label class="field info-notes"><span>备注</span><textarea rows="2" bind:value={draft.notes} on:input={queueSave} placeholder="记录导师志愿、提交方式等"></textarea></label>
          </div>
        {/if}
      </div>

      <div class="composer-toolbar">
        <div><h2>材料编排</h2><p>拖动模块或材料，调整最终 PDF 的先后顺序。</p></div>
        <div class="composer-actions">
          <button class="button ai-compose-button" type="button" disabled={!aiEnabled || saving || aiActionBusy} title={aiEnabled ? '从报名通知提取并匹配材料' : '请先在设置中启用 AI'} on:click={openAiAssistant}><Sparkles size={16} /> AI 整理材料</button>
          <button class="button secondary" type="button" on:click={() => openMaterialAction(onAddLibrary)}><Plus size={16} /> 从材料库添加</button>
          <button class="button secondary" type="button" on:click={() => openMaterialAction(onImportSpecific)}><FilePlus2 size={16} /> 导入专用材料</button>
        </div>
      </div>

      {#if aiUndoAvailable}
        <div class="ai-undo-banner"><span><Sparkles size={15} /><strong>已应用 AI 编排</strong> 如结果不合适，可恢复应用前的顺序与材料。</span><button class="button quiet" type="button" disabled={saving || aiActionBusy} on:click={undoAiArrangement}><Undo2 size={15} /> 撤销本次 AI 编排</button></div>
      {/if}

      {#if draft.modules.length === 0}
        <section class="empty-state composer-empty">
          <div class="empty-icon"><FilePlus2 size={26} /></div>
          <h2>把本次申请需要的材料放进来</h2>
          <p>从材料库选择常用材料，或导入报名表、承诺书等本项目专用 PDF。</p>
          <div class="empty-actions">
            <button class="button primary" type="button" on:click={() => openMaterialAction(onAddLibrary)}>从材料库添加</button>
            <button class="button secondary" type="button" on:click={() => openMaterialAction(onImportSpecific)}>导入专用材料</button>
          </div>
        </section>
      {:else}
        <div class="module-list" data-module-list>
          {#each draft.modules as module, moduleIndex (module.id)}
            {@const totals = moduleTotals(module)}
            {#if moduleDropIndex === moduleIndex && draggedModuleIndex !== null}
              <div class="module-drop-indicator" aria-hidden="true"></div>
            {/if}
            <article
              class:disabled={!module.enabled}
              class:dragging={draggedModuleIndex === moduleIndex}
              class="module-card"
            >
              <div class="module-header">
                <button
                  data-drag-handle
                  class="drag-handle"
                  type="button"
                  aria-label={`拖动${module.title}`}
                  title="拖动排序"
                  on:pointerdown={(event) => startModulePointerDrag(moduleIndex, event)}
                  on:pointermove={updateModulePointerDrag}
                  on:pointerup={finishModulePointerDrag}
                  on:pointercancel={cancelModulePointerDrag}
                ><GripVertical size={18} /></button>
                <span class="module-number">{String(moduleIndex + 1).padStart(2, '0')}</span>
                <button class="module-title-button" type="button" on:click={() => toggleExpanded(module.id)}>
                  <span class="module-icon"><FileText size={18} /></span>
                  <span class="module-title-copy"><strong>{module.title}</strong><small>{module.files.length} 份文件 · {totals.pages} 页 · {formatBytes(totals.size)}</small></span>
                  {#if module.required}<span class="required-chip">必需</span>{/if}
                  {#if expandedModules.has(module.id)}<ChevronDown size={17} />{:else}<ChevronRight size={17} />{/if}
                </button>
                <label class="switch" title={module.enabled ? '已参与导出' : '不参与导出'}>
                  <input type="checkbox" aria-label={`${module.title}参与导出`} checked={module.enabled} on:change={() => toggleModule(module.id)} />
                  <span></span>
                </label>
                <div class="card-menu-wrap">
                  <button class="icon-button" type="button" aria-label="模块操作" on:click={() => (moduleMenuId = moduleMenuId === module.id ? null : module.id)}><MoreHorizontal size={18} /></button>
                  {#if moduleMenuId === module.id}
                    <div class="context-menu right-aligned">
                      <button type="button" disabled={moduleIndex === 0} on:click={() => { moduleMenuId = null; moveModule(moduleIndex, moduleIndex - 1); }}><ArrowUp size={15} /> 上移</button>
                      <button type="button" disabled={moduleIndex === draft.modules.length - 1} on:click={() => { moduleMenuId = null; moveModule(moduleIndex, moduleIndex + 1); }}><ArrowDown size={15} /> 下移</button>
                      <button type="button" on:click={() => toggleRequired(module.id)}><CircleHelp size={15} /> {module.required ? '取消必需' : '标记为必需'}</button>
                      <button class="danger-text" type="button" on:click={() => removeModule(module.id)}><Trash2 size={15} /> 从项目移除</button>
                    </div>
                  {/if}
                </div>
              </div>

              {#if expandedModules.has(module.id)}
                <div class="module-files" data-module-files={module.id}>
                  {#each module.files as file, fileIndex (file.id)}
                    {#if fileDropTarget?.moduleId === module.id && fileDropTarget.index === fileIndex && draggedFile}
                      <div class="file-drop-indicator" aria-hidden="true"></div>
                    {/if}
                    <div
                      class:dragging={draggedFile?.fileId === file.id}
                      class="module-file-row"
                      role="listitem"
                    >
                      <button
                        class="drag-handle small-handle"
                        type="button"
                        aria-label={`拖动材料：${file.material.name}`}
                        title="拖动调整材料顺序"
                        on:pointerdown={(event) => startFilePointerDrag(module.id, file.id, fileIndex, event)}
                        on:pointermove={(event) => updateFilePointerDrag(module.id, event)}
                        on:pointerup={(event) => finishFilePointerDrag(module.id, event)}
                        on:pointercancel={cancelFilePointerDrag}
                      ><GripVertical size={16} /></button>
                      <span class="file-order">{fileIndex + 1}</span>
                      <button class="file-preview-button" type="button" on:click={() => onPreview(file.material)}>
                        <span><strong>{file.material.name}</strong><small>{file.material.originalName}</small></span>
                      </button>
                      <span class:project-specific={file.material.scope === 'project'} class="scope-chip">{file.material.scope === 'project' ? '本项目专用' : '材料库'}</span>
                      <span class="file-row-meta">{file.material.pageCount} 页 · {formatBytes(file.material.sizeBytes)}</span>
                      <button class="icon-button small" type="button" title="预览" aria-label="预览" on:click={() => onPreview(file.material)}><Eye size={16} /></button>
                      <button class="icon-button small" type="button" title="上移" aria-label="上移" disabled={fileIndex === 0} on:click={() => moveFile(module.id, fileIndex, fileIndex - 1)}><ArrowUp size={15} /></button>
                      <button class="icon-button small" type="button" title="下移" aria-label="下移" disabled={fileIndex === module.files.length - 1} on:click={() => moveFile(module.id, fileIndex, fileIndex + 1)}><ArrowDown size={15} /></button>
                      <button class="icon-button small danger-hover" type="button" title="从项目移除" aria-label="从项目移除" on:click={() => removeFile(module.id, file.id)}><Trash2 size={15} /></button>
                    </div>
                  {:else}
                    <div class="module-inline-empty"><AlertCircle size={16} /> 这个模块还没有文件。可以从材料库添加同分类材料。</div>
                  {/each}
                  {#if fileDropTarget?.moduleId === module.id && fileDropTarget.index === module.files.length && draggedFile}
                    <div class="file-drop-indicator" aria-hidden="true"></div>
                  {/if}
                </div>
              {/if}
            </article>
          {/each}
          {#if moduleDropIndex === draft.modules.length && draggedModuleIndex !== null}
            <div class="module-drop-indicator" aria-hidden="true"></div>
          {/if}
          <button class="add-module-button" type="button" on:click={() => (newModuleOpen = true)}><Plus size={16} /> 添加空白模块</button>
        </div>
      {/if}
    </section>

    <aside class="export-panel">
      <div class="export-panel-sticky">
        <div class="export-heading"><div class="export-icon"><FileOutput size={19} /></div><div><h2>检查与导出</h2><p>按当前顺序生成 PDF</p></div></div>

        <div class="stats-grid">
          <div><strong>{stats.enabledFiles}</strong><span>份文件</span></div>
          <div><strong>{stats.totalPages}</strong><span>总页数</span></div>
          <div><strong>{formatBytes(stats.totalSourceBytes)}</strong><span>源文件合计</span></div>
        </div>

        {#if sizeLimitBytes}
          <div class="size-check" class:over-limit={estimatedOverLimit}>
            <div class="size-check-row"><span>学校大小限制</span><strong>{formatBytes(sizeLimitBytes)}</strong></div>
            <div class="size-bar"><span style:width={`${Math.min(100, sizeRatio * 100)}%`}></span></div>
            <small>{estimatedOverLimit ? '源文件合计已超过限制，最终文件也可能超限。' : `当前约占限制的 ${Math.round(sizeRatio * 100)}%`}</small>
          </div>
        {/if}

        <div class="checks-list">
          <div class:check-warning={stats.missingRequired.length > 0} class="check-row">
            {#if stats.missingRequired.length > 0}<AlertCircle size={17} />{:else}<Check size={17} />{/if}
            <span>{stats.missingRequired.length > 0 ? `缺少 ${stats.missingRequired.length} 个必需模块` : '必需材料已就绪'}</span>
          </div>
          <div class:check-warning={stats.invalidMaterials.length > 0} class="check-row">
            {#if stats.invalidMaterials.length > 0}<AlertCircle size={17} />{:else}<Check size={17} />{/if}
            <span>{stats.invalidMaterials.length > 0 ? `${stats.invalidMaterials.length} 份文件不可用` : '所有文件均可读取'}</span>
          </div>
        </div>

        {#if stats.missingRequired.length > 0}
          <div class="warning-card"><strong>待补充</strong><p>{stats.missingRequired.join('、')}</p></div>
        {/if}

        <label class="field compression-control">
          <span>PDF 压缩</span>
          <select bind:value={compressionLevel} aria-label="PDF 压缩等级">
            <option value="none">不压缩（默认）</option>
            <option value="lossless">轻度（无损）</option>
            <option value="standard">标准（推荐）</option>
            <option value="strong">强力（画质可能下降）</option>
          </select>
          <small>{compressionHint}</small>
        </label>
        <label class="field output-name"><span>输出文件名</span><input value={outputName} readonly /></label>
        <button class="button primary export-button" type="button" disabled={saving || exportBusy || exportPreparing || stats.enabledFiles === 0 || stats.invalidMaterials.length > 0} on:click={requestExport}>
          {#if exportBusy || exportPreparing}<LoaderCircle class="spin" size={17} /> 正在生成…{:else}<Download size={17} /> 生成申请材料{/if}
        </button>
        <p class="export-footnote">导出时可选择保存位置；源 PDF 不会被修改。</p>

        {#if draft.lastExportPath}
          <div class="last-export-card">
            <div><Save size={16} /><span><strong>上次导出</strong><small>{formatDateTime(draft.lastExportedAt)}</small></span></div>
            <p title={draft.lastExportPath}>{draft.lastExportPath}</p>
            <div><button class="link-button" type="button" on:click={onOpenLastExport}>打开 PDF</button><button class="link-button" type="button" on:click={onOpenLastExportFolder}><FolderOpen size={14} /> 打开目录</button></div>
          </div>
        {/if}
      </div>
    </aside>
  </div>
</main>

<NewModuleModal
  open={newModuleOpen}
  busy={newModuleBusy}
  onClose={() => (newModuleOpen = false)}
  onCreate={createEmptyModule}
/>

<AiAssistantModal
  open={aiAssistantOpen}
  projectNoticeUrl={draft.noticeUrl}
  materials={aiMaterials}
  onAnalyze={onAnalyzeAi}
  onCancelAnalysis={onCancelAi}
  onApply={applyAiArrangement}
  onClose={() => (aiAssistantOpen = false)}
/>

<style>
  .ai-compose-button {
    border-color: #c7b8ef;
    color: #6246a8;
    background: #f5f1ff;
    box-shadow: var(--shadow-sm);
  }

  .ai-compose-button:hover:not(:disabled) {
    border-color: #a991e2;
    background: #eee7ff;
  }

  .ai-undo-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 10px;
    padding: 8px 10px 8px 12px;
    border: 1px solid #d8cef2;
    border-radius: 8px;
    color: #654e9a;
    background: #f8f5ff;
    font-size: 11px;
  }

  .ai-undo-banner > span {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .ai-undo-banner strong { color: #513887; }

  .module-drop-indicator,
  .file-drop-indicator {
    height: 2px;
    border-radius: 999px;
    background: var(--primary);
    pointer-events: none;
  }

  .module-drop-indicator {
    margin-block: -5px;
  }

  .file-drop-indicator {
    margin: -1px 4px;
  }

  @media (max-width: 900px) {
    .ai-undo-banner { align-items: flex-start; flex-direction: column; }
  }
</style>
