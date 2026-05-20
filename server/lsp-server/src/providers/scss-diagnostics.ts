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
import type { ProviderDeps } from "./provider-deps";
import { toLspRange } from "./lsp-adapters";

type RuntimeStyleDiagnosticsDeps = Partial<
  Pick<
    ProviderDeps,
    | "analysisCache"
    | "readStyleFile"
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
  readonly runRustSelectedQueryBackendJsonAsync?: RustSelectedQueryBackendJsonRunnerAsync;
};

interface QueryStyleDiagnosticsForFileV0 {
  readonly product: "omena-query.diagnostics-for-file";
  readonly fileKind: "style";
  readonly diagnostics: readonly QueryStyleDiagnosticV0[];
}

interface QueryStyleDiagnosticV0 {
  readonly code: string;
  readonly severity: "error" | "warning" | "information" | "hint";
  readonly provenance: readonly string[];
  readonly range: StyleCheckerFinding["range"];
  readonly message: string;
  readonly tags?: readonly number[];
  readonly createCustomProperty?: {
    readonly uri: string;
    readonly range: StyleCheckerFinding["range"];
    readonly newText: string;
    readonly propertyName: string;
  };
}

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
    const checkerDiagnostics = resolveStyleDiagnosticFindingsAsync(
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
    const queryDiagnostics = resolveQueryOwnedStyleDiagnostics(
      { scssPath, styleDocument },
      runtimeDeps,
    );
    if (queryDiagnostics) {
      return Promise.all([queryDiagnostics, checkerDiagnostics]).then(([query, checker]) =>
        mergeQueryOwnedStyleDiagnostics(query, checker),
      );
    }
    return checkerDiagnostics;
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

  return runJson<QueryStyleDiagnosticsForFileV0>(
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
    },
  ).then((summary) => {
    if (summary.product !== "omena-query.diagnostics-for-file" || summary.fileKind !== "style") {
      return [];
    }
    return summary.diagnostics.map((diagnostic) =>
      toQueryOwnedStyleDiagnostic(diagnostic, args.styleDocument),
    );
  });
}

function toQueryOwnedStyleDiagnostic(
  diagnostic: QueryStyleDiagnosticV0,
  styleDocument: StyleDocumentHIR,
): Diagnostic {
  const data = {
    querySeverity: diagnostic.severity,
    provenance: diagnostic.provenance,
    ...(diagnostic.createCustomProperty
      ? { createCustomProperty: diagnostic.createCustomProperty }
      : queryQuickFixData(diagnostic, styleDocument)),
  };
  return {
    range: toLspRange(diagnostic.range),
    severity: querySeverityToLspSeverity(diagnostic.severity),
    code: diagnostic.code,
    source: "css-module-explainer",
    message: diagnostic.message,
    ...(diagnostic.tags?.length
      ? { tags: diagnostic.tags.map((tag) => tag as DiagnosticTag) }
      : {}),
    data,
  };
}

function queryQuickFixData(
  diagnostic: QueryStyleDiagnosticV0,
  styleDocument: StyleDocumentHIR,
):
  | {
      readonly createKeyframes?: ReturnType<typeof buildCreateKeyframesActionData>;
      readonly createSassSymbol?: ReturnType<typeof buildCreateSassSymbolActionData>;
    }
  | Record<string, never> {
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

function querySeverityToLspSeverity(
  severity: QueryStyleDiagnosticV0["severity"],
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

function mergeQueryOwnedStyleDiagnostics(
  queryDiagnostics: readonly Diagnostic[],
  checkerDiagnostics: readonly Diagnostic[],
): Diagnostic[] {
  if (queryDiagnostics.length === 0) return [...checkerDiagnostics];
  const checkerKeys = new Set(
    checkerDiagnostics.map((diagnostic) => diagnosticKey(diagnostic.code, diagnostic.range)),
  );
  const effectiveQueryDiagnostics = queryDiagnostics.filter((diagnostic) =>
    shouldKeepQueryOwnedStyleDiagnostic(diagnostic, checkerKeys),
  );
  if (effectiveQueryDiagnostics.length === 0) return [...checkerDiagnostics];
  const queryDuplicateKeys = new Set<string>();
  for (const diagnostic of effectiveQueryDiagnostics) {
    const checkerCode = checkerDuplicateCodeForQueryCode(diagnostic.code);
    if (checkerCode) {
      queryDuplicateKeys.add(diagnosticKey(checkerCode, diagnostic.range));
    }
  }
  return [
    ...effectiveQueryDiagnostics,
    ...checkerDiagnostics.filter(
      (diagnostic) => !queryDuplicateKeys.has(diagnosticKey(diagnostic.code, diagnostic.range)),
    ),
  ];
}

function shouldKeepQueryOwnedStyleDiagnostic(
  diagnostic: Diagnostic,
  checkerKeys: ReadonlySet<string>,
): boolean {
  if (diagnostic.code !== "missingSassSymbol") return true;
  // omena-query currently reports Sass symbol diagnostics from same-file parser
  // facts; keep cross-file Sass module resolution authoritative in the checker.
  return checkerKeys.has(diagnosticKey("missing-sass-symbol", diagnostic.range));
}

function checkerDuplicateCodeForQueryCode(code: Diagnostic["code"]): string | null {
  if (typeof code !== "string") return null;
  return QUERY_TO_CHECKER_DIAGNOSTIC_CODE[code] ?? null;
}

const QUERY_TO_CHECKER_DIAGNOSTIC_CODE: Readonly<Record<string, string>> = {
  unusedSelector: "unused-selector",
  missingComposedModule: "missing-composed-module",
  missingComposedSelector: "missing-composed-selector",
  missingValueModule: "missing-value-module",
  missingImportedValue: "missing-imported-value",
  missingCustomProperty: "missing-custom-property",
  missingKeyframes: "missing-keyframes",
  missingSassSymbol: "missing-sass-symbol",
};

function diagnosticKey(code: Diagnostic["code"], range: Diagnostic["range"]): string {
  return [
    String(code ?? ""),
    range.start.line,
    range.start.character,
    range.end.line,
    range.end.character,
  ].join(":");
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
        source: "css-module-explainer",
        message: formatCheckerFinding(finding, ""),
        tags: [DiagnosticTag.Unnecessary],
      };
    case "missing-composed-module":
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Warning,
        code: finding.code,
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
        code: finding.code,
        source: "css-module-explainer",
        message: formatCheckerFinding(finding, ""),
        data,
      };
    }
    case "missing-value-module":
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Warning,
        code: finding.code,
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
        code: finding.code,
        source: "css-module-explainer",
        message: formatCheckerFinding(finding, ""),
        data,
      };
    case "missing-keyframes":
      return {
        range: toLspRange(finding.range),
        severity: DiagnosticSeverity.Warning,
        code: finding.code,
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
        code: finding.code,
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
        code: finding.code,
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
