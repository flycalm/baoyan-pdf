import { describe, expect, it } from 'vitest';
import {
  beginAiUndoTransaction,
  isAiUndoValid,
  persistAiUndoStateToStorage,
  removeAiUndoRecord,
  removeAiUndoRecordIfSame,
  rollbackAiUndoTransaction,
  restoreAiUndoState,
  serializeAiUndoState,
  shouldInvalidateAiUndoAfterSave,
  upsertAiUndoRecord,
  type AiUndoState,
} from './aiUndo';
import type { ApplicationProject } from './types';

function project(id: string, notes = ''): ApplicationProject {
  return {
    id,
    school: `学校-${id}`,
    department: '',
    program: '',
    noticeUrl: '',
    notes,
    modules: [],
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
  };
}

describe('per-project AI undo state', () => {
  it('keeps independent records when AI is applied in different projects', () => {
    const firstBefore = project('first');
    const firstApplied = project('first', 'AI first');
    const secondBefore = project('second');
    const secondApplied = project('second', 'AI second');

    let state: AiUndoState = {};
    state = upsertAiUndoRecord(state, firstBefore, firstApplied);
    state = upsertAiUndoRecord(state, secondBefore, secondApplied);
    state = removeAiUndoRecord(state, 'second');

    expect(state.first).toBeDefined();
    expect(state.second).toBeUndefined();
    expect(isAiUndoValid(state.first, firstApplied)).toBe(true);
  });

  it('invalidates undo after a later manual edit while ignoring timestamp-only changes', () => {
    const before = project('project');
    const applied = project('project', 'AI arranged');
    const state = upsertAiUndoRecord({}, before, applied);
    const savedByBackend = { ...applied, updatedAt: '2026-02-02T00:00:00Z' };
    const manuallyEdited = { ...savedByBackend, notes: 'AI arranged + my edit' };

    expect(isAiUndoValid(state.project, savedByBackend)).toBe(true);
    expect(isAiUndoValid(state.project, manuallyEdited)).toBe(false);
  });

  it('restores only still-valid records after an application restart', () => {
    const firstBefore = project('first');
    const firstApplied = project('first', 'AI first');
    const secondBefore = project('second');
    const secondApplied = project('second', 'AI second');
    let state: AiUndoState = {};
    state = upsertAiUndoRecord(state, firstBefore, firstApplied);
    state = upsertAiUndoRecord(state, secondBefore, secondApplied);

    const restored = restoreAiUndoState(
      serializeAiUndoState(state),
      [firstApplied, { ...secondApplied, notes: 'edited after AI' }],
    );

    expect(Object.keys(restored)).toEqual(['first']);
    expect(restored.first.previousProject).toEqual(firstBefore);
  });

  it('prewrites the new record and restores the exact previous per-project state after save failure', () => {
    const priorFirst = project('first', 'old AI');
    const state = upsertAiUndoRecord({}, project('first'), priorFirst);
    const beforeSecond = project('second');
    const appliedSecond = project('second', 'new AI');

    const transaction = beginAiUndoTransaction(state, beforeSecond, appliedSecond);

    expect(transaction.preparedState.first).toBeDefined();
    expect(transaction.preparedState.second).toBeDefined();
    expect(rollbackAiUndoTransaction(transaction)).toEqual(state);
  });

  it('reports a storage write failure instead of throwing or pretending persistence succeeded', () => {
    const state = upsertAiUndoRecord({}, project('first'), project('first', 'AI'));
    const failingStorage = {
      setItem() { throw new Error('quota denied'); },
      removeItem() { throw new Error('quota denied'); },
    };

    expect(persistAiUndoStateToStorage(state, failingStorage, 'undo-key')).toBe(false);
    expect(persistAiUndoStateToStorage({}, failingStorage, 'undo-key')).toBe(false);
  });

  it('keeps undo on a failed manual save and removes it only after the captured save succeeds', () => {
    const applied = project('first', 'AI result');
    const manuallyEdited = project('first', 'manual edit');
    const state = upsertAiUndoRecord({}, project('first'), applied);
    const captured = state.first;

    expect(shouldInvalidateAiUndoAfterSave(captured, manuallyEdited)).toBe(true);
    // A failed DB save performs no commit, so the state remains available for restart recovery.
    expect(state.first).toBe(captured);

    const afterSuccessfulSave = removeAiUndoRecordIfSame(state, captured);
    expect(afterSuccessfulSave.first).toBeUndefined();
  });

  it('does not let an older save invalidate a newer AI undo record', () => {
    const firstApplied = project('first', 'first AI');
    const firstState = upsertAiUndoRecord({}, project('first'), firstApplied);
    const capturedOldRecord = firstState.first;
    const newerState = upsertAiUndoRecord(firstState, firstApplied, project('first', 'second AI'));

    expect(removeAiUndoRecordIfSame(newerState, capturedOldRecord)).toBe(newerState);
    expect(newerState.first).toBeDefined();
  });
});
