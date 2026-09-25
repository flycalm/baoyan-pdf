import { MATERIAL_CATEGORIES, type Material } from './types';

export const CUSTOM_MATERIAL_NAME_MAX_LENGTH = 40;

export function normalizeCustomMaterialName(value: string): string {
  return value.trim().replace(/\s+/g, ' ');
}

export function isValidCustomMaterialName(value: string): boolean {
  const normalized = normalizeCustomMaterialName(value);
  return (
    normalized.length > 0 &&
    Array.from(normalized).length <= CUSTOM_MATERIAL_NAME_MAX_LENGTH &&
    !/[\u0000-\u001f\u007f]/.test(value)
  );
}

export function availableLibraryCategories(materials: Material[]): string[] {
  const usedCategories = new Set(
    materials
      .filter((material) => material.scope === 'library')
      .map((material) => material.category.trim())
      .filter(Boolean),
  );
  const presetCategories = MATERIAL_CATEGORIES.filter((category) => usedCategories.has(category));
  const presetSet = new Set<string>(MATERIAL_CATEGORIES);
  const customCategories = [...usedCategories]
    .filter((category) => !presetSet.has(category))
    .sort((left, right) => left.localeCompare(right, 'zh-CN'));

  return [...presetCategories, ...customCategories];
}
