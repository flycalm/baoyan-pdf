<script lang="ts">
  import { ArrowRight, FileText, LayoutTemplate, MoreHorizontal, Plus, Trash2 } from '@lucide/svelte';
  import type { ProjectTemplate } from '../lib/types';
  import { formatDateTime } from '../lib/utils';

  export let templates: ProjectTemplate[] = [];
  export let onUse: (templateId: string) => void = () => undefined;
  export let onDelete: (template: ProjectTemplate) => void = () => undefined;

  let menuTemplateId: string | null = null;
</script>

<main class="page-shell">
  <header class="page-header">
    <div>
      <p class="eyebrow">快速复用材料组合</p>
      <h1>模板</h1>
      <p class="page-description">在项目编辑器中可把当前编排保存成模板。</p>
    </div>
  </header>

  {#if templates.length === 0}
    <section class="empty-state large">
      <div class="empty-icon"><LayoutTemplate size={28} /></div>
      <h2>还没有材料模板</h2>
      <p>打开一个申请项目并选择“保存为模板”，以后可直接复用相同的材料组合。</p>
    </section>
  {:else}
    <section class="template-grid">
      {#each templates as template (template.id)}
        <article class="template-card">
          <div class="template-card-top">
            <div class="template-icon"><LayoutTemplate size={20} /></div>
            <div class="card-menu-wrap">
              <button class="icon-button" type="button" aria-label="更多操作" on:click={() => (menuTemplateId = menuTemplateId === template.id ? null : template.id)}><MoreHorizontal size={18} /></button>
              {#if menuTemplateId === template.id}
                <div class="context-menu right-aligned">
                  <button class="danger-text" type="button" on:click={() => { menuTemplateId = null; onDelete(template); }}><Trash2 size={15} /> 删除模板</button>
                </div>
              {/if}
            </div>
          </div>
          <h2>{template.name}</h2>
          <p>{template.description || '未添加说明'}</p>
          <div class="template-modules">
            {#if template.modules.length === 0}
              <span class="placeholder-chip"><Plus size={13} /> 包含材料占位项</span>
            {:else}
              {#each template.modules.slice(0, 5) as module}
                <span><FileText size={13} /> {module.title}</span>
              {/each}
              {#if template.modules.length > 5}<span>+{template.modules.length - 5}</span>{/if}
            {/if}
          </div>
          <div class="template-footer">
            <small>更新于 {formatDateTime(template.updatedAt)}</small>
            <button class="button quiet" type="button" on:click={() => onUse(template.id)}>使用模板 <ArrowRight size={15} /></button>
          </div>
        </article>
      {/each}
    </section>
  {/if}
</main>
