<script lang="ts">
  import {
    Bot,
    CheckCircle2,
    Database,
    ExternalLink,
    Eye,
    EyeOff,
    FileCog,
    FolderOpen,
    Info,
    LoaderCircle,
    RotateCcw,
    TestTube2,
  } from '@lucide/svelte';
  import type {
    AiConnectionConfig,
    AiConnectionTestResult,
    AppSettings,
    AppSnapshot,
  } from '../lib/types';

  export let snapshot: AppSnapshot;
  export let onOpenDataDirectory: () => void = () => undefined;
  export let onChooseExportDirectory: () => void = () => undefined;
  export let onResetExportDirectory: () => void = () => undefined;
  export let onUpdateSettings: (settings: AppSettings) => void | Promise<void> = () => undefined;
  export let onTestAiConnection: (config: AiConnectionConfig) => Promise<AiConnectionTestResult>;

  let sourceSettings = snapshot.settings;
  let aiEnabled = snapshot.settings.aiEnabled;
  let aiBaseUrl = snapshot.settings.aiBaseUrl;
  let aiApiKey = snapshot.settings.aiApiKey;
  let aiModel = snapshot.settings.aiModel;
  let showApiKey = false;
  let aiDirty = false;
  let savingAi = false;
  let testingAi = false;
  let connectionResult: AiConnectionTestResult | null = null;
  let aiError = '';
  let clearKeyArmed = false;

  const AI_PROVIDER_PRESETS = [
    {
      id: 'zhipu',
      name: '智谱直连（推荐）',
      baseUrl: 'https://open.bigmodel.cn/api/paas/v4',
      description: '使用智谱官方接口',
    },
    {
      id: 'opencode',
      name: 'OpenCode Go',
      baseUrl: 'https://opencode.ai/zen/go/v1',
      description: '沿用当前 OpenCode 接口',
    },
  ] as const;

  $: if (snapshot.settings !== sourceSettings) {
    sourceSettings = snapshot.settings;
    if (!aiDirty) {
      aiEnabled = snapshot.settings.aiEnabled;
      aiBaseUrl = snapshot.settings.aiBaseUrl;
      aiApiKey = snapshot.settings.aiApiKey;
      aiModel = snapshot.settings.aiModel;
    }
  }
  $: aiConfigValid = Boolean(aiBaseUrl.trim() && aiApiKey.trim() && aiModel.trim());
  $: activePreset = AI_PROVIDER_PRESETS.find(
    (preset) => preset.baseUrl === aiBaseUrl.trim().replace(/\/$/, ''),
  )?.id || null;

  function markAiDirty() {
    aiDirty = true;
    clearKeyArmed = false;
    connectionResult = null;
    aiError = '';
  }

  function currentConfig(): AiConnectionConfig {
    return { apiKey: aiApiKey.trim(), baseUrl: aiBaseUrl.trim().replace(/\/$/, ''), model: aiModel.trim() };
  }

  function useProviderPreset(baseUrl: string) {
    aiBaseUrl = baseUrl;
    markAiDirty();
  }

  async function testConnection() {
    if (!aiConfigValid) {
      aiError = '请先填写 API 地址、API Key 和模型。';
      return;
    }
    testingAi = true;
    connectionResult = null;
    aiError = '';
    try {
      connectionResult = await onTestAiConnection(currentConfig());
      if (!connectionResult.ok) aiError = connectionResult.message;
    } catch (error) {
      aiError = error instanceof Error ? error.message : String(error || '连接测试失败。');
    } finally {
      testingAi = false;
    }
  }

  async function saveAiSettings() {
    if (aiEnabled && !aiConfigValid) {
      aiError = '启用 AI 前，请完整填写 API 地址、API Key 和模型。';
      return;
    }
    savingAi = true;
    aiError = '';
    try {
      await onUpdateSettings({
        ...snapshot.settings,
        aiEnabled,
        aiBaseUrl: aiBaseUrl.trim().replace(/\/$/, ''),
        aiApiKey: aiApiKey.trim(),
        aiModel: aiModel.trim() || 'glm-5.3-flash',
      });
      aiDirty = false;
    } catch (error) {
      aiError = error instanceof Error ? error.message : String(error || 'AI 设置保存失败。');
    } finally {
      savingAi = false;
    }
  }

  async function clearApiKey() {
    if (!aiApiKey || savingAi || testingAi) return;
    if (!clearKeyArmed) {
      clearKeyArmed = true;
      return;
    }
    aiApiKey = '';
    aiEnabled = false;
    aiDirty = true;
    clearKeyArmed = false;
    connectionResult = null;
    await saveAiSettings();
  }
</script>

<main class="page-shell narrow-page">
  <header class="page-header">
    <div>
      <p class="eyebrow">应用选项</p>
      <h1>设置</h1>
      <p class="page-description">管理 PDF、存储位置和可选的 AI 整理功能。</p>
    </div>
  </header>

  <section class="settings-section ai-settings-section">
    <div class="settings-heading ai-settings-heading">
      <Bot size={20} />
      <div><h2>AI 整理报名材料</h2><p>从报名通知提取材料要求，再在本机匹配你的材料库。</p></div>
      <label class="switch ai-enable-switch" title={aiEnabled ? 'AI 已启用' : 'AI 已停用'}>
        <input type="checkbox" bind:checked={aiEnabled} aria-label="启用 AI 整理" on:change={markAiDirty} />
        <span></span>
      </label>
    </div>

    <div class="ai-settings-body">
      <div class="ai-privacy-copy">
        <CheckCircle2 size={16} />
        <span><strong>个人 PDF 不会发送给模型</strong><small>会向模型发送通知链接、网页正文及最多 3 个补充页面或附件的提取文本；不会发送材料库中的个人 PDF，本地仅用名称和分类匹配。</small></span>
      </div>
      <div class="provider-picker">
        <span>接口预设</span>
        <div role="group" aria-label="AI 接口预设">
          {#each AI_PROVIDER_PRESETS as preset}
            <button
              class:active={activePreset === preset.id}
              type="button"
              disabled={savingAi || testingAi}
              aria-pressed={activePreset === preset.id}
              on:click={() => useProviderPreset(preset.baseUrl)}
            ><strong>{preset.name}</strong><small>{preset.description}</small></button>
          {/each}
        </div>
        {#if activePreset === 'opencode'}<small>OpenCode 若遇到额度限制，可切换到“智谱直连”并填写对应密钥。</small>{/if}
      </div>
      <div class="ai-settings-grid">
        <label class="field ai-base-url">
          <span>API 地址</span>
          <input type="url" bind:value={aiBaseUrl} placeholder="https://open.bigmodel.cn/api/paas/v4" on:input={markAiDirty} />
          <small>应用会在此基础地址调用 /chat/completions。</small>
        </label>
        <label class="field">
          <span>模型</span>
          <input bind:value={aiModel} placeholder="glm-5.3-flash" on:input={markAiDirty} />
        </label>
        <label class="field ai-api-key">
          <span>API Key</span>
          <div class="secret-input">
            <input type={showApiKey ? 'text' : 'password'} bind:value={aiApiKey} autocomplete="off" placeholder="输入 API Key" on:input={markAiDirty} />
            <button type="button" aria-label={showApiKey ? '隐藏 API Key' : '显示 API Key'} title={showApiKey ? '隐藏 API Key' : '显示 API Key'} on:click={() => (showApiKey = !showApiKey)}>{#if showApiKey}<EyeOff size={16} />{:else}<Eye size={16} />{/if}</button>
          </div>
          <div class="key-storage-note">
            <small>密钥以明文保存在当前 Windows 用户的本地应用数据中，请勿共享该数据目录。</small>
            <button class:armed={clearKeyArmed} type="button" disabled={!aiApiKey || savingAi || testingAi} on:click={clearApiKey}>{clearKeyArmed ? '再次点击确认清除并停用 AI' : '清除密钥'}</button>
          </div>
        </label>
      </div>
      {#if connectionResult?.ok}
        <div class="connection-result success"><CheckCircle2 size={15} /><span>{connectionResult.message} · {connectionResult.model} · {connectionResult.latencyMs}ms</span></div>
      {:else if aiError}
        <div class="connection-result error" role="alert"><Info size={15} /><span>{aiError}</span></div>
      {/if}
      <div class="ai-settings-actions">
        <span>{aiEnabled ? 'AI 入口会显示在项目编辑器中。' : 'AI 当前停用，核心 PDF 功能仍可离线使用。'}</span>
        <button class="button secondary" type="button" disabled={testingAi || savingAi || !aiConfigValid} on:click={testConnection}>
          {#if testingAi}<LoaderCircle class="spin" size={15} /> 测试中…{:else}<TestTube2 size={15} /> 测试连接{/if}
        </button>
        <button class="button primary" type="button" disabled={savingAi || testingAi || !aiDirty} on:click={saveAiSettings}>
          {#if savingAi}<LoaderCircle class="spin" size={15} /> 保存中…{:else}保存 AI 设置{/if}
        </button>
      </div>
    </div>
  </section>

  <section class="settings-section">
    <div class="settings-heading"><FolderOpen size={19} /><div><h2>默认导出位置</h2><p>生成 PDF 时优先打开此文件夹。</p></div></div>
    <div class="settings-value-row">
      <code>{snapshot.settings.defaultExportDirectory || '每次询问保存位置'}</code>
      <button class="button secondary" type="button" on:click={onChooseExportDirectory}>选择文件夹</button>
      {#if snapshot.settings.defaultExportDirectory}
        <button class="icon-button" type="button" title="恢复为每次询问" aria-label="恢复为每次询问" on:click={onResetExportDirectory}><RotateCcw size={17} /></button>
      {/if}
    </div>
  </section>

  <section class="settings-section">
    <div class="settings-heading"><Database size={19} /><div><h2>本地数据</h2><p>材料副本、项目和模板保存在当前 Windows 用户目录。</p></div></div>
    <div class="settings-value-row">
      <code>{snapshot.dataDirectory}</code>
      <button class="button secondary" type="button" on:click={onOpenDataDirectory}>打开目录 <ExternalLink size={15} /></button>
    </div>
  </section>

  <section class="settings-section">
    <div class="settings-heading"><FileCog size={19} /><div><h2>PDF 引擎</h2><p>负责检查页数和合并输出。</p></div></div>
    <div class="about-grid"><span>引擎状态</span><strong>{snapshot.qpdfVersion || '尚未检测'}</strong></div>
  </section>

  <section class="settings-section about-section">
    <div class="settings-heading"><Info size={19} /><div><h2>关于</h2><p>保研材料助手</p></div></div>
    <div class="about-grid">
      <span>应用版本</span><strong>{snapshot.appVersion}</strong>
      <span>工作方式</span><strong>Windows 本地桌面应用</strong>
      <span>网络依赖</span><strong>{snapshot.settings.aiEnabled ? '仅 AI 整理需要联网' : '核心功能无需联网'}</strong>
    </div>
  </section>
</main>

<style>
  .ai-settings-heading { position: relative; padding-right: 54px; }
  .ai-enable-switch { position: absolute; top: 0; right: 0; }
  .ai-settings-body { display: flex; flex-direction: column; gap: 14px; margin-top: 16px; padding-left: 30px; }
  .ai-privacy-copy { display: flex; align-items: flex-start; gap: 8px; padding: 10px 12px; border: 1px solid #cee3d8; border-radius: 8px; color: #237451; background: #f1faf5; }
  .ai-privacy-copy > span { display: flex; flex-direction: column; gap: 2px; }
  .ai-privacy-copy strong { font-size: 12px; }
  .ai-privacy-copy small { color: #5c796b; font-size: 11px; line-height: 1.45; }
  .provider-picker { display: grid; gap: 6px; }
  .provider-picker > span { color: #536175; font-size: 11px; font-weight: 600; }
  .provider-picker > div { display: grid; grid-template-columns: 1fr 1fr; gap: 7px; }
  .provider-picker button { display: flex; min-width: 0; align-items: flex-start; flex-direction: column; gap: 2px; padding: 8px 10px; border: 1px solid var(--border); border-radius: 7px; color: var(--text); text-align: left; background: var(--surface); }
  .provider-picker button:hover:not(:disabled) { border-color: #aebdce; background: #fbfcfe; }
  .provider-picker button.active { border-color: #8cb3df; color: #235b94; background: #f3f8fe; box-shadow: 0 0 0 2px rgba(43, 108, 176, 0.07); }
  .provider-picker button strong { font-size: 11px; }
  .provider-picker button small,
  .provider-picker > small { color: var(--text-soft); font-size: 9px; line-height: 1.4; }
  .provider-picker > small { color: #8b5a14; }
  .ai-settings-grid { display: grid; grid-template-columns: minmax(0, 1.7fr) minmax(150px, 0.8fr); gap: 12px; }
  .ai-api-key { grid-column: 1 / -1; }
  .field small { color: var(--text-soft); font-size: 10px; }
  .secret-input { display: flex; }
  .secret-input input { padding-right: 42px; }
  .secret-input button { width: 38px; margin-left: -38px; border: 0; color: var(--text-soft); background: transparent; }
  .key-storage-note { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
  .key-storage-note small { min-width: 0; }
  .key-storage-note button { flex: 0 0 auto; padding: 2px 5px; border: 0; border-radius: 4px; color: var(--danger); background: transparent; font-size: 9px; }
  .key-storage-note button:hover:not(:disabled),
  .key-storage-note button.armed { background: #fff0ee; }
  .connection-result { display: flex; align-items: flex-start; gap: 7px; padding: 8px 10px; border-radius: 7px; font-size: 11px; }
  .connection-result.success { color: #237451; background: #edf9f3; }
  .connection-result.error { color: var(--danger); background: #fff1f1; }
  .ai-settings-actions { display: flex; align-items: center; justify-content: flex-end; gap: 8px; }
  .ai-settings-actions > span { min-width: 0; flex: 1; color: var(--text-soft); font-size: 10px; }

  @media (max-width: 760px) {
    .ai-settings-body { padding-left: 0; }
    .ai-settings-grid { grid-template-columns: 1fr; }
    .provider-picker > div { grid-template-columns: 1fr; }
    .ai-api-key { grid-column: auto; }
    .ai-settings-actions { align-items: stretch; flex-direction: column; }
  }
</style>
