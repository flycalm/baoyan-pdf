import { describe, expect, it } from 'vitest';
import { appendDroppedPdfPaths, isPhysicalPositionInsideElement } from './fileDrop';

describe('appendDroppedPdfPaths', () => {
  it('accepts PDF paths case-insensitively and rejects other files', () => {
    const result = appendDroppedPdfPaths(
      ['C:\\材料\\简历.pdf'],
      ['C:\\材料\\成绩单.PDF', 'C:\\材料\\照片.png', 'C:\\材料\\简历.PDF'],
    );

    expect(result.paths).toEqual(['C:\\材料\\简历.pdf', 'C:\\材料\\成绩单.PDF']);
    expect(result.addedCount).toBe(1);
    expect(result.duplicateCount).toBe(1);
    expect(result.rejectedCount).toBe(1);
  });
});

describe('isPhysicalPositionInsideElement', () => {
  const rect = { left: 100, right: 500, top: 200, bottom: 400 };

  it('maps Tauri physical coordinates to the CSS pixel drop zone', () => {
    expect(isPhysicalPositionInsideElement({ x: 300, y: 500 }, 2, rect)).toBe(true);
    expect(isPhysicalPositionInsideElement({ x: 1200, y: 500 }, 2, rect)).toBe(false);
  });
});
