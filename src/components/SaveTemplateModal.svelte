<script lang="ts">
  import Modal from './Modal.svelte';

  export let open = false;
  export let suggestedName = '';
  export let busy = false;
  export let onClose: () => void = () => undefined;
  export let onSave: (name: string, description: string) => void | Promise<void> = () => undefined;

  let name = '';
  let description = '';
  let wasOpen = false;

  $: if (open && !wasOpen) {
    name = suggestedName;
    description = '';
  }
  $: wasOpen = open;
</script>

<Modal {open} title="保存为模板" description="通用材料会随模板复用；本项目专用材料仅保存为占位项。" onClose={onClose} width="500px">
  <div class="form-stack">
    <label class="field"><span>模板名称</span><input bind:value={name} placeholder="例如：常规硕士推免" /></label>
    <label class="field"><span>说明</span><textarea bind:value={description} rows="3" placeholder="说明模板适用的报名类型"></textarea></label>
  </div>
  <svelte:fragment slot="footer">
    <button class="button secondary" type="button" disabled={busy} on:click={onClose}>取消</button>
    <button class="button primary" type="button" disabled={busy || !name.trim()} on:click={() => onSave(name.trim(), description.trim())}>{busy ? '正在保存…' : '保存模板'}</button>
  </svelte:fragment>
</Modal>
