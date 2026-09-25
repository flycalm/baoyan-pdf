import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Material } from '../lib/types';
import MaterialPickerModal from './MaterialPickerModal.svelte';

afterEach(cleanup);

function material(id: string, name: string, category: string): Material {
  return {
    id,
    name,
    category,
    originalName: `${name}.pdf`,
    sizeBytes: 1024,
    pageCount: 1,
    sha256: `hash-${id}`,
    scope: 'library',
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
    usedByCount: 0,
    status: 'ready',
  };
}

describe('MaterialPickerModal', () => {
  it('excludes materials already in the project and selects all visible results at once', async () => {
    const onAdd = vi.fn();
    render(MaterialPickerModal, {
      props: {
        open: true,
        materials: [
          material('resume', '个人简历', '个人简历'),
          material('transcript', '本科成绩单', '本科成绩单'),
          material('certificate', '获奖证书', '获奖证书'),
        ],
        addedMaterialIds: ['resume'],
        onAdd,
      },
    });

    expect(screen.queryByText('个人简历', { selector: '.picker-copy strong' })).toBeNull();
    await fireEvent.click(screen.getByRole('button', { name: '全选当前 2 份' }));
    expect(screen.getByText('已选择 2 份')).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: '添加 2 份到项目' }));
    expect(onAdd).toHaveBeenCalledTimes(1);
    expect(new Set(onAdd.mock.calls[0][0])).toEqual(new Set(['transcript', 'certificate']));
  });

  it('selects only the current search results and keeps earlier selections', async () => {
    render(MaterialPickerModal, {
      props: {
        open: true,
        materials: [
          material('resume', '中文简历', '个人简历'),
          material('transcript', '本科成绩单', '本科成绩单'),
          material('ranking', '成绩排名证明', '成绩排名证明'),
        ],
      },
    });

    await fireEvent.input(screen.getByPlaceholderText('搜索名称或分类'), {
      target: { value: '成绩' },
    });
    await fireEvent.click(screen.getByRole('button', { name: '全选当前 2 份' }));
    await fireEvent.input(screen.getByPlaceholderText('搜索名称或分类'), {
      target: { value: '简历' },
    });
    await fireEvent.click(screen.getByRole('button', { name: '全选当前 1 份' }));

    expect(screen.getByText('已选择 3 份')).toBeTruthy();
  });
});
