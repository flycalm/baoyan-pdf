<script lang="ts">
  import { onDestroy, tick } from 'svelte';
  import {
    AlertCircle,
    ArrowDown,
    ArrowUp,
    Check,
    CircleHelp,
    FileSearch,
    Link2,
    LoaderCircle,
    Sparkles,
  } from '@lucide/svelte';
  import Modal from './Modal.svelte';
  import {
    getAiApplyConfirmationItems,
    matchAiRequirements,
    type AiRequirementMatch,
  } from '../lib/aiMatching';
  import type { AiRegistrationAnalysis, Material } from '../lib/types';

  export let open = false;
  export let projectNoticeUrl = '';
  export let materials: Material[] = [];
  export let onAnalyze: (input: { noticeUrl: string; noticeText: string; requestId: string }) => Promise<AiRegistrationAnalysis>;
  export let onCancelAnalysis: (requestId: string) => Promise<boolean> = async () => false;
  export let onApply: (matches: AiRequirementMatch[]) => void | Promise<void> = () => undefined;
  export let onClose: () => void = () => undefined;

  let wasOpen = false;
  let inputMode: 'url' | 'text' = 'url';
  let noticeUrl = '';
  let noticeText = '';
  let analysis: AiRegistrationAnalysis | null = null;
  let matches: AiRequirementMatch[] = [];
  let analyzing = false;
  let applying = false;
  let progressStep = 0;
  let errorMessage = '';
  let confirmingApply = false;
  let cancelling = false;
  let activeRequestId: string | null = null;
  let confirmationPanel: HTMLDivElement | null = null;
  let progressTimer: ReturnType<typeof setInterval> | null = null;

  const progressLabels = ['准备通知内容', '第 1 轮：读取通知与附件线索', '第 2 轮：复核材料要求', '整理材料清单'];

  $: if (open && !wasOpen) {
    wasOpen = true;
    reset(projectNoticeUrl);
  }
  $: if (!open && wasOpen) {
    wasOpen = false;
    stopProgress();
  }
  $: includedCount = matches.filter((match) => match.included).length;
  $: missingRequired = matches.filter((match) => match.included && match.requirement.required && match.selectedMaterialIds.length === 0).length;
  $: selectedCount = matches.reduce((count, match) => count + (match.included ? match.selectedMaterialIds.length : 0), 0);
  $: roundsUsed = analysis?.toolLog.reduce((maximum, entry) => Math.max(maximum, entry.round), 0) || 0;
  $: confirmationItems = getAiApplyConfirmationItems(matches);

  function reset(initialUrl: string) {
    inputMode = initialUrl.trim() ? 'url' : 'text';
    noticeUrl = initialUrl;
    noticeText = '';
    analysis = null;
    matches = [];
    analyzing = false;
    applying = false;
    progressStep = 0;
    errorMessage = '';
    confirmingApply = false;
    cancelling = false;
    activeRequestId = null;
    stopProgress();
  }

  function stopProgress() {
    if (progressTimer) clearInterval(progressTimer);
    progressTimer = null;
  }

  async function requestClose() {
    if (applying || cancelling) return;
    if (analyzing) {
      await cancelAnalysis(true);
      return;
    }
    onClose();
  }

  async function cancelAnalysis(closeAfter = false) {
    const requestId = activeRequestId;
    activeRequestId = null;
    stopProgress();
    errorMessage = '';
    cancelling = true;
    try {
      if (requestId) await onCancelAnalysis(requestId);
    } catch {
      // Cancellation is best effort; the late result is ignored by requestId either way.
    } finally {
      analyzing = false;
      cancelling = false;
      progressStep = 0;
      if (closeAfter) onClose();
    }
  }

  onDestroy(() => {
    const requestId = activeRequestId;
    activeRequestId = null;
    stopProgress();
    if (requestId) void onCancelAnalysis(requestId).catch(() => undefined);
  });

  async function analyze() {
    const url = inputMode === 'url' ? noticeUrl.trim() : '';
    const text = inputMode === 'text' ? noticeText.trim() : '';
    if (!url && !text) {
      errorMessage = inputMode === 'url' ? '请填写报名通知链接。' : '请粘贴报名通知正文。';
      return;
    }
    if (url) {
      try {
        const parsed = new URL(url);
        if (!['http:', 'https:'].includes(parsed.protocol)) throw new Error();
      } catch {
        errorMessage = '请输入以 http:// 或 https:// 开头的有效链接。';
        return;
      }
    }

    analyzing = true;
    const requestId = crypto.randomUUID();
    activeRequestId = requestId;
    analysis = null;
    matches = [];
    errorMessage = '';
    progressStep = 0;
    stopProgress();
    progressTimer = setInterval(() => {
      progressStep = Math.min(progressLabels.length - 1, progressStep + 1);
    }, 720);
    try {
      const result = await onAnalyze({ noticeUrl: url, noticeText: text, requestId });
      if (activeRequestId !== requestId) return;
      analysis = result;
      matches = matchAiRequirements(result.requirements, materials, result.evidence);
      confirmingApply = false;
      progressStep = progressLabels.length - 1;
    } catch (error) {
      if (activeRequestId !== requestId) return;
      errorMessage = error instanceof Error ? error.message : String(error || '分析失败，请重试。');
    } finally {
      if (activeRequestId === requestId) {
        activeRequestId = null;
        stopProgress();
        analyzing = false;
      }
    }
  }

  function toggleCandidate(matchId: string, materialId: string, checked: boolean) {
    matches = matches.map((match) =>
      match.id === matchId
        ? {
            ...match,
            selectedMaterialIds: checked
              ? match.selectionLimit !== null && match.selectedMaterialIds.length >= match.selectionLimit
                ? match.selectedMaterialIds
                : [...new Set([...match.selectedMaterialIds, materialId])]
              : match.selectedMaterialIds.filter((id) => id !== materialId),
          }
        : match,
    );
    confirmingApply = false;
  }

  function moveRequirement(index: number, direction: -1 | 1) {
    const target = index + direction;
    if (target < 0 || target >= matches.length) return;
    const next = [...matches];
    [next[index], next[target]] = [next[target], next[index]];
    matches = next;
    confirmingApply = false;
  }

  function toggleIncluded(matchId: string) {
    matches = matches.map((match) =>
      match.id === matchId ? { ...match, included: !match.included } : match,
    );
    confirmingApply = false;
  }

  function evidenceFor(match: AiRequirementMatch): string[] {
    if (!analysis) return [];
    const ids = new Set(match.requirement.evidenceIds);
    return analysis.evidence.filter((entry) => ids.has(entry.id)).map((entry) => entry.quote);
  }

  async function applyArrangement() {
    if (confirmationItems.length > 0 && !confirmingApply) {
      confirmingApply = true;
      await tick();
      confirmationPanel?.scrollIntoView?.({ behavior: 'smooth', block: 'nearest' });
      return;
    }
    applying = true;
    errorMessage = '';
    try {
      await onApply(matches);
      onClose();
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error || '无法应用 AI 编排。');
    } finally {
      applying = false;
    }
  }
</script>

<Modal
  {open}
  title="AI 整理报名材料"
  description="读取通知、提取材料要求，并只在本地匹配你的材料库。"
  width="860px"
  closeOnBackdrop={!applying && !cancelling}
  onClose={requestClose}
>
  {#if !analysis && !analyzing}
    <div class="ai-intro">
      <div class="privacy-note"><Sparkles size={18} /><span><strong>个人 PDF 不会发送给模型</strong><small>会发送通知链接、网页正文及最多 3 个补充页面或附件的提取文本；材料库仅在本机按名称和分类匹配。</small></span></div>
      <div class="input-tabs" role="tablist" aria-label="通知输入方式">
        <button class:active={inputMode === 'url'} type="button" role="tab" aria-selected={inputMode === 'url'} on:click={() => (inputMode = 'url')}><Link2 size={15} /> 通知链接</button>
        <button class:active={inputMode === 'text'} type="button" role="tab" aria-selected={inputMode === 'text'} on:click={() => (inputMode = 'text')}><FileSearch size={15} /> 粘贴通知正文</button>
      </div>
      {#if inputMode === 'url'}
        <label class="field">
          <span>报名通知链接</span>
          <input type="url" bind:value={noticeUrl} placeholder="https://学校官网/通知页面" on:input={() => (errorMessage = '')} />
          <small>AI 最多读取 2 轮：先查找材料清单，再复核附件或上下文。</small>
        </label>
      {:else}
        <label class="field">
          <span>报名通知正文</span>
          <textarea rows="9" bind:value={noticeText} placeholder="粘贴通知中关于报名材料、提交顺序和格式要求的文字……" on:input={() => (errorMessage = '')}></textarea>
          <small>适用于网页无法访问、需要登录或你只想分析部分内容的情况。</small>
        </label>
      {/if}
    </div>
  {:else if analyzing}
    <div class="analysis-progress" aria-live="polite">
      <div class="analysis-orbit"><Sparkles size={25} /><span></span></div>
      <h3>正在分析报名通知</h3>
      <p>{progressLabels[progressStep]}</p>
      <div class="progress-track"><span style:width={`${Math.max(12, ((progressStep + 1) / progressLabels.length) * 100)}%`}></span></div>
      <ol>
        {#each progressLabels as label, index}
          <li class:done={index < progressStep} class:active={index === progressStep}>
            {#if index < progressStep}<Check size={14} />{:else if index === progressStep}<LoaderCircle class="spin" size={14} />{:else}<span>{index + 1}</span>{/if}
            {label}
          </li>
        {/each}
      </ol>
    </div>
  {:else if analysis}
    <div class="analysis-review">
      <div class="review-summary">
        <div><Sparkles size={18} /><span><strong>已整理 {matches.length} 项材料要求</strong><small>{selectedCount} 项已选材料，{missingRequired} 项必需材料待补</small></span></div>
        <span class="confidence">置信度 {Math.round(analysis.confidence * 100)}%</span>
      </div>
      <p class="analysis-summary">{analysis.summary}</p>

      {#if analysis.warnings.length > 0}
        <div class="analysis-warning"><AlertCircle size={16} /><span>{analysis.warnings.join('；')}</span></div>
      {/if}

      <div class="read-budget">
        <div><FileSearch size={16} /><span><strong>读取预算：最多 2 轮</strong><small>本次使用 {roundsUsed} 轮 · {analysis.source.length} 个来源</small></span></div>
        <details>
          <summary>查看读取来源</summary>
          <div class="source-list">
            {#each analysis.toolLog as log}
              <div><span class="round-chip">第 {log.round} 轮</span><span><strong>{log.url || '粘贴的通知正文'}</strong><small>{log.message || `${log.status}${log.extractedChars ? ` · 提取 ${log.extractedChars} 字` : ''}`}</small></span></div>
            {:else}
              {#each analysis.source as source}
                <div><span class="round-chip">来源</span><span><strong>{source.title || source.url}</strong><small>{source.status}</small></span></div>
              {/each}
            {/each}
          </div>
        </details>
      </div>

      <div class="requirement-list" aria-label="AI 提取的材料清单">
        {#each matches as match, index (match.id)}
          {@const quotes = evidenceFor(match)}
          <article class="requirement-row" class:ignored={!match.included} class:missing={match.included && match.status === 'missing'} class:needs-confirmation={match.included && match.status === 'multiple'}>
            <div class="requirement-order">
              <span>{index + 1}</span>
              <button class="mini-icon" type="button" title="上移" aria-label={`上移${match.requirement.name}`} disabled={index === 0} on:click={() => moveRequirement(index, -1)}><ArrowUp size={14} /></button>
              <button class="mini-icon" type="button" title="下移" aria-label={`下移${match.requirement.name}`} disabled={index === matches.length - 1} on:click={() => moveRequirement(index, 1)}><ArrowDown size={14} /></button>
            </div>
            <div class="requirement-content">
              <div class="requirement-title">
                <strong>{match.requirement.name}</strong>
                <span class:optional={!match.requirement.required} class="requirement-kind">{match.requirement.required ? '必需' : '选交'}</span>
                {#if match.included}
                  <span class:matched={match.status === 'matched'} class:multiple={match.status === 'multiple'} class:missing={match.status === 'missing'} class="match-chip">
                    {#if match.status === 'matched'}<Check size={13} /> 已匹配{:else if match.status === 'multiple'}<CircleHelp size={13} /> 需确认 · 多个候选{:else}<AlertCircle size={13} /> 暂无材料{/if}
                  </span>
                {:else}<span class="ignored-chip">已忽略</span>{/if}
                <button class="ignore-requirement" type="button" on:click={() => toggleIncluded(match.id)}>{match.included ? '忽略此项' : '恢复此项'}</button>
              </div>
              <p>{match.requirement.details}</p>
              {#if match.unverifiedLimitClaim}<div class="unverified-limit"><AlertCircle size={13} /> AI 说明中提到数量限制，但通知依据未证实；不会据此自动删减，请核对原文。</div>{/if}
              {#if quotes.length > 0}<blockquote>通知依据：{quotes.join('；')}</blockquote>{/if}
              {#if !match.included}
                <div class="ignored-hint">此项不会生成模块，也不参与缺失检查或材料计数。</div>
              {:else if match.candidates.length > 0}
                <fieldset class="candidate-select">
                  <legend>使用本地材料（可多选）{match.selectionLimit !== null ? ` · 通知上限 ${match.selectionLimit} 份` : ''}</legend>
                  {#each match.candidates as material}
                    {@const selected = match.selectedMaterialIds.includes(material.id)}
                    <label>
                      <input
                        type="checkbox"
                        checked={selected}
                        disabled={!selected && match.selectionLimit !== null && match.selectedMaterialIds.length >= match.selectionLimit}
                        on:change={(event) => toggleCandidate(match.id, material.id, (event.currentTarget as HTMLInputElement).checked)}
                      />
                      <span><strong>{material.name}</strong><small>{material.originalName}</small></span>
                    </label>
                  {/each}
                </fieldset>
                {#if match.status === 'multiple'}<div class="multiple-confirm-hint"><CircleHelp size={13} /> AI 无法替你决定使用哪些候选，应用前需要再次确认。</div>{/if}
              {:else}
                <div class="missing-hint">应用后会保留空模块，你可以稍后导入或从材料库补充。</div>
              {/if}
            </div>
          </article>
        {/each}
      </div>
      <div class="apply-impact"><AlertCircle size={15} /><span><strong>应用影响</strong>AI 清单会排在最前；当前项目中未被选中的旧材料和空模块会保留在末尾，但默认关闭、不参与导出。应用后仍可一键撤销。</span></div>
      {#if confirmingApply && confirmationItems.length > 0}
        <div bind:this={confirmationPanel} class="apply-confirmation" role="alert">
          <div><AlertCircle size={17} /><span><strong>应用前请再次确认</strong><small>以下情况需要你本人判断。再次点击“确认并应用”后才会保存。</small></span></div>
          <ul>{#each confirmationItems as item}<li class:danger={item.kind === 'missing-required' || item.message.includes('超过')}>{item.message}</li>{/each}</ul>
        </div>
      {/if}
    </div>
  {/if}

  {#if errorMessage}<div class="ai-error" role="alert"><AlertCircle size={15} /> {errorMessage}</div>{/if}

  <svelte:fragment slot="footer">
    {#if analysis}
      <button class="button secondary footer-reset" type="button" disabled={applying} on:click={() => reset(projectNoticeUrl)}>重新分析</button>
      <span class="footer-note">应用后可一键撤销</span>
      <button class="button secondary" type="button" disabled={applying} on:click={requestClose}>取消</button>
      <button class="button primary" type="button" disabled={applying || includedCount === 0} title={includedCount === 0 ? '请至少恢复一项材料要求' : ''} on:click={applyArrangement}>
        {#if applying}<LoaderCircle class="spin" size={16} /> 正在应用…{:else if confirmingApply}<Check size={16} /> 确认并应用{:else}<Sparkles size={16} /> 应用到当前项目{/if}
      </button>
    {:else}
      <button class="button secondary" type="button" disabled={cancelling} on:click={() => analyzing ? cancelAnalysis(false) : requestClose()}>
        {analyzing ? (cancelling ? '正在取消…' : '取消分析') : '取消'}
      </button>
      <button class="button primary" type="button" disabled={analyzing} on:click={analyze}>
        {#if analyzing}<LoaderCircle class="spin" size={16} /> 分析中…{:else}<Sparkles size={16} /> 开始整理{/if}
      </button>
    {/if}
  </svelte:fragment>
</Modal>

<style>
  .ai-intro,
  .analysis-review {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .privacy-note,
  .review-summary,
  .read-budget > div,
  .source-list > div {
    display: flex;
    align-items: flex-start;
    gap: 10px;
  }

  .privacy-note {
    padding: 12px 14px;
    border: 1px solid #cfe0f4;
    border-radius: 9px;
    color: #235b94;
    background: #f3f8fe;
  }

  .privacy-note span,
  .review-summary span,
  .read-budget span,
  .source-list span {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 2px;
  }

  .privacy-note small,
  .review-summary small,
  .read-budget small,
  .source-list small,
  .field small {
    color: var(--text-soft);
    font-size: 11px;
  }

  .input-tabs {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px;
    padding: 4px;
    border-radius: 8px;
    background: var(--surface-muted);
  }

  .input-tabs button {
    display: inline-flex;
    min-height: 34px;
    align-items: center;
    justify-content: center;
    gap: 7px;
    border: 0;
    border-radius: 6px;
    color: var(--text-soft);
    background: transparent;
  }

  .input-tabs button.active {
    color: var(--primary);
    background: var(--surface);
    box-shadow: var(--shadow-sm);
  }

  .analysis-progress {
    display: flex;
    min-height: 350px;
    align-items: center;
    flex-direction: column;
    justify-content: center;
    padding: 28px;
    text-align: center;
  }

  .analysis-orbit {
    position: relative;
    display: grid;
    width: 58px;
    height: 58px;
    place-items: center;
    margin-bottom: 14px;
    border-radius: 50%;
    color: var(--primary);
    background: var(--primary-soft);
  }

  .analysis-orbit span {
    position: absolute;
    inset: -4px;
    border: 2px solid transparent;
    border-top-color: var(--primary);
    border-radius: 50%;
    animation: spin 900ms linear infinite;
  }

  .analysis-progress h3 { font-size: 16px; }
  .analysis-progress > p { margin-top: 5px; color: var(--text-soft); font-size: 12px; }
  .progress-track { width: min(420px, 100%); height: 5px; margin: 18px 0; overflow: hidden; border-radius: 999px; background: var(--surface-muted); }
  .progress-track span { display: block; height: 100%; border-radius: inherit; background: var(--primary); transition: width 240ms ease; }
  .analysis-progress ol { display: grid; width: min(420px, 100%); gap: 8px; margin: 0; padding: 0; list-style: none; text-align: left; }
  .analysis-progress li { display: flex; align-items: center; gap: 8px; color: #9aa5b4; font-size: 12px; }
  .analysis-progress li > span { display: grid; width: 14px; height: 14px; place-items: center; border: 1px solid #c8d0db; border-radius: 50%; font-size: 9px; }
  .analysis-progress li.active { color: var(--primary); font-weight: 600; }
  .analysis-progress li.done { color: var(--success); }

  .review-summary {
    align-items: center;
    justify-content: space-between;
    padding: 11px 13px;
    border: 1px solid #cfe0f4;
    border-radius: 9px;
    color: #235b94;
    background: #f3f8fe;
  }

  .review-summary > div { display: flex; align-items: flex-start; gap: 9px; }
  .confidence { flex: 0 0 auto; padding: 4px 7px; border-radius: 999px; background: rgba(43, 108, 176, 0.1); font-size: 11px; }
  .analysis-summary { color: var(--text-soft); font-size: 12px; line-height: 1.65; }
  .analysis-warning,
  .ai-error { display: flex; align-items: flex-start; gap: 7px; padding: 9px 11px; border-radius: 7px; color: #8b5a14; background: #fff8e8; font-size: 12px; }
  .ai-error { margin-top: 13px; color: var(--danger); background: #fff1f1; }

  .read-budget {
    padding: 11px 13px;
    border: 1px solid var(--border-soft);
    border-radius: 8px;
    background: #fbfcfe;
  }
  .read-budget details { margin-top: 8px; padding-left: 26px; }
  .read-budget summary { cursor: pointer; color: var(--primary); font-size: 11px; }
  .source-list { display: grid; gap: 7px; margin-top: 8px; }
  .source-list > div { min-width: 0; align-items: center; }
  .source-list strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 11px; }
  .round-chip { flex: 0 0 auto; padding: 3px 6px; border-radius: 5px; color: #526075; background: var(--surface-muted); font-size: 10px; }

  .requirement-list { display: grid; gap: 9px; }
  .requirement-row { display: grid; grid-template-columns: 38px minmax(0, 1fr); border: 1px solid var(--border-soft); border-radius: 9px; background: var(--surface); }
  .requirement-row.missing { border-color: #efd6a6; }
  .requirement-row.needs-confirmation { border-color: #e5bd72; box-shadow: 0 0 0 1px rgba(196, 132, 34, 0.06); }
  .requirement-row.ignored { opacity: 0.62; background: #f7f8fa; }
  .requirement-order { display: flex; align-items: center; flex-direction: column; gap: 3px; padding: 11px 5px; border-right: 1px solid var(--border-soft); background: #fbfcfe; }
  .requirement-order > span { display: grid; width: 24px; height: 24px; place-items: center; border-radius: 50%; color: var(--primary); background: var(--primary-soft); font-size: 11px; font-weight: 700; }
  .mini-icon { display: grid; width: 24px; height: 22px; place-items: center; border: 0; border-radius: 5px; color: var(--text-soft); background: transparent; }
  .mini-icon:hover:not(:disabled) { color: var(--primary); background: var(--primary-soft); }
  .mini-icon:disabled { opacity: 0.28; }
  .requirement-content { min-width: 0; padding: 11px 12px; }
  .requirement-title { display: flex; align-items: center; gap: 7px; }
  .requirement-title > strong { margin-right: auto; font-size: 13px; }
  .requirement-kind,
  .match-chip { display: inline-flex; align-items: center; gap: 3px; padding: 3px 6px; border-radius: 999px; color: #a34a3e; background: #fff0ee; font-size: 10px; font-weight: 600; }
  .requirement-kind.optional { color: #64748b; background: #eef2f7; }
  .match-chip.matched { color: #237451; background: #eaf7f1; }
  .match-chip.multiple { color: #8b5a14; background: #fff6df; }
  .match-chip.missing { color: #a34a3e; background: #fff0ee; }
  .ignored-chip { display: inline-flex; padding: 3px 6px; border-radius: 999px; color: #64748b; background: #e9edf2; font-size: 10px; font-weight: 600; }
  .ignore-requirement { flex: 0 0 auto; padding: 3px 6px; border: 0; border-radius: 5px; color: var(--primary); background: transparent; font-size: 10px; }
  .ignore-requirement:hover { background: var(--primary-soft); }
  .requirement-content > p { margin-top: 5px; color: var(--text-soft); font-size: 11px; line-height: 1.5; }
  .unverified-limit { display: flex; align-items: flex-start; gap: 5px; margin-top: 6px; color: #8b5a14; font-size: 10px; line-height: 1.4; }
  .requirement-content blockquote { margin: 7px 0 0; padding: 6px 8px; border-left: 2px solid #bed3ec; color: #65758a; background: #f7fafd; font-size: 10px; line-height: 1.45; }
  .candidate-select { display: grid; gap: 5px; margin: 9px 0 0; padding: 0; border: 0; }
  .candidate-select legend { margin-bottom: 4px; color: #65758a; font-size: 10px; font-weight: 600; }
  .candidate-select label { display: flex; min-width: 0; align-items: center; gap: 7px; padding: 6px 8px; border: 1px solid var(--border-soft); border-radius: 6px; background: #fbfcfe; cursor: pointer; }
  .candidate-select label:has(input:checked) { border-color: #bed3ec; background: #f5f9fe; }
  .candidate-select input { flex: 0 0 auto; accent-color: var(--primary); }
  .candidate-select label > span { display: flex; min-width: 0; flex-direction: column; gap: 1px; }
  .candidate-select label strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 11px; }
  .candidate-select label small { overflow: hidden; color: var(--text-soft); text-overflow: ellipsis; white-space: nowrap; font-size: 9px; }
  .multiple-confirm-hint { display: flex; align-items: center; gap: 5px; margin-top: 6px; color: #8b5a14; font-size: 10px; }
  .ignored-hint { margin-top: 8px; color: #64748b; font-size: 10px; font-weight: 600; }
  .missing-hint { margin-top: 8px; color: #9a671c; font-size: 10px; }
  .apply-impact { display: flex; align-items: flex-start; gap: 7px; padding: 9px 11px; border: 1px solid var(--border-soft); border-radius: 7px; color: var(--text-soft); background: #fbfcfe; font-size: 10px; line-height: 1.5; }
  .apply-impact strong { display: block; margin-bottom: 1px; color: var(--text); font-size: 11px; }
  .apply-confirmation { padding: 11px 12px; border: 1px solid #e0b35e; border-radius: 8px; color: #76501b; background: #fff9e9; }
  .apply-confirmation > div { display: flex; align-items: flex-start; gap: 7px; }
  .apply-confirmation > div span { display: flex; flex-direction: column; gap: 2px; }
  .apply-confirmation strong { font-size: 12px; }
  .apply-confirmation small { color: #8b6a3d; font-size: 10px; }
  .apply-confirmation ul { display: grid; gap: 4px; margin: 8px 0 0 24px; padding: 0; font-size: 10px; line-height: 1.45; }
  .apply-confirmation li.danger { color: #a34a3e; font-weight: 600; }

  :global(.modal-footer) .footer-reset { margin-right: auto; }
  .footer-note { margin-right: 4px; color: var(--text-soft); font-size: 11px; }

  @media (max-width: 720px) {
    .review-summary { align-items: flex-start; }
    .confidence { display: none; }
    .requirement-title { flex-wrap: wrap; }
    .requirement-title > strong { width: 100%; }
    .footer-note { display: none; }
  }
</style>
