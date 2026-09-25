<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { ChevronLeft, ChevronRight, FileText, LoaderCircle, ZoomIn, ZoomOut } from '@lucide/svelte';
  import * as pdfjs from 'pdfjs-dist';
  import type { PDFDocumentLoadingTask, PDFDocumentProxy, PDFPageProxy } from 'pdfjs-dist';
  import type { Material } from '../lib/types';
  import { api, isDesktop } from '../lib/api';
  import { formatBytes } from '../lib/utils';
  import Modal from './Modal.svelte';

  pdfjs.GlobalWorkerOptions.workerSrc = new URL('pdfjs-dist/build/pdf.worker.min.mjs', import.meta.url).toString();

  export let open = false;
  export let material: Material | null = null;
  export let onClose: () => void = () => undefined;

  let canvas: HTMLCanvasElement;
  let loadingTask: PDFDocumentLoadingTask | null = null;
  let document: PDFDocumentProxy | null = null;
  let page: PDFPageProxy | null = null;
  let pageNumber = 1;
  let scale = 1.15;
  let loading = false;
  let error = '';
  let loadedMaterialId: string | null = null;
  let renderToken = 0;

  $: if (open && material && material.id !== loadedMaterialId) loadDocument(material);
  $: if (!open) {
    loadedMaterialId = null;
    void loadingTask?.destroy();
    loadingTask = null;
    document = null;
    page = null;
  }

  async function loadDocument(target: Material) {
    loading = true;
    error = '';
    loadedMaterialId = target.id;
    pageNumber = 1;
    try {
      if (!isDesktop) {
        loading = false;
        return;
      }
      const path = await api.previewPath(target.id);
      if (!path) throw new Error('未找到可预览的文件。');
      await loadingTask?.destroy();
      const task = pdfjs.getDocument({
        url: convertFileSrc(path),
        enableXfa: false,
        stopAtErrors: false,
      });
      loadingTask = task;
      document = await task.promise;
      await renderPage();
    } catch (reason) {
      error = reason instanceof Error ? reason.message : '无法预览此 PDF。';
    } finally {
      loading = false;
    }
  }

  async function renderPage() {
    if (!document || !canvas) return;
    const currentToken = ++renderToken;
    page = await document.getPage(pageNumber);
    const viewport = page.getViewport({ scale });
    const pixelRatio = Math.min(window.devicePixelRatio || 1, 2);
    canvas.width = Math.floor(viewport.width * pixelRatio);
    canvas.height = Math.floor(viewport.height * pixelRatio);
    canvas.style.width = `${Math.floor(viewport.width)}px`;
    canvas.style.height = `${Math.floor(viewport.height)}px`;
    const context = canvas.getContext('2d');
    if (!context) return;
    context.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
    await page.render({ canvas, canvasContext: context, viewport }).promise;
    if (currentToken !== renderToken) return;
  }

  async function goTo(next: number) {
    if (!document) return;
    pageNumber = Math.min(Math.max(1, next), document.numPages);
    await renderPage();
  }

  async function zoom(delta: number) {
    scale = Math.min(2, Math.max(0.65, scale + delta));
    await renderPage();
  }
</script>

<Modal {open} title={material?.name || 'PDF 预览'} onClose={onClose} width="min(1040px, calc(100vw - 48px))">
  <div class="preview-shell">
    <div class="preview-toolbar">
      <div class="preview-meta">
        <FileText size={16} />
        <span>{material?.pageCount || 0} 页 · {formatBytes(material?.sizeBytes || 0)}</span>
      </div>
      <div class="preview-controls">
        <button class="icon-button" type="button" aria-label="上一页" disabled={!document || pageNumber <= 1} on:click={() => goTo(pageNumber - 1)}><ChevronLeft size={17} /></button>
        <span>{pageNumber} / {document?.numPages || material?.pageCount || 1}</span>
        <button class="icon-button" type="button" aria-label="下一页" disabled={!document || pageNumber >= document.numPages} on:click={() => goTo(pageNumber + 1)}><ChevronRight size={17} /></button>
        <span class="toolbar-divider"></span>
        <button class="icon-button" type="button" aria-label="缩小" disabled={!document} on:click={() => zoom(-0.15)}><ZoomOut size={17} /></button>
        <button class="icon-button" type="button" aria-label="放大" disabled={!document} on:click={() => zoom(0.15)}><ZoomIn size={17} /></button>
      </div>
    </div>
    <div class="preview-stage">
      {#if loading}
        <div class="preview-message"><LoaderCircle class="spin" size={24} /><span>正在加载预览…</span></div>
      {:else if error}
        <div class="preview-message error-message"><strong>无法预览</strong><span>{error}</span></div>
      {:else if !isDesktop}
        <div class="preview-message"><FileText size={32} /><strong>浏览器设计预览</strong><span>桌面版本中会在这里显示 PDF 页面。</span></div>
      {/if}
      <canvas class:hidden={loading || Boolean(error) || !isDesktop} bind:this={canvas}></canvas>
    </div>
  </div>
</Modal>
