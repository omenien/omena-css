import type {
  CheckerFinding,
  WorkspaceCheckerFinding,
} from "../../../engine-core-ts/src/core/checker/contracts";
import type { CheckerReportSummaryV1 } from "../../../engine-core-ts/src/contracts";
import type { ClassnameTransformMode } from "../../../engine-core-ts/src/core/scss/classname-transform";
import { NOOP_LOG_ERROR } from "../../../engine-core-ts/src/provider-deps";
import { DEFAULT_SETTINGS } from "../../../engine-core-ts/src/settings";
import { buildEngineInputV2Async } from "../engine-input-v2";
import {
  resolveSelectedQueryBackendKind,
  usesRustStyleSemanticGraphBackend,
} from "../selected-query-backend";
import { resolveSourceDiagnosticFindings } from "../source-diagnostics-query";
import { resolveStyleDiagnosticFindings } from "../style-diagnostics-query";
import type { SelectorUsagePayloadCache } from "../selector-usage-query-backend";
import type { StyleSemanticGraphCache } from "../style-semantic-graph-query-backend";
import type { TypeFactBackendKind } from "../type-backend";
import {
  collectSourceDocuments,
  createWorkspaceAnalysisHost,
  createWorkspaceStyleHost,
  resolveWorkspaceCheckFiles,
} from "./workspace-check-support";

export interface WorkspaceCheckOptions {
  readonly workspaceRoot: string;
  readonly classnameTransform?: ClassnameTransformMode;
  readonly pathAlias?: Readonly<Record<string, string>>;
  readonly typeBackend?: TypeFactBackendKind;
  readonly env?: NodeJS.ProcessEnv;
  readonly includeMissingModule?: boolean;
  readonly includeUnusedSelectors?: boolean;
  readonly includeComposesResolution?: boolean;
  readonly sourceFilePaths?: readonly string[];
  readonly styleFilePaths?: readonly string[];
}

export type WorkspaceCheckSummary = CheckerReportSummaryV1;

export interface WorkspaceCheckResult {
  readonly sourceFiles: readonly string[];
  readonly styleFiles: readonly string[];
  readonly findings: readonly WorkspaceCheckerFinding[];
  readonly summary: WorkspaceCheckSummary;
}

export async function checkWorkspace(
  options: WorkspaceCheckOptions,
): Promise<WorkspaceCheckResult> {
  const workspaceRoot = options.workspaceRoot;
  const classnameTransform = options.classnameTransform ?? "asIs";
  const pathAlias = options.pathAlias ?? {};
  const settings = {
    ...DEFAULT_SETTINGS,
    diagnostics: {
      ...DEFAULT_SETTINGS.diagnostics,
      missingModule: options.includeMissingModule ?? DEFAULT_SETTINGS.diagnostics.missingModule,
      unusedSelector: options.includeUnusedSelectors ?? DEFAULT_SETTINGS.diagnostics.unusedSelector,
    },
    scss: { classnameTransform },
    pathAlias,
  };
  const { sourceFiles, styleFiles } = await resolveWorkspaceCheckFiles({
    workspaceRoot,
    ...(options.sourceFilePaths ? { sourceFilePaths: options.sourceFilePaths } : {}),
    ...(options.styleFilePaths ? { styleFilePaths: options.styleFilePaths } : {}),
  });
  const styleHost = createWorkspaceStyleHost({
    styleFiles,
    classnameTransform,
  });
  styleHost.preloadStyleDocuments();
  const analysisHost = createWorkspaceAnalysisHost({
    workspaceRoot,
    classnameTransform,
    pathAlias,
    styleDocumentForPath: styleHost.styleDocumentForPath,
    ...(options.typeBackend ? { typeBackend: options.typeBackend } : {}),
    env: options.env ?? process.env,
  });
  const sourceDocuments = collectSourceDocuments(sourceFiles, analysisHost.analysisCache);

  const findings: WorkspaceCheckerFinding[] = [];
  const sourceDiagnosticOptions = options.env ? { env: options.env } : {};
  const styleSemanticGraphCache: StyleSemanticGraphCache = new Map();
  const selectorUsagePayloadCache: SelectorUsagePayloadCache = new Map();
  const selectedQueryBackend = resolveSelectedQueryBackendKind(options.env ?? process.env);
  const styleSemanticGraphEngineInput = usesRustStyleSemanticGraphBackend(selectedQueryBackend)
    ? await buildEngineInputV2Async({
        workspaceRoot,
        classnameTransform,
        pathAlias,
        sourceDocuments,
        styleFiles,
        analysisCache: analysisHost.analysisCache,
        styleDocumentForPath: styleHost.styleDocumentForPath,
        typeResolver: analysisHost.typeResolver,
      })
    : undefined;
  const styleDiagnosticOptions = {
    ...(options.env ? { env: options.env } : {}),
    ...(styleSemanticGraphEngineInput ? { engineInput: styleSemanticGraphEngineInput } : {}),
    sourceDocuments,
    styleFiles,
    styleSemanticGraphCache,
    selectorUsagePayloadCache,
    ...(options.includeUnusedSelectors !== undefined
      ? { includeUnusedSelectors: options.includeUnusedSelectors }
      : {}),
    ...(options.includeComposesResolution !== undefined
      ? { includeComposesResolution: options.includeComposesResolution }
      : {}),
  };

  for (const document of sourceDocuments) {
    for (const finding of resolveSourceDiagnosticFindings(
      {
        documentUri: document.uri,
        content: document.content,
        filePath: document.filePath,
        version: document.version,
      },
      {
        analysisCache: analysisHost.analysisCache,
        styleDocumentForPath: styleHost.styleDocumentForPath,
        typeResolver: analysisHost.typeResolver,
        workspaceRoot,
        settings,
        logError: NOOP_LOG_ERROR,
      },
      sourceDiagnosticOptions,
    )) {
      findings.push({ filePath: document.filePath, finding });
    }
  }

  for (const styleFile of styleFiles) {
    const styleDocument = styleHost.styleDocumentForPath(styleFile);
    if (!styleDocument) continue;
    for (const finding of resolveStyleDiagnosticFindings(
      {
        scssPath: styleFile,
        styleDocument,
      },
      {
        analysisCache: analysisHost.analysisCache,
        semanticReferenceIndex: analysisHost.semanticReferenceIndex,
        styleDependencyGraph: styleHost.styleDependencyGraph,
        styleDocumentForPath: styleHost.styleDocumentForPath,
        readStyleFile: styleHost.readStyleFile,
        typeResolver: analysisHost.typeResolver,
        workspaceRoot,
        settings,
      },
      styleDiagnosticOptions,
    )) {
      findings.push({ filePath: styleFile, finding });
    }
  }

  const sortedFindings = findings.toSorted(compareWorkspaceFindings);
  const summary = summarizeFindings(sortedFindings.map((entry) => entry.finding));

  return {
    sourceFiles,
    styleFiles,
    findings: sortedFindings,
    summary,
  };
}
function compareWorkspaceFindings(a: WorkspaceCheckerFinding, b: WorkspaceCheckerFinding): number {
  return (
    a.filePath.localeCompare(b.filePath) ||
    a.finding.range.start.line - b.finding.range.start.line ||
    a.finding.range.start.character - b.finding.range.start.character ||
    a.finding.code.localeCompare(b.finding.code)
  );
}

function summarizeFindings(findings: readonly CheckerFinding[]): WorkspaceCheckSummary {
  let warnings = 0;
  let hints = 0;
  for (const finding of findings) {
    if (finding.severity === "warning") warnings += 1;
    if (finding.severity === "hint") hints += 1;
  }
  return {
    warnings,
    hints,
    total: findings.length,
  };
}
