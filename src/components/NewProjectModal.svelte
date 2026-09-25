<script lang="ts">
  import { CalendarDays, Link2 } from '@lucide/svelte';
  import type { CreateProjectInput, ProjectTemplate } from '../lib/types';
  import Modal from './Modal.svelte';

  export let open = false;
  export let templates: ProjectTemplate[] = [];
  export let initialTemplateId = '';
  export let busy = false;
  export let onClose: () => void = () => undefined;
  export let onCreate: (input: CreateProjectInput) => void | Promise<void> = () => undefined;

  let school = '';
  let department = '';
  let program = '';
  let noticeUrl = '';
  let deadline = '';
  let sizeLimitMb: number | undefined;
  let notes = '';
  let templateId = '';
  let submitted = false;
  let wasOpen = false;

  $: if (open && !wasOpen) {
    submitted = false;
    templateId = initialTemplateId;
  }
  $: wasOpen = open;
  $: isValid = school.trim().length > 0 || department.trim().length > 0 || program.trim().length > 0;

  async function submit() {
    submitted = true;
    if (!isValid || busy) return;
    await onCreate({
      school: school.trim(),
      department: department.trim(),
      program: program.trim(),
      noticeUrl: noticeUrl.trim(),
      deadline: deadline ? new Date(deadline).toISOString() : null,
      sizeLimitMb: sizeLimitMb && sizeLimitMb > 0 ? sizeLimitMb : null,
      notes: notes.trim(),
      templateId: templateId || null,
    });
    school = '';
    department = '';
    program = '';
    noticeUrl = '';
    deadline = '';
    sizeLimitMb = undefined;
    notes = '';
    templateId = '';
  }
</script>

<Modal
  {open}
  title="新建申请项目"
  description="记录院校信息，然后选择并排列本次需要的材料。"
  closeOnBackdrop={!busy}
  onClose={onClose}
  width="620px"
>
  <form class="form-grid" on:submit|preventDefault={submit}>
    <label class="field span-2">
      <span>学校</span>
      <input bind:value={school} placeholder="例如：南方科技大学" />
      {#if submitted && !isValid}<small class="field-error">学校、学院或项目名称至少填写一项。</small>{/if}
    </label>
    <label class="field">
      <span>学院 / 院系</span>
      <input bind:value={department} placeholder="计算机科学与工程系" />
    </label>
    <label class="field">
      <span>项目名称</span>
      <input bind:value={program} placeholder="2027年预推免" />
    </label>
    <label class="field">
      <span><CalendarDays size={14} /> 截止时间</span>
      <input type="datetime-local" bind:value={deadline} />
    </label>
    <label class="field">
      <span>PDF 大小限制（MB，可选）</span>
      <input type="number" min="1" max="1024" step="0.1" bind:value={sizeLimitMb} placeholder="留空表示不限" />
    </label>
    <label class="field span-2">
      <span><Link2 size={14} /> 官方通知链接</span>
      <input type="url" bind:value={noticeUrl} placeholder="https://..." />
    </label>
    <label class="field span-2">
      <span>从模板开始</span>
      <select bind:value={templateId}>
        <option value="">空白项目</option>
        {#each templates as template}
          <option value={template.id}>{template.name}</option>
        {/each}
      </select>
    </label>
    <label class="field span-2">
      <span>备注</span>
      <textarea bind:value={notes} rows="3" placeholder="导师志愿、提交方式或其他提醒"></textarea>
    </label>
  </form>
  <svelte:fragment slot="footer">
    <button class="button secondary" type="button" disabled={busy} on:click={onClose}>取消</button>
    <button class="button primary" type="button" disabled={busy || (submitted && !isValid)} on:click={submit}>
      {busy ? '正在创建…' : '创建项目'}
    </button>
  </svelte:fragment>
</Modal>
