import type {
  AiRegistrationEvidence,
  AiRegistrationRequirement,
  ApplicationProject,
  Material,
  ProjectModule,
} from './types';

export type AiMaterialMatchStatus = 'matched' | 'multiple' | 'missing';

export interface AiRequirementMatch {
  id: string;
  requirement: AiRegistrationRequirement;
  included: boolean;
  candidates: Material[];
  selectedMaterialIds: string[];
  selectionLimit: number | null;
  unverifiedLimitClaim: boolean;
  status: AiMaterialMatchStatus;
}

export interface AiApplyConfirmationItem {
  kind: 'multiple' | 'missing-required' | 'selection-limit' | 'ignored';
  requirementId: string;
  message: string;
}

const CATEGORY_ALIASES: string[][] = [
  ['个人简历', '简历', 'cv'],
  ['本科成绩单', '成绩单', '本科成绩'],
  ['成绩排名证明', '排名证明', '专业排名', '成绩排名'],
  ['英语成绩证明', '外语成绩', '外语水平', '英语成绩', '四六级', 'cet4', 'cet6'],
  ['论文与科研成果', '论文', '科研成果', '科研证明', '专利'],
  ['获奖证书', '获奖证明', '荣誉证书', '奖项'],
  ['身份证明', '身份证'],
  ['学籍证明', '学籍', '在读证明'],
  ['推荐信', '专家推荐'],
  ['申请表', '报名表'],
  ['诚信承诺书', '承诺书'],
  ['政审材料', '政审表', '思想政治'],
  ['研究计划', '科研计划', '研究设想'],
];

function normalize(value: string): string {
  return value.toLocaleLowerCase().replace(/\.pdf$/i, '').replace(/[\s\p{P}\p{S}]+/gu, '');
}

function aliasesFor(requirement: AiRegistrationRequirement): string[] {
  const seed = [requirement.category, requirement.name].map(normalize).filter(Boolean);
  const group = CATEGORY_ALIASES.find((aliases) => {
    const normalized = aliases.map(normalize);
    return seed.some((item) => normalized.some((alias) => item.includes(alias) || alias.includes(item)));
  });
  return [...new Set([...seed, ...(group || []).map(normalize)])].filter((item) => item.length >= 2);
}

function categoryAliasGroup(value: string): number | null {
  const normalizedValue = normalize(value);
  if (!normalizedValue) return null;

  const exactGroup = CATEGORY_ALIASES.findIndex((aliases) =>
    aliases.some((alias) => normalize(alias) === normalizedValue),
  );
  if (exactGroup >= 0) return exactGroup;

  let bestMatch: { group: number; aliasLength: number } | null = null;
  for (let group = 0; group < CATEGORY_ALIASES.length; group += 1) {
    const aliases = CATEGORY_ALIASES[group];
    for (const alias of aliases) {
      const normalizedAlias = normalize(alias);
      if (normalizedAlias.length < 2 || !normalizedValue.includes(normalizedAlias)) continue;
      if (!bestMatch || normalizedAlias.length > bestMatch.aliasLength) {
        bestMatch = { group, aliasLength: normalizedAlias.length };
      }
    }
  }
  return bestMatch ? bestMatch.group : null;
}

function materialScore(material: Material, requirement: AiRegistrationRequirement): number {
  if (material.status !== 'ready') return 0;
  const requirementCategoryGroup = categoryAliasGroup(requirement.category);
  const materialCategoryGroup = categoryAliasGroup(material.category);
  if (
    requirementCategoryGroup !== null
    && materialCategoryGroup !== null
    && requirementCategoryGroup !== materialCategoryGroup
  ) return 0;
  const category = normalize(material.category);
  const name = normalize(material.name);
  const originalName = normalize(material.originalName);
  const requirementCategory = normalize(requirement.category);
  const requirementName = normalize(requirement.name);
  const aliases = aliasesFor(requirement);
  let score = 0;

  if (category === requirementCategory) score += 120;
  if (name === requirementName || originalName === requirementName) score += 80;
  for (const alias of aliases) {
    if (category === alias) score = Math.max(score, 100);
    if (category.includes(alias) || alias.includes(category)) score = Math.max(score, 70);
    if (name.includes(alias) || originalName.includes(alias)) score = Math.max(score, 55);
  }
  return score;
}

function isCollectionRequirement(requirement: AiRegistrationRequirement): boolean {
  const category = normalize(requirement.category);
  const name = normalize(requirement.name);
  return ['论文与科研成果', '科研成果', '论文', '获奖证书', '获奖证明']
    .map(normalize)
    .some((alias) => category.includes(alias) || name.includes(alias));
}

function parseChineseNumber(value: string): number | null {
  if (/^\d{1,2}$/.test(value)) {
    const parsed = Number(value);
    return parsed > 0 ? parsed : null;
  }
  const digits: Record<string, number> = {
    一: 1, 二: 2, 两: 2, 三: 3, 四: 4, 五: 5, 六: 6, 七: 7, 八: 8, 九: 9,
  };
  if (value === '十') return 10;
  if (value.includes('十')) {
    const [tens, ones] = value.split('十');
    const parsed = (tens ? digits[tens] : 1) * 10 + (ones ? digits[ones] : 0);
    return parsed > 0 && parsed <= 99 ? parsed : null;
  }
  return digits[value] || null;
}

function extractSelectionLimitFromText(text: string, collectionRequirement: boolean): number | null {
  const patterns = [
    /(?:最多|至多|上限(?:为)?|不超过|不得超过|限)\s*(?:可)?\s*(?:选择|选取|任选|提交|上传|提供|附)?\s*(\d{1,2}|[一二两三四五六七八九十]{1,3})\s*(?:篇|项|个|件|种)(?!以上|起)/gu,
    /(?:选择|选取|任选|选)\s*(\d{1,2}|[一二两三四五六七八九十]{1,3})\s*(?:篇|项|份|个|件|种)(?!以上|起)/gu,
    /(?:最多|至多|上限(?:为)?|不超过|不得超过|限)\s*(?:可)?\s*(?:选择|选取|任选)\s*(\d{1,2}|[一二两三四五六七八九十]{1,3})\s*份(?:材料|成果|证书|论文)?(?!以上|起)/gu,
  ];
  if (collectionRequirement) {
    patterns.push(/(?:最多|至多|上限(?:为)?|不超过|不得超过|限)\s*(\d{1,2}|[一二两三四五六七八九十]{1,3})\s*份(?!以上|起)/gu);
  }
  const limits = patterns.flatMap((pattern) =>
    [...text.matchAll(pattern)]
      .map((match) => parseChineseNumber(match[1]))
      .filter((value): value is number => value !== null),
  );
  return limits.length > 0 ? Math.min(...limits) : null;
}

export function extractExplicitSelectionLimit(
  requirement: AiRegistrationRequirement,
  evidence: AiRegistrationEvidence[],
): number | null {
  const evidenceIds = new Set(requirement.evidenceIds);
  const quotedSourceText = evidence
    .filter((item) => evidenceIds.has(item.id))
    .map((item) => item.quote)
    .join(' ');
  if (!quotedSourceText) return null;
  return extractSelectionLimitFromText(quotedSourceText, isCollectionRequirement(requirement));
}

function hasUnverifiedLimitClaim(requirement: AiRegistrationRequirement): boolean {
  const modelText = `${requirement.name} ${requirement.details}`;
  return extractSelectionLimitFromText(modelText, isCollectionRequirement(requirement)) !== null;
}

export function matchAiRequirements(
  requirements: AiRegistrationRequirement[],
  materials: Material[],
  evidence: AiRegistrationEvidence[] = [],
): AiRequirementMatch[] {
  return requirements.map((requirement, index) => {
    const candidates = materials
      .map((material) => ({ material, score: materialScore(material, requirement) }))
      .filter((entry) => entry.score > 0)
      .sort((left, right) => right.score - left.score || left.material.name.localeCompare(right.material.name, 'zh-CN'))
      .map((entry) => entry.material);
    const defaultsToAll = isCollectionRequirement(requirement);
    const selectionLimit = extractExplicitSelectionLimit(requirement, evidence);
    const defaultCandidates = defaultsToAll
      ? candidates.slice(0, selectionLimit ?? candidates.length)
      : candidates.slice(0, 1);
    return {
      id: `ai-requirement-${index}`,
      requirement,
      included: true,
      candidates,
      selectedMaterialIds: defaultCandidates.map((material) => material.id),
      selectionLimit,
      unverifiedLimitClaim: selectionLimit === null && hasUnverifiedLimitClaim(requirement),
      status: candidates.length === 0 ? 'missing' : candidates.length === 1 ? 'matched' : 'multiple',
    };
  });
}

export function getAiApplyConfirmationItems(matches: AiRequirementMatch[]): AiApplyConfirmationItem[] {
  return matches.flatMap((match) => {
    const items: AiApplyConfirmationItem[] = [];
    if (!match.included) {
      items.push({
        kind: 'ignored',
        requirementId: match.id,
        message: `已忽略：${match.requirement.name}${match.requirement.required ? '（通知标为必需）' : ''}。`,
      });
      return items;
    }
    if (match.status === 'multiple') {
      items.push({
        kind: 'multiple',
        requirementId: match.id,
        message: `${match.requirement.name}：发现 ${match.candidates.length} 份候选，当前选择 ${match.selectedMaterialIds.length} 份，请确认取舍。`,
      });
    }
    if (match.requirement.required && match.selectedMaterialIds.length === 0) {
      items.push({
        kind: 'missing-required',
        requirementId: match.id,
        message: `${match.requirement.name}：必需材料尚未选择，应用后会保留空模块。`,
      });
    }
    if (match.selectionLimit !== null && match.selectedMaterialIds.length >= match.selectionLimit) {
      const exceeded = match.selectedMaterialIds.length > match.selectionLimit;
      items.push({
        kind: 'selection-limit',
        requirementId: match.id,
        message: `${match.requirement.name}：已选择 ${match.selectedMaterialIds.length} 份，${exceeded ? '超过' : '达到'}通知上限 ${match.selectionLimit} 份。`,
      });
    }
    return items;
  });
}

export function buildAiArrangedProject(
  project: ApplicationProject,
  matches: AiRequirementMatch[],
): ApplicationProject {
  const includedMatches = matches.filter((match) => match.included);
  const selectedMaterialIds = new Set(includedMatches.flatMap((match) => match.selectedMaterialIds));
  const aiModules: ProjectModule[] = includedMatches.map((match, position) => {
    const selectedIds = new Set(match.selectedMaterialIds);
    const selected = match.candidates.filter((material) => selectedIds.has(material.id));
    return {
      id: crypto.randomUUID(),
      title: match.requirement.name,
      category: match.requirement.category,
      position,
      required: match.requirement.required,
      enabled: match.requirement.required || selected.length > 0,
      files: selected.map((material, filePosition) => ({
        id: crypto.randomUUID(),
        materialId: material.id,
        position: filePosition,
        material: structuredClone(material),
      })),
    };
  });
  const preservedModules: ProjectModule[] = project.modules.flatMap((module) => {
    const files = module.files.filter((file) => !selectedMaterialIds.has(file.materialId));
    if (module.files.length > 0 && files.length === 0) return [];
    const partiallyUsed = files.length !== module.files.length;
    return [{
      ...structuredClone(module),
      title: partiallyUsed ? `${module.title}（未纳入 AI 编排）` : module.title,
      position: 0,
      required: false,
      enabled: false,
      files: files.map((file, filePosition) => ({ ...structuredClone(file), position: filePosition })),
    }];
  });
  const modules = [...aiModules, ...preservedModules].map((module, position) => ({ ...module, position }));

  return {
    ...structuredClone(project),
    modules,
    updatedAt: new Date().toISOString(),
  };
}
