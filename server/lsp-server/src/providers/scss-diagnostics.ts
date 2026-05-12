import { DiagnosticSeverity, DiagnosticTag, type Diagnostic } from "vscode-languageserver/node";
import { type StyleCheckerFinding } from "../../../engine-core-ts/src/core/checker";
import { formatCheckerFinding } from "../../../engine-core-ts/src/checker-surface";
import type { StyleDocumentHIR } from "../../../engine-core-ts/src/core/hir/style-types";
import { pathToFileUrl } from "../../../engine-core-ts/src/core/util/text-utils";
import {
  buildCreateCustomPropertyActionData,
  buildCreateKeyframesActionData,
  buildCreateSassSymbolActionData,
  buildCreateSelectorActionData,
  buildCreateValueActionData,
} from "../../../engine-host-node/src/code-action-data";
import {
  resolveStyleDiagnosticFindingsAsync,
  resolveStyleDiagnosticFindings,
  type StyleDiagnosticsQueryOptions,
} from "../../../engine-host-node/src/style-diagnostics-query";
import type { RustSelectedQueryBackendJsonRunnerAsync } from "../../../engine-host-node/src/selected-query-backend";
import type { ProviderDeps } from "./provider-deps";
import { toLspRange } from "./lsp-adapters";

/**
 * Compute "unused selector" diagnostics for a single SCSS module file.
 *
 * Caller is responsible for gating behind IndexerWorker.ready so
 * this function is never called before the initial index walk
 * completes.
 */
export function computeScssUnusedDiagnostics(
  scssPath: string,
  styleDocument: StyleDocumentHIR,
  semanticReferenceIndex: ProviderDeps["semanticReferenceIndex"],
  styleDependencyGraph?: ProviderDeps["styleDependencyGraph"],
  styleDocumentForPath?: (filePath: string) => StyleDocumentHIR | null,
  runtimeDeps?: Pick<
    ProviderDeps,
    | "analysisCache"
    | "readStyleFile"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "aliasResolver"
  > & {
    readonly env?: NodeJS.ProcessEnv;
    readonly styleSemanticGraphCache?: StyleDiagnosticsQueryOptions["styleSemanticGraphCache"];
    readonly styleSemanticGraphBatchOutputCache?: StyleDiagnosticsQueryOptions["styleSemanticGraphBatchOutputCache"];
    readonly selectorUsagePayloadCache?: StyleDiagnosticsQueryOptions["selectorUsagePayloadCache"];
    readonly runRustSelectedQueryBackendJsonAsync?: RustSelectedQueryBackendJsonRunnerAsync;
  },
): Diagnostic[] | Promise<Diagnostic[]> {
  const queryOptions =
    runtimeDeps?.env ||
    runtimeDeps?.styleSemanticGraphCache ||
    runtimeDeps?.styleSemanticGraphBatchOutputCache ||
    runtimeDeps?.selectorUsagePayloadCache ||
    runtimeDeps?.runRustSelectedQueryBackendJsonAsync
      ? {
          ...(runtimeDeps?.env ? { env: runtimeDeps.env } : {}),
          ...(runtimeDeps?.styleSemanticGraphCache
            ? { styleSemanticGraphCache: runtimeDeps.styleSemanticGraphCache }
            : {}),
          ...(runtimeDeps?.styleSemanticGraphBatchOutputCache
            ? { styleSemanticGraphBatchOutputCache: runtimeDeps.styleSemanticGraphBatchOutputCache }
            : {}),
          ...(runtimeDeps?.selectorUsagePayloadCache
            ? { selectorUsagePayloadCache: runtimeDeps.selectorUsagePayloadCache }
            : {}),
          ...(runtimeDeps?.runRustSelectedQueryBackendJsonAsync
            ? {
                runRustSelectedQueryBackendJsonAsync:
                  runtimeDeps.runRustSelectedQueryBackendJsonAsync,
              }
            : {}),
        }
      : undefined;
  if (runtimeDeps?.runRustSelectedQueryBackendJsonAsync) {
    return resolveStyleDiagnosticFindingsAsync(
      { scssPath, styleDocument },
      {
        ...(runtimeDeps?.analysisCache ? { analysisCache: runtimeDeps.analysisCache } : {}),
        ...(runtimeDeps?.readStyleFile ? { readStyleFile: runtimeDeps.readStyleFile } : {}),
        semanticReferenceIndex,
        ...(styleDependencyGraph ? { styleDependencyGraph } : {}),
        ...(styleDocumentForPath ? { styleDocumentForPath } : {}),
        ...(runtimeDeps?.typeResolver ? { typeResolver: runtimeDeps.typeResolver } : {}),
        ...(runtimeDeps?.workspaceRoot ? { workspaceRoot: runtimeDeps.workspaceRoot } : {}),
        ...(runtimeDeps?.settings ? { settings: runtimeDeps.settings } : {}),
        ...(runtimeDeps?.aliasResolver ? { aliasResolver: runtimeDeps.aliasResolver } : {}),
        ...(runtimeDeps?.styleSemanticGraphCache
          ? { styleSemanticGraphCache: runtimeDeps.styleSemanticGraphCache }
          : {}),
        ...(runtimeDeps?.styleSemanticGraphBatchOutputCache
          ? { styleSemanticGraphBatchOutputCache: runtimeDeps.styleSemanticGraphBatchOutputCache }
          : {}),
        ...(runtimeDeps?.selectorUsagePayloadCache
          ? { selectorUsagePayloadCache: runtimeDeps.selectorUsagePayloadCache }
          : {}),
      },
      queryOptions,
    ).then((findings) =>
      findings.map((finding) =>
        toDiagnostic(finding, styleDocument, styleDocumentForPath, runtimeDeps?.readStyleFile),
      ),
    );
  }
  return resolveStyleDiagnosticFindings(
    { scssPath, styleDocument },
    {
      ...(runtimeDeps?.analysisCache ? { analysisCache: runtimeDeps.analysisCache } : {}),
      ...(runtimeDeps?.readStyleFile ? { readStyleFile: runtimeDeps.readStyleFile } : {}),
      semanticReferenceIndex,
      ...(styleDependencyGraph ? { styleDependencyGraph } : {}),
      ...(styleDocumentForPath ? { styleDocumentForPath } : {}),
      ...(runtimeDeps?.typeResolver ? { typeResolver: runtimeDeps.typeResolver } : {}),
      ...(runtimeDeps?.workspaceRoot ? { workspaceRoot: runtimeDeps.workspaceRoot } : {}),
      ...(runtimeDeps?.settings ? { settings: runtimeDeps.settings } : {}),
      ...(runtimeDeps?.aliasResolver ? { aliasResolver: runtimeDeps.aliasResolver } : {}),
      ...(runtimeDeps?.styleSemanticGraphCache
        ? { styleSemanticGraphCache: runtimeDeps.styleSemanticGraphCache }
        : {}),
      ...(runtimeDeps?.styleSemanticGraphBatchOutputCache
        ? { styleSemanticGraphBatchOutputCache: runtimeDeps.styleSemanticGraphBatchOutputCache }
        : {}),
      ...(runtimeDeps?.selectorUsagePayloadCache
        ? { selectorUsagePayloadCache: runtimeDeps.selectorUsagePayloadCache }
        : {}),
    },
    queryOptions,
  ).map((finding) =>
    toDiagnostic(finding, styleDocument, styleDocumentForPath, runtimeDeps?.readStyleFile),
  );
}

function toDiagnostic(
  finding: StyleCheckerFinding,
  styleDocument: StyleDocumentHIR,
  styleDocumentForPath?: (filePath: string) => StyleDocumentHIR | null,
  readStyleFile?: (filePath: string) => string | null,
): Diagnostic {
  switch (finding.code) {
    case "unused-selector":
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Hint,
        source: "css-module-explainer",
        message: formatCheckerFinding(finding, ""),
        tags: [DiagnosticTag.Unnecessary],
      };
    case "missing-composed-module":
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Warning,
        source: "css-module-explainer",
        message: formatCheckerFinding(finding, ""),
        data: {
          createModuleFile: {
            uri: pathToFileUrl(finding.targetFilePath),
          },
        },
      };
    case "missing-composed-selector": {
      const targetDocument = styleDocumentForPath?.(finding.targetFilePath);
      const data = targetDocument
        ? {
            createSelector: buildCreateSelectorActionData(
              finding.className,
              finding.targetFilePath,
              targetDocument,
              readStyleFile?.(finding.targetFilePath) ?? undefined,
            ),
          }
        : {};
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Warning,
        source: "css-module-explainer",
        message: formatCheckerFinding(finding, ""),
        data,
      };
    }
    case "missing-value-module":
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Warning,
        source: "css-module-explainer",
        message: formatCheckerFinding(finding, ""),
        data: {
          createModuleFile: {
            uri: pathToFileUrl(finding.targetFilePath),
          },
        },
      };
    case "missing-imported-value":
      const targetDocument = styleDocumentForPath?.(finding.targetFilePath);
      const data = targetDocument
        ? {
            createValue: buildCreateValueActionData(
              finding.importedName,
              finding.targetFilePath,
              targetDocument,
              readStyleFile?.(finding.targetFilePath) ?? undefined,
            ),
          }
        : {};
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Warning,
        source: "css-module-explainer",
        message: formatCheckerFinding(finding, ""),
        data,
      };
    case "missing-keyframes":
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Warning,
        source: "css-module-explainer",
        message: formatCheckerFinding(finding, ""),
        data: {
          createKeyframes: buildCreateKeyframesActionData(
            finding.animationName,
            finding.selectorFilePath,
            styleDocument,
          ),
        },
      };
    case "missing-custom-property":
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Warning,
        source: "css-module-explainer",
        message: formatCheckerFinding(finding, ""),
        data: {
          createCustomProperty: buildCreateCustomPropertyActionData(
            finding.propertyName,
            finding.selectorFilePath,
            styleDocument,
          ),
        },
      };
    case "missing-sass-symbol":
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Warning,
        source: "css-module-explainer",
        message: formatCheckerFinding(finding, ""),
        data: {
          createSassSymbol: buildCreateSassSymbolActionData(
            finding.symbolKind,
            finding.symbolName,
            finding.selectorFilePath,
            styleDocument,
            finding.symbolSyntax,
          ),
        },
      };
    default:
      finding satisfies never;
      return finding;
  }
}
