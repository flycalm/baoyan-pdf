export interface DroppedPdfSelection {
  paths: string[];
  addedCount: number;
  duplicateCount: number;
  rejectedCount: number;
}

function pathKey(path: string) {
  return path.trim().replaceAll('/', '\\').toLocaleLowerCase();
}

export function isPdfPath(path: string) {
  return path.trim().toLocaleLowerCase().endsWith('.pdf');
}

export function appendDroppedPdfPaths(
  currentPaths: readonly string[],
  droppedPaths: readonly string[],
): DroppedPdfSelection {
  const paths = [...currentPaths];
  const knownPaths = new Set(paths.map(pathKey));
  let addedCount = 0;
  let duplicateCount = 0;
  let rejectedCount = 0;

  for (const rawPath of droppedPaths) {
    const path = rawPath.trim();
    if (!isPdfPath(path)) {
      rejectedCount += 1;
      continue;
    }

    const key = pathKey(path);
    if (knownPaths.has(key)) {
      duplicateCount += 1;
      continue;
    }

    knownPaths.add(key);
    paths.push(path);
    addedCount += 1;
  }

  return { paths, addedCount, duplicateCount, rejectedCount };
}

export function isPhysicalPositionInsideElement(
  position: { x: number; y: number },
  scaleFactor: number,
  rect: Pick<DOMRect, 'left' | 'right' | 'top' | 'bottom'>,
) {
  const safeScaleFactor = Number.isFinite(scaleFactor) && scaleFactor > 0 ? scaleFactor : 1;
  const x = position.x / safeScaleFactor;
  const y = position.y / safeScaleFactor;
  return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
}
