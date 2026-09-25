<script lang="ts">
  import { CheckCircle2, AlertTriangle, Info, X, XCircle } from '@lucide/svelte';
  import type { ToastMessage } from '../lib/types';

  export let toasts: ToastMessage[] = [];
  export let onDismiss: (id: number) => void = () => undefined;
</script>

<div class="toast-region" aria-live="polite" aria-relevant="additions removals">
  {#each toasts as toast (toast.id)}
    <article class="toast toast-{toast.tone}">
      <div class="toast-icon" aria-hidden="true">
        {#if toast.tone === 'success'}
          <CheckCircle2 size={19} />
        {:else if toast.tone === 'warning'}
          <AlertTriangle size={19} />
        {:else if toast.tone === 'error'}
          <XCircle size={19} />
        {:else}
          <Info size={19} />
        {/if}
      </div>
      <div class="toast-copy">
        <strong>{toast.title}</strong>
        {#if toast.message}<p>{toast.message}</p>{/if}
        {#if toast.actionLabel && toast.onAction}
          <button class="link-button" type="button" on:click={toast.onAction}>{toast.actionLabel}</button>
        {/if}
      </div>
      <button class="icon-button small" type="button" aria-label="关闭提示" on:click={() => onDismiss(toast.id)}>
        <X size={16} />
      </button>
    </article>
  {/each}
</div>
