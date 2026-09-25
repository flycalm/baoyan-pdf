<script lang="ts">
  import Modal from './Modal.svelte';

  export let open = false;
  export let busy = false;
  export let onClose: () => void = () => undefined;
  export let onCreate: (title: string) => void | Promise<void> = () => undefined;

  let title = '其他材料';
  let wasOpen = false;

  $: if (open && !wasOpen) title = '其他材料';
  $: wasOpen = open;

  async function submit() {
    const normalized = title.trim();
    if (!normalized || busy) return;
    await onCreate(normalized);
  }
</script>

<Modal
  {open}
  title="添加材料模块"
  description="模块名称会显示在最终 PDF 的材料顺序中。"
  closeOnBackdrop={!busy}
  onClose={onClose}
  width="440px"
>
  <form class="form-stack" on:submit|preventDefault={submit}>
    <label class="field">
      <span>模块名称</span>
      <input bind:value={title} maxlength="60" placeholder="例如：推荐信" />
    </label>
  </form>
  <svelte:fragment slot="footer">
    <button class="button secondary" type="button" disabled={busy} on:click={onClose}>取消</button>
    <button class="button primary" type="button" disabled={busy || !title.trim()} on:click={submit}>
      {busy ? '正在添加…' : '添加模块'}
    </button>
  </svelte:fragment>
</Modal>
