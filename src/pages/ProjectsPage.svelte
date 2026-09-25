<script lang="ts">
  import {
    CalendarClock,
    ChevronRight,
    Clock3,
    Copy,
    FilePlus2,
    Files,
    MoreHorizontal,
    Plus,
    Search,
    Trash2,
  } from '@lucide/svelte';
  import type { ApplicationProject } from '../lib/types';
  import { calculateProjectStats, formatDateTime, formatDeadline, projectDisplayName } from '../lib/utils';

  export let projects: ApplicationProject[] = [];
  export let onCreate: () => void = () => undefined;
  export let onOpen: (projectId: string) => void = () => undefined;
  export let onDuplicate: (projectId: string) => void = () => undefined;
  export let onDelete: (projectId: string) => void = () => undefined;

  let search = '';
  let menuProjectId: string | null = null;

  $: normalizedSearch = search.trim().toLocaleLowerCase('zh-CN');
  $: visibleProjects = [...projects]
    .filter((project) =>
      projectDisplayName(project).toLocaleLowerCase('zh-CN').includes(normalizedSearch),
    )
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));
</script>

<main class="page-shell">
  <header class="page-header">
    <div>
      <p class="eyebrow">申请管理</p>
      <h1>申请项目</h1>
      <p class="page-description">为每所院校保存独立的材料顺序和导出记录。</p>
    </div>
    <button class="button primary" type="button" on:click={onCreate}><Plus size={17} /> 新建申请项目</button>
  </header>

  {#if projects.length > 0}
    <div class="toolbar surface-toolbar">
      <label class="search-box">
        <Search size={16} />
        <input bind:value={search} aria-label="搜索申请项目" placeholder="搜索学校、学院或项目" />
      </label>
      <span class="toolbar-count">共 {visibleProjects.length} 个项目</span>
    </div>
  {/if}

  {#if projects.length === 0}
    <section class="empty-state large">
      <div class="empty-icon"><FilePlus2 size={28} /></div>
      <h2>创建你的第一个申请项目</h2>
      <p>选择常用材料、补充院校专用文件，然后按通知要求排列成最终 PDF。</p>
      <button class="button primary" type="button" on:click={onCreate}><Plus size={17} /> 新建申请项目</button>
    </section>
  {:else if visibleProjects.length === 0}
    <section class="empty-state">
      <div class="empty-icon"><Search size={25} /></div>
      <h2>没有找到“{search}”</h2>
      <p>尝试更换关键词。</p>
      <button class="button secondary" type="button" on:click={() => (search = '')}>清除搜索</button>
    </section>
  {:else}
    <section class="project-grid" aria-label="申请项目列表">
      {#each visibleProjects as project (project.id)}
        {@const stats = calculateProjectStats(project)}
        <article class="project-card">
          <button class="project-card-click" type="button" on:click={() => onOpen(project.id)}>
            <div class="project-card-top">
              <div class="school-avatar" aria-hidden="true">{project.school.trim().slice(0, 1) || '申'}</div>
              <div class="project-heading">
                <h2>{project.school || project.department || project.program || '未命名申请'}</h2>
                <p>{[project.department, project.program].filter(Boolean).join(' · ') || '尚未填写学院和项目'}</p>
              </div>
            </div>
            <div class="project-stats">
              <span><Files size={15} /> {stats.enabledFiles} 份材料</span>
              <span>{stats.totalPages} 页</span>
              {#if stats.missingRequired.length > 0}<span class="warning-dot">缺 {stats.missingRequired.length} 项</span>{/if}
            </div>
            <div class="project-meta-row">
              <span><CalendarClock size={15} /> {formatDeadline(project.deadline)}</span>
              <span><Clock3 size={15} /> {formatDateTime(project.lastExportedAt)}</span>
            </div>
            <div class="project-card-footer">
              <span>更新于 {formatDateTime(project.updatedAt)}</span>
              <span class="open-label">打开编排 <ChevronRight size={15} /></span>
            </div>
          </button>
          <div class="card-menu-wrap project-card-menu">
            <button
              class="icon-button"
              type="button"
              aria-label="更多操作"
              title="更多操作"
              on:click={() => (menuProjectId = menuProjectId === project.id ? null : project.id)}
            ><MoreHorizontal size={18} /></button>
            {#if menuProjectId === project.id}
              <div class="context-menu right-aligned">
                <button type="button" on:click={() => { menuProjectId = null; onDuplicate(project.id); }}><Copy size={15} /> 复制项目</button>
                <button class="danger-text" type="button" on:click={() => { menuProjectId = null; onDelete(project.id); }}><Trash2 size={15} /> 删除项目</button>
              </div>
            {/if}
          </div>
        </article>
      {/each}
    </section>
  {/if}
</main>
