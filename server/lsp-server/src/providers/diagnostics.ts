import {
  DiagnosticSeverity,
  type Diagnostic,
  type Range as LspRange,
} from "vscode-languageserver/node";
import { type SourceCheckerFinding } from "../../../engine-core-ts/src/core/checker";
import { formatCheckerFinding } from "../../../engine-core-ts/src/checker-surface";
import { pathToFileUrl } from "../../../engine-core-ts/src/core/util/text-utils";
import { buildCreateSelectorActionData } from "../../../engine-host-node/src/code-action-data";
import {
  resolveSourceDiagnosticFindings,
  resolveSourceDiagnosticFindingsAsync,
} from "../../../engine-host-node/src/source-diagnostics-query";
import {
  SELECTED_QUERY_RUNNER_COMMANDS,
  resolveSelectedQueryBackendKind,
  type RustSelectedQueryBackendJsonRunnerAsync,
} from "../../../engine-host-node/src/selected-query-backend";
import { toLspRange } from "./lsp-adapters";
import { wrapHandler } from "./_wrap-handler";
import { getRustSelectedQueryBackendJsonRunnerAsync } from "./selected-query-runner";
import type { DocumentParams, ProviderDeps } from "./provider-deps";

interface QuerySourceDiagnosticsForFileV0 {
  readonly product: "omena-query.diagnostics-for-file";
  readonly fileKind: "source";
  readonly diagnostics: readonly QuerySourceDiagnosticV0[];
}

interface QuerySourceDiagnosticV0 {
  readonly code: string;
  readonly severity: "error" | "warning" | "information" | "hint";
  readonly provenance: readonly string[];
  readonly range: SourceCheckerFinding["range"];
  readonly message: string;
  readonly createSelector?: {
    readonly uri: string;
    readonly range: SourceCheckerFinding["range"];
    readonly newText: string;
    readonly selectorName: string;
  };
}

/**
 * Compute diagnostics for an open document.
 *
 * Push-based: the composition root calls this on
 * `onDidChangeContent` (debounced) and pipes the result into
 * `connection.sendDiagnostics(...)`.
 *
 * Iterates every cached class expression whose origin is `cxCall` in the
 * document's analysis entry, classifies each, and emits a
 * Diagnostic for unresolved / missing class names. Returns [] for
 * clean documents — caller MUST still publish to clear prior
 * warnings.
 *
 * Error isolation is owned by `wrapHandler` at the entry level;
 * per-ref validation failures are caught inside so a single bad
 * ref cannot erase sibling diagnostics.
 */
export const computeDiagnostics = wrapHandler<
  DocumentParams,
  [severity?: DiagnosticSeverity],
  Diagnostic[]
>(
  "diagnostics",
  (params, deps, severity: DiagnosticSeverity = DiagnosticSeverity.Warning) => {
    const rustRunner = getRustSelectedQueryBackendJsonRunnerAsync(deps);
    if (rustRunner) {
      const checkerDiagnostics = resolveSourceDiagnosticFindingsAsync(params, deps, {
        runRustSelectedQueryBackendJsonAsync: rustRunner,
      }).then((findings) => findings.map((finding) => toDiagnostic(finding, deps, severity)));
      const queryDiagnostics = resolveQueryOwnedSourceDiagnostics(params, deps, rustRunner);
      if (queryDiagnostics) {
        return Promise.all([queryDiagnostics, checkerDiagnostics]).then(([query, checker]) =>
          mergeQueryOwnedSourceDiagnostics(query, checker),
        );
      }
      return checkerDiagnostics;
    }
    return resolveSourceDiagnosticFindings(params, deps).map((finding) =>
      toDiagnostic(finding, deps, severity),
    );
  },
  [],
);

const DIAGNOSTIC_SOURCE = "css-module-explainer";

function resolveQueryOwnedSourceDiagnostics(
  params: DocumentParams,
  deps: ProviderDeps,
  runJson: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<Diagnostic[]> | null {
  if (resolveSelectedQueryBackendKind() !== "rust-selected-query") return null;
  const styles = collectSourceDiagnosticStyleSources(params, deps);
  if (styles.length === 0) return null;
  const diagnosticScopeRanges = collectQueryOwnedSourceDiagnosticScopeRanges(params, deps);
  if (diagnosticScopeRanges.length === 0) return null;

  return runJson<QuerySourceDiagnosticsForFileV0>(
    SELECTED_QUERY_RUNNER_COMMANDS.sourceDiagnosticsForFile,
    {
      sourcePath: params.filePath,
      sourceSource: params.content,
      styles,
      packageManifests: [],
    },
  ).then((summary) => {
    if (summary.product !== "omena-query.diagnostics-for-file" || summary.fileKind !== "source") {
      return [];
    }
    return summary.diagnostics
      .filter((diagnostic) => diagnostic.code !== "missingModule")
      .filter((diagnostic) => sourceRangeMatchesAny(diagnostic.range, diagnosticScopeRanges))
      .map(toQueryOwnedSourceDiagnostic);
  });
}

function collectSourceDiagnosticStyleSources(
  params: DocumentParams,
  deps: ProviderDeps,
): readonly { readonly stylePath: string; readonly styleSource: string }[] {
  const entry = deps.analysisCache.get(
    params.documentUri,
    params.content,
    params.filePath,
    params.version,
  );
  const stylePaths = new Set<string>();
  for (const styleImport of entry.stylesBindings.values()) {
    if (styleImport.kind === "resolved") {
      stylePaths.add(styleImport.absolutePath);
    }
  }
  for (const expression of entry.sourceDocument.classExpressions) {
    stylePaths.add(expression.scssModulePath);
  }

  const styles: { stylePath: string; styleSource: string }[] = [];
  for (const stylePath of stylePaths) {
    const styleSource = deps.readOpenDocumentText?.(stylePath) ?? deps.readStyleFile(stylePath);
    if (styleSource !== null) {
      styles.push({ stylePath, styleSource });
    }
  }
  return styles;
}

function collectQueryOwnedSourceDiagnosticScopeRanges(
  params: DocumentParams,
  deps: ProviderDeps,
): readonly SourceCheckerFinding["range"][] {
  const entry = deps.analysisCache.get(
    params.documentUri,
    params.content,
    params.filePath,
    params.version,
  );
  return entry.sourceDocument.classExpressions
    .filter((expression) => expression.origin === "cxCall")
    .map((expression) => expression.range);
}

function sourceRangeMatchesAny(
  range: SourceCheckerFinding["range"],
  candidates: readonly SourceCheckerFinding["range"][],
): boolean {
  return candidates.some((candidate) => sourceRangeContains(candidate, range));
}

function sourceRangeContains(
  outer: SourceCheckerFinding["range"],
  inner: SourceCheckerFinding["range"],
): boolean {
  return (
    comparePositions(outer.start, inner.start) <= 0 && comparePositions(inner.end, outer.end) <= 0
  );
}

function comparePositions(
  left: SourceCheckerFinding["range"]["start"],
  right: SourceCheckerFinding["range"]["start"],
): number {
  if (left.line !== right.line) return left.line - right.line;
  return left.character - right.character;
}

function toQueryOwnedSourceDiagnostic(diagnostic: QuerySourceDiagnosticV0): Diagnostic {
  const createSelector = diagnostic.createSelector
    ? {
        ...diagnostic.createSelector,
        uri: diagnostic.createSelector.uri.startsWith("file://")
          ? diagnostic.createSelector.uri
          : pathToFileUrl(diagnostic.createSelector.uri),
      }
    : undefined;
  return {
    range: toLspRange(diagnostic.range),
    severity: querySeverityToLspSeverity(diagnostic.severity),
    code: diagnostic.code,
    source: DIAGNOSTIC_SOURCE,
    message: diagnostic.message,
    data: {
      querySeverity: diagnostic.severity,
      provenance: diagnostic.provenance,
      ...(createSelector ? { createSelector } : {}),
    },
  };
}

function querySeverityToLspSeverity(
  severity: QuerySourceDiagnosticV0["severity"],
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

function mergeQueryOwnedSourceDiagnostics(
  queryDiagnostics: readonly Diagnostic[],
  checkerDiagnostics: readonly Diagnostic[],
): Diagnostic[] {
  if (queryDiagnostics.length === 0) return [...checkerDiagnostics];
  const checkerByQueryDuplicateKey = new Map<string, Diagnostic>();
  for (const checkerDiagnostic of checkerDiagnostics) {
    checkerByQueryDuplicateKey.set(
      diagnosticKey(checkerDiagnostic.code, checkerDiagnostic.range),
      checkerDiagnostic,
    );
  }
  const consumedCheckerKeys = new Set<string>();
  const mergedQueryDiagnostics = queryDiagnostics.flatMap((queryDiagnostic) => {
    const checkerCode = checkerDuplicateCodeForQueryCode(queryDiagnostic.code);
    if (!checkerCode) return [queryDiagnostic];
    const checkerKey = diagnosticKey(checkerCode, queryDiagnostic.range);
    const checkerDiagnostic = checkerByQueryDuplicateKey.get(checkerKey);
    if (!checkerDiagnostic) return [queryDiagnostic];
    consumedCheckerKeys.add(checkerKey);
    return [mergeQueryDiagnosticWithCheckerData(queryDiagnostic, checkerDiagnostic)];
  });
  return [
    ...mergedQueryDiagnostics,
    ...checkerDiagnostics.filter(
      (diagnostic) => !consumedCheckerKeys.has(diagnosticKey(diagnostic.code, diagnostic.range)),
    ),
  ];
}

function mergeQueryDiagnosticWithCheckerData(
  queryDiagnostic: Diagnostic,
  checkerDiagnostic: Diagnostic,
): Diagnostic {
  const checkerData = isRecord(checkerDiagnostic.data) ? checkerDiagnostic.data : {};
  const queryData = isRecord(queryDiagnostic.data) ? queryDiagnostic.data : {};
  return {
    ...queryDiagnostic,
    message: checkerDiagnostic.message,
    data: {
      ...checkerData,
      ...queryData,
    },
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function checkerDuplicateCodeForQueryCode(code: Diagnostic["code"]): string | null {
  if (typeof code !== "string") return null;
  return QUERY_TO_CHECKER_DIAGNOSTIC_CODE[code] ?? null;
}

const QUERY_TO_CHECKER_DIAGNOSTIC_CODE: Readonly<Record<string, string>> = {
  missingSelector: "missing-static-class",
  missingStaticClass: "missing-static-class",
  missingTemplatePrefix: "missing-template-prefix",
  missingResolvedClassValues: "missing-resolved-class-values",
  missingResolvedClassDomain: "missing-resolved-class-domain",
};

function diagnosticKey(code: Diagnostic["code"], range: LspRange): string {
  return [
    String(code ?? ""),
    range.start.line,
    range.start.character,
    range.end.line,
    range.end.character,
  ].join(":");
}

function toDiagnostic(
  finding: SourceCheckerFinding,
  deps: ProviderDeps,
  severity: DiagnosticSeverity,
): Diagnostic {
  const range: LspRange = toLspRange(finding.range);

  switch (finding.code) {
    case "missing-static-class": {
      const styleDocument = deps.styleDocumentForPath(finding.scssModulePath);
      return {
        range,
        severity,
        code: finding.code,
        source: DIAGNOSTIC_SOURCE,
        message: formatCheckerFinding(finding, deps.workspaceRoot),
        data: {
          ...(finding.suggestion ? { suggestion: finding.suggestion } : {}),
          ...(styleDocument
            ? {
                createSelector: buildCreateSelectorActionData(
                  finding.className,
                  finding.scssModulePath,
                  styleDocument,
                ),
              }
            : {}),
        },
      };
    }
    case "missing-template-prefix":
      return {
        range,
        severity,
        code: finding.code,
        source: DIAGNOSTIC_SOURCE,
        message: formatCheckerFinding(finding, deps.workspaceRoot),
      };
    case "missing-resolved-class-values":
    case "missing-resolved-class-domain":
      return {
        range,
        severity,
        code: finding.code,
        source: DIAGNOSTIC_SOURCE,
        message: formatCheckerFinding(finding, deps.workspaceRoot),
        ...(finding.valueDomainDerivation
          ? { data: { valueDomainDerivation: finding.valueDomainDerivation } }
          : {}),
      };
    case "missing-module":
      return {
        range,
        severity,
        source: DIAGNOSTIC_SOURCE,
        message: formatCheckerFinding(finding, deps.workspaceRoot),
        code: "missing-module",
        data: {
          createModuleFile: {
            uri: pathToFileUrl(finding.absolutePath),
          },
        },
      };
    default:
      finding satisfies never;
      return finding;
  }
}
