<script lang="ts">
  import { tick } from 'svelte';
  import { X } from '@lucide/svelte';

  export let open = false;
  export let title = '';
  export let description = '';
  export let width = '560px';
  export let closeOnBackdrop = true;
  export let onClose: () => void = () => undefined;

  let modalCard: HTMLDivElement;
  let previouslyFocused: HTMLElement | null = null;
  let wasOpen = false;

  const focusableSelector = [
    'a[href]',
    'button:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ].join(',');

  $: if (open && !wasOpen) {
    wasOpen = true;
    previouslyFocused = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    void focusInitialControl();
  }

  $: if (!open && wasOpen) {
    wasOpen = false;
    void restorePreviousFocus();
  }

  async function focusInitialControl() {
    await tick();
    if (!open || !modalCard) return;
    const preferred = modalCard.querySelector<HTMLElement>([
      '.modal-body input:not([disabled])',
      '.modal-body select:not([disabled])',
      '.modal-body textarea:not([disabled])',
      '.modal-body button:not([disabled])',
      '.modal-footer button:not([disabled])',
    ].join(','));
    (preferred || modalCard).focus();
  }

  async function restorePreviousFocus() {
    const target = previouslyFocused;
    previouslyFocused = null;
    await tick();
    if (target?.isConnected) target.focus();
  }

  function backdropClick(event: MouseEvent) {
    if (closeOnBackdrop && event.target === event.currentTarget) onClose();
  }

  function keydown(event: KeyboardEvent) {
    if (!open) return;
    if (event.key === 'Escape') {
      if (!closeOnBackdrop) return;
      event.preventDefault();
      event.stopPropagation();
      onClose();
      return;
    }
    if (event.key !== 'Tab' || !modalCard) return;

    const focusable = [...modalCard.querySelectorAll<HTMLElement>(focusableSelector)]
      .filter((element) => element.getClientRects().length > 0);
    if (focusable.length === 0) {
      event.preventDefault();
      modalCard.focus();
      return;
    }

    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    const active = document.activeElement;
    if (!modalCard.contains(active)) {
      event.preventDefault();
      (event.shiftKey ? last : first).focus();
    } else if (event.shiftKey && active === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && active === last) {
      event.preventDefault();
      first.focus();
    }
  }
</script>

{#if open}
  <div class="modal-backdrop" role="presentation" on:click={backdropClick}>
    <div
      bind:this={modalCard}
      class="modal-card"
      style:--modal-width={width}
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
      tabindex="-1"
      on:keydown={keydown}
    >
      <header class="modal-header">
        <div>
          <h2 id="modal-title">{title}</h2>
          {#if description}<p>{description}</p>{/if}
        </div>
        <button class="icon-button" type="button" aria-label="关闭" title="关闭" disabled={!closeOnBackdrop} on:click={onClose}>
          <X size={18} />
        </button>
      </header>
      <div class="modal-body">
        <slot />
      </div>
      {#if $$slots.footer}
        <footer class="modal-footer"><slot name="footer" /></footer>
      {/if}
    </div>
  </div>
{/if}
