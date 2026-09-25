import { describe, expect, it } from 'vitest';
import {
  buildAiArrangedProject,
  extractExplicitSelectionLimit,
  getAiApplyConfirmationItems,
  matchAiRequirements,
} from './aiMatching';
import type { AiRegistrationRequirement, ApplicationProject, Material } from './types';

function material(
  id: string,
  name: string,
  category: string,
  status: Material['status'] = 'ready',
  originalName = `${name}.pdf`,
): Material {
  return {
    id,
    name,
    category,
    originalName,
    sizeBytes: 100,
    pageCount: 1,
    sha256: `hash-${id}`,
    scope: 'library',
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
    usedByCount: 0,
    status,
  };
}

function requirement(name: string, category: string, required = true): AiRegistrationRequirement {
  return { name, category, required, details: `${name}要求`, evidenceIds: [] };
}

function project(): ApplicationProject {
  return {
    id: 'project-1',
    school: '测试大学',
    department: '',
    program: '',
    noticeUrl: '',
    notes: '',
    modules: [],
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
  };
}

describe('AI material matching', () => {
  it('uses category aliases and excludes unavailable materials', () => {
    const matches = matchAiRequirements(
      [requirement('外语水平证明', '英语成绩证明')],
      [
        material('cet6', '六级成绩单', '英语成绩证明'),
        material('broken', '英语四级成绩', '英语成绩证明', 'invalid'),
      ],
    );

    expect(matches[0].status).toBe('matched');
    expect(matches[0].candidates.map((entry) => entry.id)).toEqual(['cet6']);
    expect(matches[0].selectedMaterialIds).toEqual(['cet6']);
  });

  it('never matches an English-score material to a transcript requirement through the generic word 成绩单', () => {
    const matches = matchAiRequirements(
      [requirement('本科成绩单', '本科成绩单')],
      [
        material('transcript', '本科成绩单（盖章）', '本科成绩单'),
        material('cet6', '大学英语六级成绩单', '英语成绩证明'),
      ],
    );

    expect(matches[0].candidates.map((entry) => entry.id)).toEqual(['transcript']);
    expect(matches[0].selectedMaterialIds).toEqual(['transcript']);
  });

  it('still allows filename fallback when the material uses a custom category', () => {
    const matches = matchAiRequirements(
      [requirement('本科成绩单', '本科成绩单')],
      [material('custom-transcript', '附件一', '其他材料', 'ready', '本科成绩单.pdf')],
    );

    expect(matches[0].candidates.map((entry) => entry.id)).toEqual(['custom-transcript']);
  });

  it('finds a material by original file name when its category is custom', () => {
    const matches = matchAiRequirements(
      [requirement('研究计划', '研究计划')],
      [material('plan', '附件三', '其他材料', 'ready', '附件3-研究计划.pdf')],
    );

    expect(matches[0].status).toBe('matched');
    expect(matches[0].selectedMaterialIds).toEqual(['plan']);
  });

  it('defaults collection categories to all candidates but single-document categories to the first', () => {
    const materials = [
      material('award-a', '国家奖学金', '获奖证书'),
      material('award-b', '竞赛证书', '获奖证书'),
      material('resume-a', '中文简历', '个人简历'),
      material('resume-b', '英文简历', '个人简历'),
    ];
    const matches = matchAiRequirements(
      [requirement('获奖证书', '获奖证书', false), requirement('个人简历', '个人简历')],
      materials,
    );

    expect(matches[0].status).toBe('multiple');
    expect(matches[0].selectedMaterialIds).toEqual(['award-a', 'award-b']);
    expect(matches[1].status).toBe('multiple');
    expect(matches[1].selectedMaterialIds).toHaveLength(1);
  });

  it.each([
    ['代表性论文限1篇', 1],
    ['获奖材料最多3项', 3],
    ['证明材料不超过2份', 2],
    ['请选择一项代表性成果', 1],
    ['任选一篇论文', 1],
    ['论文最多十二篇', 12],
  ])('conservatively extracts an explicit selection limit from “%s”', (details, expected) => {
    const target = {
      ...requirement('代表性材料', '论文与科研成果', false),
      details,
      evidenceIds: ['limit-evidence'],
    };
    expect(extractExplicitSelectionLimit(target, [
      { id: 'limit-evidence', sourceUrl: 'https://example.edu', quote: details },
    ])).toBe(expected);
  });

  it.each([
    ['最多提交3篇', 3],
    ['限上传1项', 1],
    ['不超过提供2个', 2],
  ])('accepts an upload verb when it follows explicit upper-limit wording: “%s”', (quote, expected) => {
    const target = {
      ...requirement('代表性材料', '论文与科研成果', false),
      evidenceIds: ['limit-evidence'],
    };
    expect(extractExplicitSelectionLimit(target, [
      { id: 'limit-evidence', sourceUrl: '', quote },
    ])).toBe(expected);
  });

  it('does not infer a limit from counts without upper-limit wording', () => {
    const target = {
      ...requirement('论文与科研成果', '论文与科研成果', false),
      details: '已发表论文共3篇，请上传证明材料',
      evidenceIds: ['count-evidence'],
    };
    expect(extractExplicitSelectionLimit(target, [
      { id: 'count-evidence', sourceUrl: '', quote: target.details },
    ])).toBeNull();
  });

  it.each([
    ['个人简历', '个人简历', '提交2份'],
    ['本科成绩单', '本科成绩单', '一式两份'],
    ['申请表', '申请表', '申请表1份及证书若干'],
    ['推荐信', '推荐信', '请上传2份专家推荐信'],
    ['本科成绩单', '本科成绩单', '成绩单限2份'],
  ])('does not mistake copy counts for selection limits: %s / %s', (name, category, details) => {
    const target = {
      ...requirement(name, category),
      details,
      evidenceIds: ['copies-evidence'],
    };
    expect(extractExplicitSelectionLimit(target, [
      { id: 'copies-evidence', sourceUrl: '', quote: details },
    ])).toBeNull();
  });

  it('accepts “份” only with explicit choice wording for a non-collection document', () => {
    const target = {
      ...requirement('其他证明', '其他材料', false),
      details: '最多选择2份材料',
      evidenceIds: ['choice-evidence'],
    };
    expect(extractExplicitSelectionLimit(target, [
      { id: 'choice-evidence', sourceUrl: '', quote: target.details },
    ])).toBe(2);
  });

  it('truncates collection defaults at an explicit limit', () => {
    const matches = matchAiRequirements(
      [{ ...requirement('代表性获奖', '获奖证书', false), details: '任选两项代表性获奖', evidenceIds: ['award-limit'] }],
      [
        material('award-a', '奖项 A', '获奖证书'),
        material('award-b', '奖项 B', '获奖证书'),
        material('award-c', '奖项 C', '获奖证书'),
      ],
      [{ id: 'award-limit', sourceUrl: '', quote: '任选两项代表性获奖' }],
    );

    expect(matches[0].selectionLimit).toBe(2);
    expect(matches[0].selectedMaterialIds).toHaveLength(2);
  });

  it('requires confirmation for multiple candidates, missing required files, and reached limits', () => {
    const multiple = matchAiRequirements(
      [{ ...requirement('代表性获奖', '获奖证书', false), details: '限1项', evidenceIds: ['award-limit'] }],
      [material('award-a', '奖项 A', '获奖证书'), material('award-b', '奖项 B', '获奖证书')],
      [{ id: 'award-limit', sourceUrl: '', quote: '代表性获奖限1项' }],
    )[0];
    const missing = matchAiRequirements([requirement('推荐信', '推荐信')], [])[0];
    const items = getAiApplyConfirmationItems([multiple, missing]);

    expect(items.map((item) => item.kind)).toEqual(['multiple', 'selection-limit', 'missing-required']);
  });

  it('does not truncate defaults from a model-only limit claim without supporting evidence', () => {
    const target = {
      ...requirement('论文与科研成果', '论文与科研成果', false),
      details: '最多选择1篇代表性论文',
      evidenceIds: ['paper-evidence'],
    };
    const matches = matchAiRequirements(
      [target],
      [
        material('paper-a', '论文 A', '论文与科研成果'),
        material('paper-b', '论文 B', '论文与科研成果'),
        material('paper-c', '论文 C', '论文与科研成果'),
      ],
      [{ id: 'paper-evidence', sourceUrl: '', quote: '可提交论文材料。' }],
    );

    expect(matches[0].selectionLimit).toBeNull();
    expect(matches[0].selectedMaterialIds).toHaveLength(3);
    expect(matches[0].unverifiedLimitClaim).toBe(true);
  });

  it('keeps every selected candidate in candidate order when building project modules', () => {
    const matches = matchAiRequirements(
      [requirement('论文与科研成果', '论文与科研成果', false)],
      [
        material('paper-b', '论文 B', '论文与科研成果'),
        material('paper-a', '论文 A', '论文与科研成果'),
      ],
    );
    const arranged = buildAiArrangedProject(project(), matches);

    expect(arranged.modules).toHaveLength(1);
    expect(arranged.modules[0].files.map((file) => file.materialId)).toEqual(
      matches[0].candidates.map((candidate) => candidate.id),
    );
    expect(arranged.modules[0].enabled).toBe(true);
  });

  it('keeps unselected old files and empty modules disabled at the end', () => {
    const resume = material('resume', '个人简历', '个人简历');
    const oldAward = material('old-award', '旧获奖材料', '获奖证书');
    const source = project();
    source.modules = [
      {
        id: 'old-mixed',
        title: '旧材料组合',
        category: '其他材料',
        position: 0,
        required: true,
        enabled: true,
        files: [resume, oldAward].map((entry, position) => ({
          id: `ref-${entry.id}`,
          materialId: entry.id,
          position,
          material: entry,
        })),
      },
      {
        id: 'old-empty',
        title: '待补推荐信',
        category: '推荐信',
        position: 1,
        required: true,
        enabled: true,
        files: [],
      },
    ];
    const matches = matchAiRequirements([requirement('个人简历', '个人简历')], [resume]);
    const arranged = buildAiArrangedProject(source, matches);

    expect(arranged.modules.map((module) => module.title)).toEqual([
      '个人简历',
      '旧材料组合（未纳入 AI 编排）',
      '待补推荐信',
    ]);
    expect(arranged.modules[1].files.map((file) => file.materialId)).toEqual(['old-award']);
    expect(arranged.modules.slice(1).every((module) => !module.enabled && !module.required)).toBe(true);
  });

  it('does not create a required empty module when the user explicitly ignores that requirement', () => {
    const matches = matchAiRequirements([requirement('推荐信', '推荐信')], []);
    matches[0] = { ...matches[0], included: false };

    const arranged = buildAiArrangedProject(project(), matches);
    const confirmation = getAiApplyConfirmationItems(matches);

    expect(arranged.modules).toEqual([]);
    expect(confirmation.map((item) => item.kind)).toEqual(['ignored']);
    expect(confirmation[0].message).toContain('通知标为必需');
  });
});
