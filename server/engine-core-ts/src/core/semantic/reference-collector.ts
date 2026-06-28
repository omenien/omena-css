import type { AnalysisEntry } from "../indexing/document-analysis-cache";
import type { SourceExpressionKind } from "../hir/source-types";
import type { StyleDocumentHIR } from "../hir/style-types";
import {
  exactClassValue,
  prefixClassValue,
  type AbstractClassValue,
} from "../abstract-value/class-value-domain";
import { listStyleModulePaths } from "../binder/source-binding-graph";
import type { TypeResolver } from "../ts/type-resolver";
import { readSourceExpressionResolution } from "../query/read-source-expression-resolution";
import { deriveReferenceExpansion, type EdgeCertainty } from "./certainty";
import type { SemanticReferenceSite } from "./reference-types";
import type { EdgeReason } from "./provenance";

export interface SemanticReferenceCollectionContext {
  readonly styleDocumentForPath: (path: string) => StyleDocumentHIR | null;
  readonly typeResolver: TypeResolver;
  readonly workspaceRoot: string;
  readonly filePath: string;
  readonly settingsKey: string;
}

export interface SemanticModuleUsageSite {
  readonly refId: string;
  readonly uri: string;
  readonly filePath: string;
  readonly range: SemanticReferenceSite["range"];
  readonly origin: SemanticReferenceSite["origin"];
  readonly scssModulePath: string;
  readonly expressionKind: SourceExpressionKind;
  readonly hasResolvedTargets: boolean;
  readonly isDynamic: boolean;
}

export interface SemanticContributionDeps {
  readonly workspaceRoot: string;
  readonly settingsKey: string;
  readonly stylePaths: readonly string[];
  readonly sourcePaths: readonly string[];
}

export function collectSemanticReferenceContribution(
  uri: string,
  entry: AnalysisEntry,
  ctx: SemanticReferenceCollectionContext,
): {
  readonly referenceSites: readonly SemanticReferenceSite[];
  readonly moduleUsages: readonly SemanticModuleUsageSite[];
  readonly deps: SemanticContributionDeps;
} {
  const bindingGraph = entry.sourceBindingGraph;
  const styleDocumentsByPath = new Map<string, StyleDocumentHIR>();

  for (const scssModulePath of listStyleModulePaths(bindingGraph)) {
    const styleDocument = ctx.styleDocumentForPath(scssModulePath);
    if (styleDocument) {
      styleDocumentsByPath.set(scssModulePath, styleDocument);
    }
  }

  if (styleDocumentsByPath.size === 0) {
    return {
      referenceSites: [],
      moduleUsages: [],
      deps: {
        workspaceRoot: ctx.workspaceRoot,
        settingsKey: ctx.settingsKey,
        stylePaths: [],
        sourcePaths: entry.sourceDependencyPaths,
      },
    };
  }

  const referenceSites: SemanticReferenceSite[] = [];
  const moduleUsages = entry.sourceDocument.classExpressions
    .map((expr) => {
      const styleDocument = styleDocumentsByPath.get(expr.scssModulePath) ?? null;
      const resolution = readSourceExpressionResolution(
        {
          expression: expr,
          styleDocument,
        },
        {
          typeResolver: ctx.typeResolver,
          filePath: ctx.filePath,
          workspaceRoot: ctx.workspaceRoot,
          sourceBinder: entry.sourceBinder,
          sourceBindingGraph: entry.sourceBindingGraph,
          classValueUniverses: entry.classValueUniverses,
        },
      );
      if (resolution.styleDocument) {
        for (const selector of resolution.selectors) {
          referenceSites.push(
            toReferenceSite(
              uri,
              ctx.filePath,
              expr,
              resolution.styleDocument.filePath,
              selector.canonicalName,
              resolution.selectorCertainty,
              reasonForExpression(expr, resolution.reason),
              abstractValueForExpression(expr, resolution.abstractValue),
            ),
          );
        }
      }
      return {
        refId: expr.id,
        uri,
        filePath: ctx.filePath,
        range: expr.range,
        origin: expr.origin,
        scssModulePath: expr.scssModulePath,
        expressionKind: expr.kind,
        hasResolvedTargets: resolution.selectors.length > 0,
        isDynamic: expr.kind === "template" || expr.kind === "symbolRef",
      } satisfies SemanticModuleUsageSite;
    })
    .toSorted(compareModuleUsages);

  return {
    referenceSites: referenceSites.toSorted(compareSites),
    moduleUsages,
    deps: {
      workspaceRoot: ctx.workspaceRoot,
      settingsKey: ctx.settingsKey,
      stylePaths: [...styleDocumentsByPath.keys()].toSorted(),
      sourcePaths: entry.sourceDependencyPaths,
    },
  };
}

function compareSites(a: SemanticReferenceSite, b: SemanticReferenceSite): number {
  return (
    a.selectorFilePath.localeCompare(b.selectorFilePath) ||
    a.canonicalName.localeCompare(b.canonicalName) ||
    a.range.start.line - b.range.start.line ||
    a.range.start.character - b.range.start.character ||
    a.refId.localeCompare(b.refId)
  );
}

function compareModuleUsages(a: SemanticModuleUsageSite, b: SemanticModuleUsageSite): number {
  return (
    a.scssModulePath.localeCompare(b.scssModulePath) ||
    a.range.start.line - b.range.start.line ||
    a.range.start.character - b.range.start.character ||
    a.refId.localeCompare(b.refId)
  );
}

function toReferenceSite(
  uri: string,
  filePath: string,
  expression: AnalysisEntry["sourceDocument"]["classExpressions"][number],
  selectorFilePath: string,
  canonicalName: string,
  selectorCertainty: EdgeCertainty,
  reason: EdgeReason,
  abstractValue?: AbstractClassValue,
): SemanticReferenceSite {
  return {
    refId: expression.id,
    selectorId: `selector:${selectorFilePath}:${canonicalName}`,
    filePath,
    uri,
    range: expression.range,
    origin: expression.origin,
    scssModulePath: expression.scssModulePath,
    selectorFilePath,
    canonicalName,
    className:
      expression.kind === "literal" || expression.kind === "styleAccess"
        ? expression.className
        : canonicalName,
    selectorCertainty,
    reason,
    expansion: deriveReferenceExpansion(expression.kind),
    ...(abstractValue ? { abstractValue } : {}),
  };
}

function reasonForExpression(
  expression: AnalysisEntry["sourceDocument"]["classExpressions"][number],
  dynamicReason?: EdgeReason,
): EdgeReason {
  switch (expression.kind) {
    case "literal":
      return "literal";
    case "styleAccess":
      return "styleAccess";
    case "template":
      return "templatePrefix";
    case "symbolRef":
      return dynamicReason ?? "typeUnion";
    default:
      expression satisfies never;
      return "typeUnion";
  }
}

function abstractValueForExpression(
  expression: AnalysisEntry["sourceDocument"]["classExpressions"][number],
  abstractValue?: AbstractClassValue,
): AbstractClassValue | undefined {
  switch (expression.kind) {
    case "literal":
    case "styleAccess":
      return exactClassValue(expression.className);
    case "template":
      return prefixClassValue(expression.staticPrefix);
    case "symbolRef":
      return abstractValue;
    default:
      expression satisfies never;
      return abstractValue;
  }
}
