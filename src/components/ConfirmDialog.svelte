<script lang="ts">
  import Modal from './Modal.svelte';

  export let open = false;
  export let title = '确认操作';
  export let message = '';
  export let confirmLabel = '确认';
  export let cancelLabel = '取消';
  export let destructive = false;
  export let busy = false;
  export let onConfirm: () => void | Promise<void> = () => undefined;
  export let onCancel: () => void = () => undefined;
</script>

<Modal {open} {title} closeOnBackdrop={!busy} onClose={onCancel} width="440px">
  <p class="confirm-message">{message}</p>
  <svelte:fragment slot="footer">
    <button class="button secondary" type="button" disabled={busy} on:click={onCancel}>{cancelLabel}</button>
    <button class:danger={destructive} class="button primary" type="button" disabled={busy} on:click={onConfirm}>
      {busy ? '处理中…' : confirmLabel}
    </button>
  </svelte:fragment>
</Modal>

