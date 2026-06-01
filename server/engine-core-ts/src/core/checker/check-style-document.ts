import path from "node:path";
import type { ComposesRef, Range } from "@omena/shared";
import type { StyleDocumentHIR } from "../hir/style-types";
import {
  findCustomPropertyDeclByName,
  listCustomPropertyModuleUseDeclTargets,
  resolveCustomPropertyDeclTarget,
  readStyleModuleUsageSummary,
  resolveSassWildcardSymbolTarget,
  type SassModulePathAliasResolver,
  type SassModuleResolutionOptions,
} from "../query";
import type { SemanticWorkspaceReferenceIndex, StyleDependencyGraph } from "../semantic";
import type { StyleCheckerFinding } from "./contracts";
import { runCheckerRules, type CheckerRule } from "./rule-template";

export interface StyleDocumentCheckParams {
  readonly scssPath: string;
  readonly styleDocument: StyleDocumentHIR;
}

export interface StyleDocumentCheckEnv {
  readonly semanticReferenceIndex: SemanticWorkspaceReferenceIndex;
  readonly styleDependencyGraph?: StyleDependencyGraph;
  readonly styleDocumentForPath?: (filePath: string) => StyleDocumentHIR | null;
  readonly aliasResolver?: SassModulePathAliasResolver;
  readonly readFile?: (path: string) => string | null;
}

export interface StyleDocumentCheckOptions {
  readonly includeUnusedSelectors?: boolean;
  readonly includeComposesResolution?: boolean;
  readonly includeKeyframesResolution?: boolean;
  readonly includeSassSymbolResolution?: boolean;
}

export function checkStyleDocument(
  params: StyleDocumentCheckParams,
  env: StyleDocumentCheckEnv,
  options: StyleDocumentCheckOptions = {},
): readonly StyleCheckerFinding[] {
  return runCheckerRules(STYLE_DOCUMENT_RULES, { params, env, options });
}

const STYLE_DOCUMENT_RULES: readonly CheckerRule<
  StyleDocumentCheckParams,
  StyleDocumentCheckEnv,
  StyleDocumentCheckOptions,
  StyleCheckerFinding
>[] = [
  checkUnusedSelectorsRule,
  checkComposesResolutionRule,
  checkImportedValuesRule,
  checkSassSymbolResolutionRule,
  checkCustomPropertyResolutionRule,
  checkMissingKeyframesRule,
];

function checkUnusedSelectorsRule({
  params,
  env,
  options,
}: {
  readonly params: StyleDocumentCheckParams;
  readonly env: StyleDocumentCheckEnv;
  readonly options: StyleDocumentCheckOptions;
}): readonly StyleCheckerFinding[] {
  if (!(options.includeUnusedSelectors ?? true)) return [];
  const findings: StyleCheckerFinding[] = [];
  const usage = readStyleModuleUsageSummary(
    params.scssPath,
    params.styleDocument,
    env.semanticReferenceIndex,
    env.styleDependencyGraph,
  );
  for (const selector of usage.unusedSelectors) {
    findings.push({
      category: "style",
      code: "unused-selector",
      severity: "hint",
      range: selector.range,
      selectorFilePath: params.scssPath,
      canonicalName: selector.canonicalName,
    });
  }

  return findings;
}

function checkComposesResolutionRule({
  params,
  env,
  options,
}: {
  readonly params: StyleDocumentCheckParams;
  readonly env: StyleDocumentCheckEnv;
  readonly options: StyleDocumentCheckOptions;
}): readonly StyleCheckerFinding[] {
  if (!(options.includeComposesResolution ?? true) || !env.styleDocumentForPath) return [];
  const findings: StyleCheckerFinding[] = [];
  for (const selector of params.styleDocument.selectors) {
    if (selector.viewKind !== "canonical") continue;
    for (const ref of selector.composes) {
      if (ref.fromGlobal) continue;

      const targetFilePath = ref.from
        ? path.resolve(path.dirname(params.styleDocument.filePath), ref.from)
        : params.styleDocument.filePath;
      const targetDocument = env.styleDocumentForPath(targetFilePath);
      if (!targetDocument) {
        findings.push({
          category: "style",
          code: "missing-composed-module",
          severity: "warning",
          range: rangeForComposesRef(selector, ref),
          selectorFilePath: params.styleDocument.filePath,
          ...(ref.from ? { fromSpecifier: ref.from } : {}),
          targetFilePath,
        });
        continue;
      }

      for (const missing of unresolvedComposedClasses(selector, ref, targetDocument)) {
        findings.push({
          category: "style",
          code: "missing-composed-selector",
          severity: "warning",
          range: missing.range,
          selectorFilePath: params.styleDocument.filePath,
          ...(ref.from ? { fromSpecifier: ref.from } : {}),
          targetFilePath,
          className: missing.className,
        });
      }
    }
  }

  return findings;
}

function checkImportedValuesRule({
  params,
  env,
}: {
  readonly params: StyleDocumentCheckParams;
  readonly env: StyleDocumentCheckEnv;
  readonly options: StyleDocumentCheckOptions;
}): readonly StyleCheckerFinding[] {
  if (!env.styleDocumentForPath) return [];
  const findings: StyleCheckerFinding[] = [];
  const reportedMissingValueModules = new Set<string>();

  for (const valueImport of params.styleDocument.valueImports) {
    const targetFilePath = path.resolve(
      path.dirname(params.styleDocument.filePath),
      valueImport.from,
    );
    const targetDocument = env.styleDocumentForPath(targetFilePath);
    if (!targetDocument) {
      const moduleKey = `${valueImport.from}:${targetFilePath}`;
      if (!reportedMissingValueModules.has(moduleKey)) {
        reportedMissingValueModules.add(moduleKey);
        findings.push({
          category: "style",
          code: "missing-value-module",
          severity: "warning",
          range: valueImport.range,
          selectorFilePath: params.styleDocument.filePath,
          fromSpecifier: valueImport.from,
          targetFilePath,
        });
      }
      continue;
    }

    const targetValueDecl = targetDocument.valueDecls.find(
      (valueDecl) => valueDecl.name === valueImport.importedName,
    );
    if (!targetValueDecl) {
      findings.push({
        category: "style",
        code: "missing-imported-value",
        severity: "warning",
        range: valueImport.range,
        selectorFilePath: params.styleDocument.filePath,
        fromSpecifier: valueImport.from,
        targetFilePath,
        importedName: valueImport.importedName,
        localName: valueImport.name,
      });
      continue;
    }
  }

  return findings;
}

function checkMissingKeyframesRule({
  params,
  options,
}: {
  readonly params: StyleDocumentCheckParams;
  readonly env: StyleDocumentCheckEnv;
  readonly options: StyleDocumentCheckOptions;
}): readonly StyleCheckerFinding[] {
  if (!(options.includeKeyframesResolution ?? true)) return [];
  return findMissingKeyframes(params);
}

function checkSassSymbolResolutionRule({
  params,
  env,
  options,
}: {
  readonly params: StyleDocumentCheckParams;
  readonly env: StyleDocumentCheckEnv;
  readonly options: StyleDocumentCheckOptions;
}): readonly StyleCheckerFinding[] {
  if (!(options.includeSassSymbolResolution ?? true)) return [];
  const findings: StyleCheckerFinding[] = [];
  const reported = new Set<string>();

  for (const symbol of params.styleDocument.sassSymbols) {
    if (symbol.resolution !== "unresolved") continue;
    if (
      env.styleDocumentForPath &&
      resolveSassWildcardSymbolTarget(
        env.styleDocumentForPath,
        params.styleDocument.filePath,
        params.styleDocument,
        symbol,
        env.aliasResolver,
        sassModuleResolutionOptions(env.readFile),
      )
    ) {
      continue;
    }
    const key = [
      symbol.symbolKind,
      symbol.name,
      symbol.role,
      symbol.range.start.line,
      symbol.range.start.character,
      symbol.range.end.line,
      symbol.range.end.character,
    ].join(":");
    if (reported.has(key)) continue;
    reported.add(key);
    findings.push({
      category: "style",
      code: "missing-sass-symbol",
      severity: "warning",
      range: symbol.range,
      selectorFilePath: params.styleDocument.filePath,
      selectorName: symbol.selectorName,
      ...(symbol.syntax ? { symbolSyntax: symbol.syntax } : {}),
      symbolKind: symbol.symbolKind,
      symbolName: symbol.name,
      symbolRole: symbol.role,
    });
  }

  return findings;
}

function checkCustomPropertyResolutionRule({
  params,
  env,
}: {
  readonly params: StyleDocumentCheckParams;
  readonly env: StyleDocumentCheckEnv;
  readonly options: StyleDocumentCheckOptions;
}): readonly StyleCheckerFinding[] {
  const knownDeclCount =
    params.styleDocument.customPropertyDecls.length +
    (env.styleDocumentForPath
      ? listCustomPropertyModuleUseDeclTargets(
          env.styleDocumentForPath,
          params.styleDocument.filePath,
          params.styleDocument,
          env.aliasResolver,
          sassModuleResolutionOptions(env.readFile),
        ).length
      : 0) +
    (env.styleDependencyGraph?.getAllCustomPropertyDecls().length ?? 0);
  if (knownDeclCount === 0) return [];

  const findings: StyleCheckerFinding[] = [];
  const reported = new Set<string>();
  for (const ref of params.styleDocument.customPropertyRefs) {
    if (findCustomPropertyDeclByName(params.styleDocument, ref.name)) continue;
    if (
      env.styleDocumentForPath &&
      resolveCustomPropertyDeclTarget(
        env.styleDocumentForPath,
        params.styleDocument.filePath,
        params.styleDocument,
        ref,
        env.styleDependencyGraph,
        env.aliasResolver,
        sassModuleResolutionOptions(env.readFile),
      )
    ) {
      continue;
    }
    if (
      !env.styleDocumentForPath &&
      env.styleDependencyGraph?.getCustomPropertyDecls(ref.name).length
    ) {
      continue;
    }

    const key = [
      ref.name,
      ref.range.start.line,
      ref.range.start.character,
      ref.range.end.line,
      ref.range.end.character,
    ].join(":");
    if (reported.has(key)) continue;
    reported.add(key);
    findings.push({
      category: "style",
      code: "missing-custom-property",
      severity: "warning",
      range: ref.range,
      selectorFilePath: params.styleDocument.filePath,
      propertyName: ref.name,
    });
  }

  return findings;
}

function sassModuleResolutionOptions(
  readFile: StyleDocumentCheckEnv["readFile"],
): SassModuleResolutionOptions {
  return readFile ? { readFile } : {};
}

function findMissingKeyframes(
  params: StyleDocumentCheckParams,
): readonly Extract<StyleCheckerFinding, { code: "missing-keyframes" }>[] {
  const findings: Extract<StyleCheckerFinding, { code: "missing-keyframes" }>[] = [];
  const declared = new Set(params.styleDocument.keyframes.map((keyframes) => keyframes.name));

  for (const animationNameRef of params.styleDocument.animationNameRefs) {
    if (declared.has(animationNameRef.name)) continue;
    findings.push({
      category: "style",
      code: "missing-keyframes",
      severity: "warning",
      range: animationNameRef.range,
      selectorFilePath: params.styleDocument.filePath,
      animationName: animationNameRef.name,
    });
  }

  return findings;
}

function unresolvedComposedClasses(
  selector: StyleDocumentHIR["selectors"][number],
  ref: ComposesRef,
  targetDocument: StyleDocumentHIR,
): ReadonlyArray<{ className: string; range: Range }> {
  const unresolved: Array<{ className: string; range: Range }> = [];
  const tokenByName = new Map(
    ref.classTokens?.map((token) => [token.className, token.range]) ?? [],
  );

  for (const className of ref.classNames) {
    const targetSelector =
      targetDocument.selectors.find(
        (candidate) => candidate.canonicalName === className && candidate.viewKind === "canonical",
      ) ?? targetDocument.selectors.find((candidate) => candidate.canonicalName === className);
    if (targetSelector) continue;
    const range = tokenByName.get(className) ?? rangeForComposesRef(selector, ref);
    unresolved.push({ className, range });
  }

  return unresolved;
}

function rangeForComposesRef(
  selector: StyleDocumentHIR["selectors"][number],
  ref: ComposesRef,
): Range {
  return ref.classTokens?.[0]?.range ?? selector.range;
}
