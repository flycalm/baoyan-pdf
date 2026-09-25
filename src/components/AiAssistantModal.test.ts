import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { AiRegistrationAnalysis, Material } from '../lib/types';
import AiAssistantModal from './AiAssistantModal.svelte';

afterEach(cleanup);

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

function analysis(): AiRegistrationAnalysis {
  return {
    summary: '不应显示的晚到结果',
    requirements: [],
    evidence: [],
    source: [],
    confidence: 0.9,
    warnings: [],
    toolLog: [],
  };
}

function material(id: string, name: string): Material {
  return {
    id,
    name,
    category: '获奖证书',
    originalName: `${name}.pdf`,
    sizeBytes: 100,
    pageCount: 1,
    sha256: `hash-${id}`,
    scope: 'library',
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
    usedByCount: 0,
    status: 'ready',
  };
}

describe('AiAssistantModal cancellation', () => {
  it('cancels by requestId and ignores a result that arrives after cancellation', async () => {
    const pending = deferred<AiRegistrationAnalysis>();
    const onAnalyze = vi.fn((_input: { noticeUrl: string; noticeText: string; requestId: string }) => pending.promise);
    const onCancelAnalysis = vi.fn(async () => true);
    render(AiAssistantModal, {
      props: {
        open: true,
        projectNoticeUrl: 'https://example.edu/notice',
        materials: [],
        onAnalyze,
        onCancelAnalysis,
      },
    });

    await fireEvent.click(screen.getByRole('button', { name: '开始整理' }));
    await waitFor(() => expect(onAnalyze).toHaveBeenCalledOnce());
    const requestId = onAnalyze.mock.calls[0]![0].requestId;
    expect(requestId).toEqual(expect.any(String));

    await fireEvent.click(screen.getByRole('button', { name: '取消分析' }));
    await waitFor(() => expect(onCancelAnalysis).toHaveBeenCalledWith(requestId));
    await waitFor(() => expect(screen.getByRole('button', { name: '开始整理' })).toBeTruthy());

    pending.resolve(analysis());
    await Promise.resolve();
    expect(screen.queryByText('不应显示的晚到结果')).toBeNull();
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('cancels before closing an analyzing modal', async () => {
    const pending = deferred<AiRegistrationAnalysis>();
    const onAnalyze = vi.fn((_input: { noticeUrl: string; noticeText: string; requestId: string }) => pending.promise);
    const onCancelAnalysis = vi.fn(async () => true);
    const onClose = vi.fn();
    render(AiAssistantModal, {
      props: {
        open: true,
        projectNoticeUrl: 'https://example.edu/notice',
        materials: [],
        onAnalyze,
        onCancelAnalysis,
        onClose,
      },
    });

    await fireEvent.click(screen.getByRole('button', { name: '开始整理' }));
    await fireEvent.click(screen.getByRole('button', { name: '关闭' }));

    await waitFor(() => expect(onCancelAnalysis).toHaveBeenCalledOnce());
    expect(onClose).toHaveBeenCalledOnce();
  });

  it('requires a second explicit click before applying a multiple-candidate result', async () => {
    const result: AiRegistrationAnalysis = {
      ...analysis(),
      summary: '发现多个获奖候选',
      requirements: [{
        name: '获奖证书',
        category: '获奖证书',
        required: false,
        details: '代表性获奖证明',
        evidenceIds: ['award-evidence'],
      }],
      evidence: [{ id: 'award-evidence', sourceUrl: '', quote: '可提交获奖证明。' }],
    };
    const onApply = vi.fn(async () => undefined);
    render(AiAssistantModal, {
      props: {
        open: true,
        projectNoticeUrl: 'https://example.edu/notice',
        materials: [material('award-a', '奖项 A'), material('award-b', '奖项 B')],
        onAnalyze: vi.fn(async () => result),
        onApply,
      },
    });

    await fireEvent.click(screen.getByRole('button', { name: '开始整理' }));
    await screen.findByText('发现多个获奖候选');
    await fireEvent.click(screen.getByRole('button', { name: '应用到当前项目' }));

    expect(onApply).not.toHaveBeenCalled();
    expect(screen.getByText('应用前请再次确认')).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: '确认并应用' }));
    await waitFor(() => expect(onApply).toHaveBeenCalledOnce());
  });
});
