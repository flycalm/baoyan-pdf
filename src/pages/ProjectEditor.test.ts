import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ApplicationProject, Material, ModuleFile } from '../lib/types';
import ProjectEditor from './ProjectEditor.svelte';

afterEach(cleanup);

function pointerEvent(type: string, clientY: number): Event {
  const event = new MouseEvent(type, { bubbles: true, button: 0, clientY });
  Object.defineProperty(event, 'pointerId', { value: 1 });
  return event;
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
    usedByCount: 1,
    status: 'ready',
  };
}

function moduleFile(id: string, name: string, position: number): ModuleFile {
  const fileMaterial = material(id, name);
  return { id: `file-${id}`, materialId: id, position, material: fileMaterial };
}

function project(): ApplicationProject {
  return {
    id: 'project-1',
    school: '回归测试大学',
    department: '',
    program: '',
    noticeUrl: '',
    notes: '',
    modules: [
      {
        id: 'module-1',
        title: '获奖证书',
        category: '获奖证书',
        position: 0,
        required: false,
        enabled: true,
        files: [moduleFile('first', '第一份材料', 0), moduleFile('second', '第二份材料', 1)],
      },
    ],
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
  };
}

describe('ProjectEditor material drag ordering', () => {
  it('tracks a pointer drag from the module handle and persists the dropped order', async () => {
    const onSave = vi.fn(async (_project: ApplicationProject) => undefined);
    const targetProject = project();
    targetProject.modules = [
      { ...targetProject.modules[0], id: 'module-first', title: '第一模块' },
      { ...targetProject.modules[0], id: 'module-second', title: '第二模块' },
    ];
    render(ProjectEditor, { props: { project: targetProject, onSave } });
    const secondHandle = screen.getByRole('button', { name: '拖动第二模块' });

    await fireEvent(secondHandle, pointerEvent('pointerdown', 10));
    await waitFor(() =>
      expect(secondHandle.closest('.module-card')?.classList.contains('dragging')).toBe(true),
    );
    await fireEvent(secondHandle, pointerEvent('pointermove', -1));
    await fireEvent(secondHandle, pointerEvent('pointerup', -1));

    await waitFor(() => expect(onSave).toHaveBeenCalled());
    const savedProject = onSave.mock.calls.at(-1)![0];
    expect(savedProject.modules.map((module) => module.title)).toEqual(['第二模块', '第一模块']);
    expect(savedProject.modules.map((module) => module.position)).toEqual([0, 1]);
  });

  it('tracks a pointer drag from the material handle and persists the dropped order', async () => {
    const onSave = vi.fn(async (_project: ApplicationProject) => undefined);
    render(ProjectEditor, { props: { project: project(), onSave } });
    const secondHandle = screen.getByRole('button', { name: '拖动材料：第二份材料' });
    await fireEvent(secondHandle, pointerEvent('pointerdown', 10));
    await waitFor(() =>
      expect(secondHandle.closest('.module-file-row')?.classList.contains('dragging')).toBe(true),
    );
    await fireEvent(secondHandle, pointerEvent('pointermove', -1));
    await fireEvent(secondHandle, pointerEvent('pointerup', -1));

    await waitFor(() => expect(onSave).toHaveBeenCalled());
    const savedProject = onSave.mock.calls.at(-1)![0];
    expect(savedProject.modules[0].files.map((file) => file.material.name)).toEqual([
      '第二份材料',
      '第一份材料',
    ]);
    expect(savedProject.modules[0].files.map((file) => file.position)).toEqual([0, 1]);
  });

  it('immediately applies a same-project material refresh from the library', async () => {
    const onSave = vi.fn(async (_project: ApplicationProject) => undefined);
    const emptyProject = { ...project(), modules: [] };
    const updatedProject = project();
    const { rerender } = render(ProjectEditor, {
      props: { project: emptyProject, refreshRevision: 0, onSave },
    });

    expect(screen.getByText('把本次申请需要的材料放进来')).toBeTruthy();
    await rerender({ project: updatedProject, refreshRevision: 1, onSave });

    expect(screen.queryByText('把本次申请需要的材料放进来')).toBeNull();
    expect(screen.getByText('第一份材料')).toBeTruthy();
    expect(screen.getByText('第二份材料')).toBeTruthy();
  });
});
