import path from "node:path";
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
import {
  SELECTED_QUERY_RUNNER_COMMANDS,
  resolveSelectedQueryBackendKind,
  type RustSelectedQueryBackendJsonRunnerAsync,
} from "../../../engine-host-node/src/selected-query-backend";
import type { QueryExternalSifInputV0 } from "../../../engine-host-node/src/external-sif-loader";
import type {
  OmenaQueryStyleDiagnosticV0Json,
  OmenaQueryStyleDiagnosticsForFileV0Json,
} from "../../../engine-host-node/src/query-diagnostics-idl.generated";
import type { ProviderDeps } from "./provider-deps";
import { toLspRange } from "./lsp-adapters";

type RuntimeStyleDiagnosticsDeps = Partial<
  Pick<
    ProviderDeps,
    | "analysisCache"
    | "readStyleFile"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "aliasResolver"
  >
> & {
  readonly env?: NodeJS.ProcessEnv;
  readonly styleSource?: string;
  readonly styleSemanticGraphCache?: StyleDiagnosticsQueryOptions["styleSemanticGraphCache"];
  readonly styleSemanticGraphBatchOutputCache?: StyleDiagnosticsQueryOptions["styleSemanticGraphBatchOutputCache"];
  readonly selectorUsagePayloadCache?: StyleDiagnosticsQueryOptions["selectorUsagePayloadCache"];
  readonly sourceDocuments?: readonly QuerySourceDocumentInputV0[];
  readonly externalMode?: "ignored" | "sif";
  readonly externalSifs?: readonly QueryExternalSifInputV0[];
  readonly runRustSelectedQueryBackendJsonAsync?: RustSelectedQueryBackendJsonRunnerAsync;
};

interface QuerySourceDocumentInputV0 {
  readonly sourcePath: string;
  readonly sourceSource: string;
}

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
  runtimeDeps?: RuntimeStyleDiagnosticsDeps,
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
    const queryDiagnostics = resolveQueryOwnedStyleDiagnostics(
      { scssPath, styleDocument },
      runtimeDeps,
    );
    if (queryDiagnostics) {
      return queryDiagnostics;
    }
    if (resolveSelectedQueryBackendKind(runtimeDeps.env) === "rust-selected-query") {
      return [];
    }
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

function resolveQueryOwnedStyleDiagnostics(
  args: {
    readonly scssPath: string;
    readonly styleDocument: StyleDocumentHIR;
  },
  runtimeDeps: RuntimeStyleDiagnosticsDeps,
): Promise<Diagnostic[]> | null {
  if (resolveSelectedQueryBackendKind(runtimeDeps.env) !== "rust-selected-query") return null;
  const runJson = runtimeDeps.runRustSelectedQueryBackendJsonAsync;
  if (!runJson) return null;
  const styleSource = runtimeDeps.styleSource ?? runtimeDeps.readStyleFile?.(args.scssPath);
  if (!styleSource) return null;

  return runJson<OmenaQueryStyleDiagnosticsForFileV0Json>(
    SELECTED_QUERY_RUNNER_COMMANDS.styleDiagnosticsForFile,
    {
      targetStylePath: args.scssPath,
      styles: [
        {
          stylePath: args.scssPath,
          styleSource,
        },
      ],
      sourceDocuments: runtimeDeps.sourceDocuments ?? [],
      packageManifests: [],
      classnameTransform: runtimeDeps.settings?.scss.classnameTransform,
      // Omit external* entirely when no SIFs are supplied so the engine wire
      // defaults to today's `Ignored` + empty-SIF behaviour (#32).
      ...(runtimeDeps.externalSifs && runtimeDeps.externalSifs.length > 0
        ? {
            externalMode: runtimeDeps.externalMode ?? "sif",
            externalSifs: runtimeDeps.externalSifs,
          }
        : {}),
    },
  ).then((summary) => {
    if (summary.product !== "omena-query.diagnostics-for-file" || summary.fileKind !== "style") {
      return [];
    }
    return summary.diagnostics.map((diagnostic) =>
      toQueryOwnedStyleDiagnostic(
        diagnostic,
        args.styleDocument,
        runtimeDeps.styleDocumentForPath,
        runtimeDeps.readStyleFile,
      ),
    );
  });
}

function toQueryOwnedStyleDiagnostic(
  diagnostic: OmenaQueryStyleDiagnosticV0Json,
  styleDocument: StyleDocumentHIR,
  styleDocumentForPath?: (filePath: string) => StyleDocumentHIR | null,
  readStyleFile?: (filePath: string) => string | null,
): Diagnostic {
  const data = {
    querySeverity: diagnostic.severity,
    provenance: diagnostic.provenance,
    ...(diagnostic.createCustomProperty
      ? { createCustomProperty: diagnostic.createCustomProperty }
      : queryQuickFixData(diagnostic, styleDocument, styleDocumentForPath, readStyleFile)),
    ...(diagnostic.cascadeConfidence ? { cascadeConfidence: diagnostic.cascadeConfidence } : {}),
    ...(diagnostic.polynomialProvenance
      ? { polynomialProvenance: diagnostic.polynomialProvenance }
      : {}),
    ...(diagnostic.crossFileScc ? { crossFileScc: diagnostic.crossFileScc } : {}),
  };
  return {
    range: toLspRange(diagnostic.range),
    severity: querySeverityToLspSeverity(diagnostic.severity),
    code: diagnostic.code,
    source: "omena-css",
    message: diagnostic.message,
    ...(diagnostic.tags?.length
      ? { tags: diagnostic.tags.map((tag) => tag as DiagnosticTag) }
      : {}),
    data,
  };
}

function queryQuickFixData(
  diagnostic: OmenaQueryStyleDiagnosticV0Json,
  styleDocument: StyleDocumentHIR,
  styleDocumentForPath?: (filePath: string) => StyleDocumentHIR | null,
  readStyleFile?: (filePath: string) => string | null,
):
  | {
      readonly createModuleFile?: { readonly uri: string };
      readonly createSelector?: ReturnType<typeof buildCreateSelectorActionData>;
      readonly createValue?: ReturnType<typeof buildCreateValueActionData>;
      readonly createKeyframes?: ReturnType<typeof buildCreateKeyframesActionData>;
      readonly createSassSymbol?: ReturnType<typeof buildCreateSassSymbolActionData>;
    }
  | Record<string, never> {
  if (diagnostic.code === "missingComposedModule") {
    const specifier = extractComposedModuleSpecifier(diagnostic.message);
    const targetPath = specifier
      ? resolveRelativeStyleSpecifier(styleDocument.filePath, specifier)
      : null;
    if (targetPath) {
      return { createModuleFile: { uri: pathToFileUrl(targetPath) } };
    }
  }
  if (diagnostic.code === "missingValueModule") {
    const specifier = extractValueModuleSpecifier(diagnostic.message);
    const targetPath = specifier
      ? resolveRelativeStyleSpecifier(styleDocument.filePath, specifier)
      : null;
    if (targetPath) {
      return { createModuleFile: { uri: pathToFileUrl(targetPath) } };
    }
  }
  if (diagnostic.code === "missingComposedSelector") {
    const target = extractComposedSelectorTarget(diagnostic.message);
    if (target) {
      const targetPath = target.specifier
        ? resolveRelativeStyleSpecifier(styleDocument.filePath, target.specifier)
        : styleDocument.filePath;
      const targetDocument = targetPath ? styleDocumentForPath?.(targetPath) : null;
      if (targetPath && targetDocument) {
        return {
          createSelector: buildCreateSelectorActionData(
            target.className,
            targetPath,
            targetDocument,
            readStyleFile?.(targetPath) ?? undefined,
          ),
        };
      }
    }
  }
  if (diagnostic.code === "missingImportedValue") {
    const target = extractImportedValueTarget(diagnostic.message);
    const targetPath = target
      ? resolveRelativeStyleSpecifier(styleDocument.filePath, target.specifier)
      : null;
    const targetDocument = targetPath ? styleDocumentForPath?.(targetPath) : null;
    if (target && targetPath && targetDocument) {
      return {
        createValue: buildCreateValueActionData(
          target.valueName,
          targetPath,
          targetDocument,
          readStyleFile?.(targetPath) ?? undefined,
        ),
      };
    }
  }
  if (diagnostic.code === "missingKeyframes") {
    const keyframesName = extractQuotedName(diagnostic.message);
    if (keyframesName) {
      return {
        createKeyframes: buildCreateKeyframesActionData(
          keyframesName,
          styleDocument.filePath,
          styleDocument,
        ),
      };
    }
  }
  if (diagnostic.code === "missingSassSymbol") {
    const symbol = extractSassSymbol(diagnostic.message);
    if (symbol) {
      return {
        createSassSymbol: buildCreateSassSymbolActionData(
          symbol.kind,
          symbol.name,
          styleDocument.filePath,
          styleDocument,
          symbol.syntax,
        ),
      };
    }
  }
  return {};
}

function resolveRelativeStyleSpecifier(styleFilePath: string, specifier: string): string | null {
  if (!specifier.startsWith(".")) return null;
  return path.resolve(path.dirname(styleFilePath), specifier);
}

function extractComposedModuleSpecifier(message: string): string | null {
  return /Cannot resolve composed CSS Module '([^']+)'/u.exec(message)?.[1] ?? null;
}

function extractValueModuleSpecifier(message: string): string | null {
  return /Cannot resolve imported @value module '([^']+)'/u.exec(message)?.[1] ?? null;
}

function extractComposedSelectorTarget(message: string): {
  readonly className: string;
  readonly specifier?: string;
} | null {
  const external = /Selector '\.([^']+)' not found in composed module '([^']+)'/u.exec(message);
  if (external?.[1] && external[2]) {
    return { className: external[1], specifier: external[2] };
  }
  const local = /Selector '\.([^']+)' not found in this file for composes/u.exec(message);
  if (local?.[1]) {
    return { className: local[1] };
  }
  return null;
}

function extractImportedValueTarget(message: string): {
  readonly valueName: string;
  readonly specifier: string;
} | null {
  const result = /@value '([^']+)' not found in '([^']+)'/u.exec(message);
  if (!result?.[1] || !result[2]) return null;
  return { valueName: result[1], specifier: result[2] };
}

function querySeverityToLspSeverity(
  severity: OmenaQueryStyleDiagnosticV0Json["severity"],
): DiagnosticSeverity {
  switch (severity) {
    case "error":
      return DiagnosticSeverity.Error;
    case "information":
      return DiagnosticSeverity.Information;
    case "hint":
      return DiagnosticSeverity.Hint;
    case "warning":
      return DiagnosticSeverity.Warning;
  }
}

function extractQuotedName(message: string): string | null {
  return /'([^']+)'/u.exec(message)?.[1] ?? null;
}

function extractSassSymbol(message: string): {
  readonly kind: Parameters<typeof buildCreateSassSymbolActionData>[0];
  readonly name: string;
  readonly syntax: Parameters<typeof buildCreateSassSymbolActionData>[4];
} | null {
  const variable = /Sass variable '\$([^']+)'/u.exec(message);
  if (variable?.[1]) {
    return { kind: "variable", name: variable[1], syntax: "sass" };
  }
  const mixin = /Sass mixin '@mixin ([^'()\s]+)'/u.exec(message);
  if (mixin?.[1]) {
    return { kind: "mixin", name: mixin[1], syntax: "sass" };
  }
  const fn = /Sass function '@function ([^'()\s]+)'/u.exec(message);
  if (fn?.[1]) {
    return { kind: "function", name: fn[1], syntax: "sass" };
  }
  return null;
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
        code: finding.code,
        source: "omena-css",
        message: formatCheckerFinding(finding, ""),
        tags: [DiagnosticTag.Unnecessary],
      };
    case "missing-composed-module":
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Warning,
        code: finding.code,
        source: "omena-css",
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
        code: finding.code,
        source: "omena-css",
        message: formatCheckerFinding(finding, ""),
        data,
      };
    }
    case "missing-value-module":
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Warning,
        code: finding.code,
        source: "omena-css",
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
        code: finding.code,
        source: "omena-css",
        message: formatCheckerFinding(finding, ""),
        data,
      };
    case "missing-keyframes":
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Warning,
        code: finding.code,
        source: "omena-css",
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
        code: finding.code,
        source: "omena-css",
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
        code: finding.code,
        source: "omena-css",
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
