import { describe, expect, it } from 'vitest';
import { formatBytes, moveItem, safeOutputStem } from './utils';
import type { ApplicationProject } from './types';

describe('formatBytes', () => {
  it('formats common sizes', () => {
    expect(formatBytes(0)).toBe('0 KB');
    expect(formatBytes(1024)).toBe('1.00 KB');
    expect(formatBytes(10 * 1024 * 1024)).toBe('10.0 MB');
  });
});

describe('moveItem', () => {
  it('moves without mutating the source array', () => {
    const source = ['a', 'b', 'c'];
    expect(moveItem(source, 0, 2)).toEqual(['b', 'c', 'a']);
    expect(source).toEqual(['a', 'b', 'c']);
  });
});

describe('safeOutputStem', () => {
  it('removes Windows-invalid characters', () => {
    const project = {
      school: 'A:B?',
      department: '计算机/学院',
      program: '',
    } as ApplicationProject;
    expect(safeOutputStem(project)).toBe('A-B-计算机-学院-申请材料');
  });
});

