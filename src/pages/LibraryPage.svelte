<script lang="ts">
  import {
    Eye,
    FileCheck2,
    FilePlus2,
    Files,
    MoreHorizontal,
    Pencil,
    Plus,
    Search,
    Trash2,
  } from '@lucide/svelte';
  import { availableLibraryCategories } from '../lib/materialCategories';
  import type { Material } from '../lib/types';
  import { formatBytes, formatDateTime } from '../lib/utils';

  export let materials: Material[] = [];
  export let onImport: () => void = () => undefined;
  export let onPreview: (material: Material) => void = () => undefined;
  export let onRename: (material: Material) => void = () => undefined;
  export let onDelete: (material: Material) => void = () => undefined;

  let search = '';
  let category = '全部材料';
  let menuMaterialId: string | null = null;

  $: availableCategories = [
    '全部材料',
    ...availableLibraryCategories(materials),
  ];
  $: if (!availableCategories.includes(category)) category = '全部材料';
  $: normalizedSearch = search.trim().toLocaleLowerCase('zh-CN');
  $: visibleMaterials = [...materials]
    .filter((material) => material.scope === 'library')
    .filter((material) => category === '全部材料' || material.category === category)
    .filter((material) =>
      `${material.name} ${material.originalName} ${material.category}`
        .toLocaleLowerCase('zh-CN')
        .includes(normalizedSearch),
    )
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));
</script>

<main class="page-shell">
  <header class="page-header">
    <div>
      <p class="eyebrow">重复使用，一次维护</p>
      <h1>材料库</h1>
      <p class="page-description">集中保存成绩单、简历、英语成绩和科研材料。</p>
    </div>
    <button class="button primary" type="button" on:click={onImport}><Plus size={17} /> 导入 PDF</button>
  </header>

  {#if materials.filter((item) => item.scope === 'library').length === 0}
    <section class="empty-state large">
      <div class="empty-icon"><FilePlus2 size={28} /></div>
      <h2>还没有常用材料</h2>
      <p>导入成绩单、简历、英语成绩等 PDF，只需保存一次，之后可在不同申请中重复使用。</p>
      <button class="button primary" type="button" on:click={onImport}><Plus size={17} /> 导入 PDF</button>
    </section>
  {:else}
    <div class="toolbar surface-toolbar library-toolbar">
      <label class="search-box">
        <Search size={16} />
        <input bind:value={search} aria-label="搜索材料" placeholder="搜索材料名称" />
      </label>
      <select bind:value={category} aria-label="按分类筛选">
        {#each availableCategories as item}<option value={item}>{item}</option>{/each}
      </select>
      <span class="toolbar-count">{visibleMaterials.length} 份材料</span>
    </div>

    {#if visibleMaterials.length === 0}
      <section class="empty-state compact">
        <div class="empty-icon"><Search size={24} /></div>
        <h2>没有匹配的材料</h2>
        <p>尝试更换关键词或分类。</p>
        <button class="button secondary" type="button" on:click={() => { search = ''; category = '全部材料'; }}>清除筛选</button>
      </section>
    {:else}
      <section class="material-table" aria-label="材料列表">
        <div class="material-table-head">
          <span>材料</span><span>分类</span><span>页数 / 大小</span><span>使用情况</span><span></span>
        </div>
        {#each visibleMaterials as material (material.id)}
          <article class="material-row">
            <button class="material-main" type="button" on:click={() => onPreview(material)}>
              <span class:problem={material.status !== 'ready'} class="file-icon"><FileCheck2 size={19} /></span>
              <span class="material-name">
                <strong>{material.name}</strong>
                <small>{material.originalName} · 更新于 {formatDateTime(material.updatedAt)}</small>
              </span>
            </button>
            <span><span class="category-chip">{material.category}</span></span>
            <span class="material-metric">{material.pageCount} 页<small>{formatBytes(material.sizeBytes)}</small></span>
            <span class="material-metric"><Files size={14} /> {material.usedByCount} 个项目</span>
            <div class="card-menu-wrap align-right">
              <button class="icon-button" type="button" aria-label="更多操作" on:click={() => (menuMaterialId = menuMaterialId === material.id ? null : material.id)}><MoreHorizontal size={18} /></button>
              {#if menuMaterialId === material.id}
                <div class="context-menu right-aligned">
                  <button type="button" on:click={() => { menuMaterialId = null; onPreview(material); }}><Eye size={15} /> 预览</button>
                  <button type="button" on:click={() => { menuMaterialId = null; onRename(material); }}><Pencil size={15} /> 重命名</button>
                  <button class="danger-text" type="button" on:click={() => { menuMaterialId = null; onDelete(material); }}><Trash2 size={15} /> 删除</button>
                </div>
              {/if}
            </div>
          </article>
        {/each}
      </section>
    {/if}
  {/if}
</main>
