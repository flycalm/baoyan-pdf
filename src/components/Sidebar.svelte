<script lang="ts">
  import {
    FileStack,
    FolderKanban,
    LayoutTemplate,
    Settings,
    PanelLeftClose,
    PanelLeftOpen,
    GraduationCap,
    HardDrive,
  } from '@lucide/svelte';
  import type { ApplicationProject, PageId } from '../lib/types';
  import { projectDisplayName } from '../lib/utils';

  export let currentPage: PageId = 'projects';
  export let currentProjectId: string | null = null;
  export let projects: ApplicationProject[] = [];
  export let collapsed = false;
  export let onNavigate: (page: PageId) => void = () => undefined;
  export let onOpenProject: (projectId: string) => void = () => undefined;
  export let onToggle: () => void = () => undefined;

  const navigation = [
    { id: 'projects' as const, label: '申请项目', icon: FolderKanban },
    { id: 'library' as const, label: '材料库', icon: FileStack },
    { id: 'templates' as const, label: '模板', icon: LayoutTemplate },
  ];

  $: recentProjects = [...projects]
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
    .slice(0, collapsed ? 0 : 5);
</script>

<aside class:collapsed class="sidebar">
  <div class="brand-row">
    <div class="brand-mark" aria-hidden="true"><GraduationCap size={20} strokeWidth={2.2} /></div>
    {#if !collapsed}
      <div class="brand-copy">
        <strong>保研材料助手</strong>
        <span>PDF 编排与合并</span>
      </div>
    {/if}
    <button
      class="icon-button sidebar-toggle"
      type="button"
      aria-label={collapsed ? '展开侧栏' : '收起侧栏'}
      title={collapsed ? '展开侧栏' : '收起侧栏'}
      on:click={onToggle}
    >
      {#if collapsed}<PanelLeftOpen size={18} />{:else}<PanelLeftClose size={18} />{/if}
    </button>
  </div>

  <nav class="primary-nav" aria-label="主要功能">
    {#each navigation as item}
      <button
        class:active={currentPage === item.id && !currentProjectId}
        class="nav-item"
        type="button"
        title={collapsed ? item.label : undefined}
        on:click={() => onNavigate(item.id)}
      >
        <svelte:component this={item.icon} size={18} />
        {#if !collapsed}<span>{item.label}</span>{/if}
      </button>
    {/each}
  </nav>

  {#if !collapsed && recentProjects.length > 0}
    <div class="recent-section">
      <div class="sidebar-label">最近项目</div>
      <div class="recent-list">
        {#each recentProjects as project}
          <button
            class:active={currentProjectId === project.id}
            class="recent-project"
            type="button"
            on:click={() => onOpenProject(project.id)}
          >
            <span class="recent-dot" aria-hidden="true"></span>
            <span>{projectDisplayName(project)}</span>
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <div class="sidebar-spacer"></div>
  <div class="sidebar-footer">
    <button
      class:active={currentPage === 'settings'}
      class="nav-item"
      type="button"
      title={collapsed ? '设置' : undefined}
      on:click={() => onNavigate('settings')}
    >
      <Settings size={18} />
      {#if !collapsed}<span>设置</span>{/if}
    </button>
    {#if !collapsed}
      <div class="local-note"><HardDrive size={14} /> 文件仅在本机处理</div>
    {/if}
  </div>
</aside>
