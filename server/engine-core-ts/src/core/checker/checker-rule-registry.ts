import type { CheckerFinding, CheckerSeverity } from "./contracts";

export type CheckerRuleCode = CheckerFinding["code"];
export type CheckerRuleTier = "s-tier" | "t-tier";
export type CheckerRulePreset = "recommended" | "strict";
export type CheckerRuleFixability = "none" | "codeAction" | "autofix";

export interface CheckerRuleDescriptor {
  readonly code: CheckerRuleCode;
  readonly category: CheckerFinding["category"];
  readonly tier: CheckerRuleTier;
  readonly defaultSeverity: CheckerSeverity;
  readonly fixability: CheckerRuleFixability;
  readonly presets: readonly CheckerRulePreset[];
  readonly description: string;
}

const CHECKER_RULE_DESCRIPTORS: readonly CheckerRuleDescriptor[] = [
  {
    code: "missing-module",
    category: "source",
    tier: "s-tier",
    defaultSeverity: "warning",
    fixability: "codeAction",
    presets: ["recommended"],
    description: "Report unresolved CSS Module imports from source files.",
  },
  {
    code: "missing-static-class",
    category: "source",
    tier: "s-tier",
    defaultSeverity: "warning",
    fixability: "codeAction",
    presets: ["recommended"],
    description: "Report static class names that do not exist in the target CSS Module.",
  },
  {
    code: "missing-template-prefix",
    category: "source",
    tier: "s-tier",
    defaultSeverity: "warning",
    fixability: "none",
    presets: ["recommended"],
    description: "Report template-literal class prefixes that match no target selector.",
  },
  {
    code: "missing-resolved-class-values",
    category: "source",
    tier: "s-tier",
    defaultSeverity: "warning",
    fixability: "none",
    presets: ["recommended"],
    description: "Report finite dynamic class values that resolve outside the selector set.",
  },
  {
    code: "missing-resolved-class-domain",
    category: "source",
    tier: "s-tier",
    defaultSeverity: "warning",
    fixability: "none",
    presets: ["recommended"],
    description: "Report dynamic class domains that cannot be proven against known selectors.",
  },
  {
    code: "unused-selector",
    category: "style",
    tier: "t-tier",
    defaultSeverity: "hint",
    fixability: "none",
    presets: ["strict"],
    description: "Report CSS Module selectors with no indexed source references.",
  },
  {
    code: "missing-composed-module",
    category: "style",
    tier: "t-tier",
    defaultSeverity: "warning",
    fixability: "codeAction",
    presets: ["recommended"],
    description: "Report unresolved composes-from module specifiers.",
  },
  {
    code: "missing-composed-selector",
    category: "style",
    tier: "t-tier",
    defaultSeverity: "warning",
    fixability: "codeAction",
    presets: ["recommended"],
    description: "Report composed class names missing from the resolved target module.",
  },
  {
    code: "missing-value-module",
    category: "style",
    tier: "t-tier",
    defaultSeverity: "warning",
    fixability: "codeAction",
    presets: ["recommended"],
    description: "Report unresolved Sass/CSS @value module specifiers.",
  },
  {
    code: "missing-imported-value",
    category: "style",
    tier: "t-tier",
    defaultSeverity: "warning",
    fixability: "codeAction",
    presets: ["recommended"],
    description: "Report @value names missing from the resolved target module.",
  },
  {
    code: "missing-keyframes",
    category: "style",
    tier: "t-tier",
    defaultSeverity: "warning",
    fixability: "codeAction",
    presets: ["recommended"],
    description: "Report animation names that do not resolve to local @keyframes.",
  },
  {
    code: "missing-custom-property",
    category: "style",
    tier: "t-tier",
    defaultSeverity: "warning",
    fixability: "none",
    presets: ["strict"],
    description: "Report CSS custom property references with no indexed declaration.",
  },
  {
    code: "missing-sass-symbol",
    category: "style",
    tier: "t-tier",
    defaultSeverity: "warning",
    fixability: "none",
    presets: ["recommended"],
    description: "Report unresolved Sass/Less variable, mixin, and function references.",
  },
];

const CHECKER_RULE_DESCRIPTOR_BY_CODE = new Map(
  CHECKER_RULE_DESCRIPTORS.map((descriptor) => [descriptor.code, descriptor]),
);

export function listCheckerRuleDescriptors(): readonly CheckerRuleDescriptor[] {
  return CHECKER_RULE_DESCRIPTORS;
}

export function listCheckerRuleCodes(): readonly CheckerRuleCode[] {
  return CHECKER_RULE_DESCRIPTORS.map((descriptor) => descriptor.code);
}

export function getCheckerRuleDescriptor(code: CheckerRuleCode): CheckerRuleDescriptor {
  return CHECKER_RULE_DESCRIPTOR_BY_CODE.get(code)!;
}

export function isCheckerRuleCode(value: string): value is CheckerRuleCode {
  return CHECKER_RULE_DESCRIPTOR_BY_CODE.has(value as CheckerRuleCode);
}
