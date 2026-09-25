import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import { MATERIAL_CATEGORIES } from '../lib/types';
import ImportMaterialsModal from './ImportMaterialsModal.svelte';

describe('ImportMaterialsModal', () => {
  it('keeps every preset and imports with a custom material name', async () => {
    const onImport = vi.fn();
    render(ImportMaterialsModal, {
      props: {
        open: true,
        onChooseFiles: async () => ['C:\\材料\\申请表.pdf'],
        onImport,
      },
    });

    const select = screen.getByRole('combobox', { name: '材料分类' }) as HTMLSelectElement;
    expect([...select.options].map((option) => option.text)).toEqual([...MATERIAL_CATEGORIES]);

    await fireEvent.click(screen.getByRole('button', { name: /自定义新的材料名称/ }));
    await fireEvent.input(screen.getByRole('textbox', { name: '自定义材料名称' }), {
      target: { value: '  夏令营   申请表  ' },
    });
    await fireEvent.click(screen.getByRole('button', { name: /选择或拖入一个或多个 PDF/ }));
    const importButton = await waitFor(() =>
      screen.getByRole('button', { name: '导入 1 份材料' }),
    );
    expect((importButton as HTMLButtonElement).disabled).toBe(false);
    await fireEvent.click(importButton);

    expect(onImport).toHaveBeenCalledWith(['C:\\材料\\申请表.pdf'], '夏令营 申请表');
  });
});
