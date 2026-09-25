import { describe, expect, it } from 'vitest';
import {
  availableLibraryCategories,
  isValidCustomMaterialName,
  normalizeCustomMaterialName,
} from './materialCategories';
import type { Material } from './types';

function material(category: string, scope: Material['scope'] = 'library'): Material {
  return {
    id: crypto.randomUUID(),
    name: category,
    category,
    originalName: `${category}.pdf`,
    sizeBytes: 1,
    pageCount: 1,
    sha256: 'hash',
    scope,
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
    usedByCount: 0,
    status: 'ready',
  };
}

describe('custom material categories', () => {
  it('keeps presets in their original order and appends custom names', () => {
    const result = availableLibraryCategories([
      material('夏令营申请表'),
      material('获奖证书'),
      material('个人简历'),
      material('专项证明'),
    ]);

    expect(result).toEqual(['个人简历', '获奖证书', '夏令营申请表', '专项证明']);
  });

  it('does not expose project-only custom names in the library filter', () => {
    expect(availableLibraryCategories([material('项目临时材料', 'project')])).toEqual([]);
  });

  it('normalizes and validates a custom name', () => {
    expect(normalizeCustomMaterialName('  夏令营   申请表  ')).toBe('夏令营 申请表');
    expect(isValidCustomMaterialName('  夏令营申请表  ')).toBe(true);
    expect(isValidCustomMaterialName('   ')).toBe(false);
    expect(isValidCustomMaterialName('a'.repeat(41))).toBe(false);
    expect(isValidCustomMaterialName('申请表\n副本')).toBe(false);
  });
});
