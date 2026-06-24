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
import type {
  OmenaQuerySourceDiagnosticV0Json,
  OmenaQuerySourceDiagnosticsForFileV0Json,
} from "../../../engine-host-node/src/query-diagnostics-idl.generated";
import { toLspRange } from "./lsp-adapters";
import { wrapHandler } from "./_wrap-handler";
import { getRustSelectedQueryBackendJsonRunnerAsync } from "./selected-query-runner";
import type { DocumentParams, ProviderDeps } from "./provider-deps";

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
      const queryDiagnostics = resolveQueryOwnedSourceDiagnostics(params, deps, rustRunner);
      if (queryDiagnostics) {
        return queryDiagnostics;
      }
      if (resolveSelectedQueryBackendKind() === "rust-selected-query") {
        return [];
      }
      return resolveSourceDiagnosticFindingsAsync(params, deps, {
        runRustSelectedQueryBackendJsonAsync: rustRunner,
      }).then((findings) => findings.map((finding) => toDiagnostic(finding, deps, severity)));
    }
    return resolveSourceDiagnosticFindings(params, deps).map((finding) =>
      toDiagnostic(finding, deps, severity),
    );
  },
  [],
);

const DIAGNOSTIC_SOURCE = "omena-css";

function resolveQueryOwnedSourceDiagnostics(
  params: DocumentParams,
  deps: ProviderDeps,
  runJson: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<Diagnostic[]> | null {
  if (resolveSelectedQueryBackendKind() !== "rust-selected-query") return null;
  const styles = collectSourceDiagnosticStyleSources(params, deps);
  const diagnosticScopeRanges = collectQueryOwnedSourceDiagnosticScopeRanges(params, deps);
  if (diagnosticScopeRanges.length === 0) return null;

  return runJson<OmenaQuerySourceDiagnosticsForFileV0Json>(
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
      .filter((diagnostic) => sourceRangeMatchesAny(diagnostic.range, diagnosticScopeRanges))
      .map((diagnostic) => toQueryOwnedSourceDiagnostic(diagnostic, params, deps));
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
    .map((expression) => expression.range)
    .concat(
      Array.from(entry.stylesBindings.values()).flatMap((styleImport) =>
        styleImport.kind === "missing" ? [styleImport.range] : [],
      ),
    );
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

function toQueryOwnedSourceDiagnostic(
  diagnostic: OmenaQuerySourceDiagnosticV0Json,
  params: DocumentParams,
  deps: ProviderDeps,
): Diagnostic {
  const createSelector = diagnostic.createSelector
    ? {
        ...diagnostic.createSelector,
        uri: diagnostic.createSelector.uri.startsWith("file://")
          ? diagnostic.createSelector.uri
          : pathToFileUrl(diagnostic.createSelector.uri),
      }
    : undefined;
  const createModuleFile = findMissingModuleCreateFileData(diagnostic, params, deps);
  return {
    range: toLspRange(diagnostic.range),
    severity: querySeverityToLspSeverity(diagnostic.severity),
    code: diagnostic.code,
    source: DIAGNOSTIC_SOURCE,
    message: diagnostic.message,
    data: {
      querySeverity: diagnostic.severity,
      provenance: diagnostic.provenance,
      ...(diagnostic.precision ? { precision: diagnostic.precision } : {}),
      ...(diagnostic.suggestion ? { suggestion: diagnostic.suggestion } : {}),
      ...(createSelector ? { createSelector } : {}),
      ...(createModuleFile ? { createModuleFile } : {}),
    },
  };
}

function findMissingModuleCreateFileData(
  diagnostic: OmenaQuerySourceDiagnosticV0Json,
  params: DocumentParams,
  deps: ProviderDeps,
): { readonly uri: string } | undefined {
  if (diagnostic.code !== "missingModule") return undefined;
  const entry = deps.analysisCache.get(
    params.documentUri,
    params.content,
    params.filePath,
    params.version,
  );
  for (const styleImport of entry.stylesBindings.values()) {
    if (
      styleImport.kind === "missing" &&
      sourceRangeContains(styleImport.range, diagnostic.range) &&
      sourceRangeContains(diagnostic.range, styleImport.range)
    ) {
      return { uri: pathToFileUrl(styleImport.absolutePath) };
    }
  }
  return undefined;
}

function querySeverityToLspSeverity(
  severity: OmenaQuerySourceDiagnosticV0Json["severity"],
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
