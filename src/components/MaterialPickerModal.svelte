<script lang="ts">
  import { Check, CheckSquare2, FileText, Search, Square } from '@lucide/svelte';
  import type { Material } from '../lib/types';
  import { formatBytes } from '../lib/utils';
  import Modal from './Modal.svelte';

  export let open = false;
  export let materials: Material[] = [];
  export let addedMaterialIds: string[] = [];
  export let busy = false;
  export let onClose: () => void = () => undefined;
  export let onAdd: (materialIds: string[]) => void | Promise<void> = () => undefined;

  let search = '';
  let selected = new Set<string>();

  $: if (!open) {
    search = '';
    selected = new Set();
  }
  $: normalizedSearch = search.trim().toLocaleLowerCase('zh-CN');
  $: addedIds = new Set(addedMaterialIds);
  $: available = materials
    .filter((item) => item.scope === 'library' && item.status === 'ready')
    .filter((item) => !addedIds.has(item.id));
  $: visible = available
    .filter((item) => `${item.name} ${item.category}`.toLocaleLowerCase('zh-CN').includes(normalizedSearch));
  $: allVisibleSelected = visible.length > 0 && visible.every((item) => selected.has(item.id));

  function toggle(id: string) {
    const next = new Set(selected);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selected = next;
  }

  function toggleVisible() {
    const next = new Set(selected);
    if (allVisibleSelected) visible.forEach((item) => next.delete(item.id));
    else visible.forEach((item) => next.add(item.id));
    selected = next;
  }

  function submit() {
    if (busy || selected.size === 0) return;
    return onAdd([...selected]);
  }
</script>

<Modal {open} title="从材料库添加" description="选择本次申请需要使用的通用材料；已添加的材料不会重复显示。" closeOnBackdrop={!busy} onClose={onClose} width="680px">
  <label class="search-box modal-search"><Search size={16} /><input bind:value={search} placeholder="搜索名称或分类" /></label>
  <div class="picker-toolbar">
    <button
      class="picker-select-all"
      type="button"
      disabled={busy || visible.length === 0}
      aria-pressed={allVisibleSelected}
      on:click={toggleVisible}
    >
      {#if allVisibleSelected}<CheckSquare2 size={16} /> 取消选择当前 {visible.length} 份{:else}<Square size={16} /> 全选当前 {visible.length} 份{/if}
    </button>
    <span>材料库可选 {available.length} 份{normalizedSearch ? `，当前显示 ${visible.length} 份` : ''}</span>
  </div>
  <div class="picker-list">
    {#each visible as material (material.id)}
      <button class:selected={selected.has(material.id)} class="picker-row" type="button" disabled={busy} aria-pressed={selected.has(material.id)} on:click={() => toggle(material.id)}>
        <span class="picker-check">{#if selected.has(material.id)}<Check size={15} />{/if}</span>
        <span class="file-icon"><FileText size={18} /></span>
        <span class="picker-copy"><strong>{material.name}</strong><small>{material.category}</small></span>
        <span class="picker-meta">{material.pageCount} 页 · {formatBytes(material.sizeBytes)}</span>
      </button>
    {:else}
      <div class="inline-empty">{available.length === 0 ? '材料库中的可用材料都已添加到当前项目。' : '没有匹配的材料。'}</div>
    {/each}
  </div>
  <svelte:fragment slot="footer">
    <span class="footer-hint">已选择 {selected.size} 份</span>
    <button class="button secondary" type="button" disabled={busy} on:click={onClose}>取消</button>
    <button class="button primary" type="button" disabled={busy || selected.size === 0} on:click={submit}>{busy ? '正在添加…' : `添加 ${selected.size} 份到项目`}</button>
  </svelte:fragment>
</Modal>
