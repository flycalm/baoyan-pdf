<script lang="ts">
  import type { Material } from '../lib/types';
  import Modal from './Modal.svelte';

  export let open = false;
  export let material: Material | null = null;
  export let busy = false;
  export let onClose: () => void = () => undefined;
  export let onRename: (name: string) => void | Promise<void> = () => undefined;

  let name = '';
  let previousMaterialId: string | null = null;

  $: if (material?.id !== previousMaterialId) {
    previousMaterialId = material?.id || null;
    name = material?.name || '';
  }
</script>

<Modal {open} title="重命名材料" description="只修改软件中的显示名称，不会改动 PDF 文件内容。" onClose={onClose} width="460px">
  <label class="field">
    <span>材料名称</span>
    <input bind:value={name} on:keydown={(event) => event.key === 'Enter' && name.trim() && onRename(name.trim())} />
  </label>
  <svelte:fragment slot="footer">
    <button class="button secondary" type="button" disabled={busy} on:click={onClose}>取消</button>
    <button class="button primary" type="button" disabled={busy || !name.trim()} on:click={() => onRename(name.trim())}>{busy ? '保存中…' : '保存名称'}</button>
  </svelte:fragment>
</Modal>
