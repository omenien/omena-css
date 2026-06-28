import type { DocumentAnalysisCache } from "../indexing/document-analysis-cache";
import type { StyleDocumentHIR } from "../hir/style-types";
import { findInvalidClassReference } from "../query";
import type { InvalidClassReferenceQueryEnv } from "../query/find-invalid-class-references";
import type { TypeResolver } from "../ts/type-resolver";
import type { SourceCheckerFinding } from "./contracts";
import { runCheckerRules, type CheckerRule } from "./rule-template";

export interface SourceDocumentCheckParams {
  readonly documentUri: string;
  readonly content: string;
  readonly filePath: string;
  readonly version: number;
}

export interface SourceDocumentCheckEnv {
  readonly analysisCache: DocumentAnalysisCache;
  readonly styleDocumentForPath: (path: string) => StyleDocumentHIR | null;
  readonly typeResolver: TypeResolver;
  readonly workspaceRoot: string;
  readonly resolveSymbolValues?: InvalidClassReferenceQueryEnv["resolveSymbolValues"];
}

export interface SourceDocumentCheckOptions {
  readonly includeMissingModule?: boolean;
  readonly logError?: (message: string, err: unknown) => void;
}

export function checkSourceDocument(
  params: SourceDocumentCheckParams,
  env: SourceDocumentCheckEnv,
  options: SourceDocumentCheckOptions = {},
): readonly SourceCheckerFinding[] {
  return runCheckerRules(SOURCE_DOCUMENT_RULES, { params, env, options });
}

const SOURCE_DOCUMENT_RULES: readonly CheckerRule<
  SourceDocumentCheckParams,
  SourceDocumentCheckEnv,
  SourceDocumentCheckOptions,
  SourceCheckerFinding
>[] = [checkMissingModulesRule, checkInvalidClassReferencesRule];

function checkMissingModulesRule({
  params,
  env,
  options,
}: {
  readonly params: SourceDocumentCheckParams;
  readonly env: SourceDocumentCheckEnv;
  readonly options: SourceDocumentCheckOptions;
}): readonly SourceCheckerFinding[] {
  if (!(options.includeMissingModule ?? true)) return [];
  const entry = env.analysisCache.get(
    params.documentUri,
    params.content,
    params.filePath,
    params.version,
  );
  const findings: SourceCheckerFinding[] = [];

  for (const imp of entry.stylesBindings.values()) {
    if (imp.kind !== "missing") continue;
    findings.push({
      category: "source",
      code: "missing-module",
      severity: "warning",
      range: imp.range,
      specifier: imp.specifier,
      absolutePath: imp.absolutePath,
    });
  }

  return findings;
}

function checkInvalidClassReferencesRule({
  params,
  env,
  options,
}: {
  readonly params: SourceDocumentCheckParams;
  readonly env: SourceDocumentCheckEnv;
  readonly options: SourceDocumentCheckOptions;
}): readonly SourceCheckerFinding[] {
  const entry = env.analysisCache.get(
    params.documentUri,
    params.content,
    params.filePath,
    params.version,
  );
  const findings: SourceCheckerFinding[] = [];

  for (const expression of entry.sourceDocument.classExpressions) {
    if (expression.origin !== "cxCall") continue;
    try {
      const styleDocument = env.styleDocumentForPath(expression.scssModulePath);
      if (!styleDocument) continue;
      const finding = findInvalidClassReference(expression, styleDocument, {
        typeResolver: env.typeResolver,
        filePath: params.filePath,
        workspaceRoot: env.workspaceRoot,
        sourceBinder: entry.sourceBinder,
        sourceBindingGraph: entry.sourceBindingGraph,
        classValueUniverses: entry.classValueUniverses,
        ...(env.resolveSymbolValues ? { resolveSymbolValues: env.resolveSymbolValues } : {}),
      });
      if (!finding) continue;
      findings.push(mapInvalidClassFinding(finding, styleDocument));
    } catch (err) {
      options.logError?.("diagnostics per-call validation failed", err);
    }
  }

  return findings;
}

function mapInvalidClassFinding(
  finding: NonNullable<ReturnType<typeof findInvalidClassReference>>,
  styleDocument: StyleDocumentHIR,
): SourceCheckerFinding {
  switch (finding.kind) {
    case "missingStaticClass":
      return {
        category: "source",
        code: "missing-static-class",
        severity: "warning",
        range: finding.range,
        scssModulePath: finding.expression.scssModulePath,
        className: finding.expression.className,
        ...(finding.suggestion ? { suggestion: finding.suggestion } : {}),
      };
    case "missingTemplatePrefix":
      return {
        category: "source",
        code: "missing-template-prefix",
        severity: "warning",
        range: finding.range,
        scssModulePath: finding.expression.scssModulePath,
        staticPrefix: finding.expression.staticPrefix,
      };
    case "missingResolvedClassValues":
      return {
        category: "source",
        code: "missing-resolved-class-values",
        severity: "warning",
        range: finding.range,
        scssModulePath: styleDocument.filePath,
        missingValues: finding.missingValues,
        abstractValue: finding.abstractValue,
        valueCertainty: finding.valueCertainty,
        selectorCertainty: finding.selectorCertainty,
        reason: finding.reason,
        ...(finding.valueDomainDerivation
          ? { valueDomainDerivation: finding.valueDomainDerivation }
          : {}),
      };
    case "missingResolvedClassDomain":
      return {
        category: "source",
        code: "missing-resolved-class-domain",
        severity: "warning",
        range: finding.range,
        scssModulePath: styleDocument.filePath,
        abstractValue: finding.abstractValue,
        valueCertainty: finding.valueCertainty,
        selectorCertainty: finding.selectorCertainty,
        reason: finding.reason,
        ...(finding.valueDomainDerivation
          ? { valueDomainDerivation: finding.valueDomainDerivation }
          : {}),
      };
    default:
      finding satisfies never;
      return finding;
  }
}
