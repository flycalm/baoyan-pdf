<script lang="ts">
  import { onMount, tick } from 'svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { FilePlus2, FolderOpen, Plus, RotateCcw, X } from '@lucide/svelte';
  import { isDesktop } from '../lib/api';
  import { appendDroppedPdfPaths, isPhysicalPositionInsideElement } from '../lib/fileDrop';
  import {
    CUSTOM_MATERIAL_NAME_MAX_LENGTH,
    isValidCustomMaterialName,
    normalizeCustomMaterialName,
  } from '../lib/materialCategories';
  import { MATERIAL_CATEGORIES, type MaterialCategory } from '../lib/types';
  import Modal from './Modal.svelte';

  export let open = false;
  export let title = '导入 PDF';
  export let description = '文件会复制到软件管理的材料库，原文件保持不变。';
  export let busy = false;
  export let initialCategory: MaterialCategory = '个人简历';
  export let onClose: () => void = () => undefined;
  export let onChooseFiles: () => Promise<string[]> = async () => [];
  export let onImport: (paths: string[], category: MaterialCategory) => void | Promise<void> = () => undefined;

  let category: MaterialCategory = initialCategory;
  let paths: string[] = [];
  let customMode = false;
  let customName = '';
  let customNameInput: HTMLInputElement;
  let dropZone: HTMLButtonElement;
  let draggingOverDropZone = false;
  let dropFeedback = '';
  let dropFeedbackKind: 'success' | 'warning' = 'success';
  let windowScaleFactor = 1;
  let wasOpen = false;

  $: effectiveCategory = customMode ? normalizeCustomMaterialName(customName) : category;
  $: customNameValid = !customMode || isValidCustomMaterialName(customName);
  $: {
    if (open && !wasOpen) {
      category = initialCategory;
      paths = [];
      customMode = false;
      customName = '';
      draggingOverDropZone = false;
      dropFeedback = '';
    } else if (!open && wasOpen) {
      paths = [];
      draggingOverDropZone = false;
      dropFeedback = '';
    }
    wasOpen = open;
  }

  async function chooseFiles() {
    const selected = await onChooseFiles();
    if (selected.length > 0) {
      paths = appendDroppedPdfPaths(paths, selected).paths;
      dropFeedback = '';
    }
  }

  function isInsideDropZone(position: { x: number; y: number }) {
    if (!dropZone) return false;
    return isPhysicalPositionInsideElement(position, windowScaleFactor, dropZone.getBoundingClientRect());
  }

  function addDroppedFiles(droppedPaths: string[]) {
    const result = appendDroppedPdfPaths(paths, droppedPaths);
    paths = result.paths;
    dropFeedbackKind = result.addedCount > 0 ? 'success' : 'warning';

    if (result.addedCount === 0) {
      dropFeedback = result.duplicateCount > 0 && result.rejectedCount === 0
        ? '这些 PDF 已经在待导入列表中了。'
        : '未找到可添加的 PDF，非 PDF 文件已被忽略。';
      return;
    }

    const ignoredCount = result.duplicateCount + result.rejectedCount;
    dropFeedback = `已添加 ${result.addedCount} 份 PDF${ignoredCount > 0 ? `，另有 ${ignoredCount} 个文件被忽略` : ''}。`;
  }

  onMount(() => {
    if (!isDesktop) return;

    let disposed = false;
    let unlisten: UnlistenFn | undefined;

    void (async () => {
      windowScaleFactor = await getCurrentWindow().scaleFactor().catch(() => 1);
      const stopListening = await getCurrentWebview().onDragDropEvent(({ payload }) => {
        if (!open || busy) {
          draggingOverDropZone = false;
          return;
        }

        if (payload.type === 'leave') {
          draggingOverDropZone = false;
          return;
        }

        const inside = isInsideDropZone(payload.position);
        draggingOverDropZone = inside;
        if (payload.type === 'drop') {
          draggingOverDropZone = false;
          if (inside) addDroppedFiles(payload.paths);
        }
      });

      if (disposed) stopListening();
      else unlisten = stopListening;
    })().catch(() => {
      draggingOverDropZone = false;
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  });

  function fileName(path: string) {
    return path.split(/[\\/]/).pop() || path;
  }

  async function enableCustomName() {
    customMode = true;
    await tick();
    customNameInput?.focus();
  }

  function usePresetCategories() {
    customMode = false;
    customName = '';
  }

  function submit() {
    if (paths.length === 0 || !customNameValid) return;
    return onImport(paths, effectiveCategory);
  }
</script>

<Modal {open} {title} {description} closeOnBackdrop={!busy} onClose={onClose} width="620px">
  <div class="import-panel">
    {#if customMode}
      <div class="custom-category-control">
        <label class="field">
          <span>自定义材料名称</span>
          <input
            bind:this={customNameInput}
            bind:value={customName}
            maxlength={CUSTOM_MATERIAL_NAME_MAX_LENGTH}
            placeholder="例如：夏令营申请表"
            aria-invalid={!customNameValid}
            on:keydown={(event) => {
              if (event.key === 'Enter' && paths.length > 0 && customNameValid && !busy) submit();
            }}
          />
        </label>
        <div class="custom-category-meta">
          <small>导入后将显示为新的分类，也会作为项目中的模块名称。</small>
          <span>{Array.from(customName).length}/{CUSTOM_MATERIAL_NAME_MAX_LENGTH}</span>
        </div>
        <button class="link-button" type="button" disabled={busy} on:click={usePresetCategories}>
          <RotateCcw size={14} /> 改回预设分类
        </button>
      </div>
    {:else}
      <div class="preset-category-control">
        <label class="field">
          <span>材料分类</span>
          <select bind:value={category}>
            {#each MATERIAL_CATEGORIES as item}<option value={item}>{item}</option>{/each}
          </select>
        </label>
        <button class="link-button custom-category-trigger" type="button" disabled={busy} on:click={enableCustomName}>
          <Plus size={14} /> 自定义新的材料名称
        </button>
      </div>
    {/if}

    <button
      bind:this={dropZone}
      class:drag-over={draggingOverDropZone}
      class="file-drop-zone"
      type="button"
      disabled={busy}
      aria-label="选择或拖入一个或多个 PDF"
      on:click={chooseFiles}
    >
      <span class="drop-icon"><FolderOpen size={22} /></span>
      <strong>{draggingOverDropZone ? '松开即可添加 PDF' : paths.length ? '继续选择或拖入 PDF' : '选择或拖入一个或多个 PDF'}</strong>
      <small>支持从资源管理器拖拽，仅支持 PDF 文件</small>
    </button>

    {#if dropFeedback}
      <p class:warning={dropFeedbackKind === 'warning'} class="drop-feedback" role="status" aria-live="polite">{dropFeedback}</p>
    {/if}

    {#if paths.length > 0}
      <div class="selected-file-list">
        {#each paths as path, index}
          <div class="selected-file-row">
            <FilePlus2 size={16} />
            <span title={path}>{fileName(path)}</span>
            <button class="icon-button small" type="button" aria-label="移除" on:click={() => (paths = paths.filter((_, itemIndex) => itemIndex !== index))}><X size={15} /></button>
          </div>
        {/each}
      </div>
    {/if}
  </div>
  <svelte:fragment slot="footer">
    <button class="button secondary" type="button" disabled={busy} on:click={onClose}>取消</button>
    <button class="button primary" type="button" disabled={busy || paths.length === 0 || !customNameValid} on:click={submit}>
      {busy ? '正在导入…' : paths.length ? `导入 ${paths.length} 份材料` : '选择 PDF 后导入'}
    </button>
  </svelte:fragment>
</Modal>
