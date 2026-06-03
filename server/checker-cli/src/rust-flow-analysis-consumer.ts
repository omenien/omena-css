import { spawn } from "node:child_process";
import path from "node:path";
import { buildEngineInputV2 } from "../../engine-host-node/src/engine-input-v2";
import type { WorkspaceCheckOptions } from "../../engine-host-node/src/checker-host";
import {
  collectSourceDocuments,
  createWorkspaceAnalysisHost,
  createWorkspaceStyleHost,
  resolveWorkspaceCheckFiles,
} from "../../engine-host-node/src/checker-host/workspace-check-support";

const REPO_ROOT = path.resolve(__dirname, "../../..");
const RUST_MANIFEST = path.join(REPO_ROOT, "rust/Cargo.toml");

export interface ExpressionDomainFlowAnalysisV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly inputVersion: string;
  readonly analyses: readonly ExpressionDomainFlowAnalysisEntryV0[];
}

export interface ExpressionDomainFlowAnalysisEntryV0 {
  readonly graphId: string;
  readonly filePath: string;
  readonly analysis: ClassValueFlowAnalysisV0;
}

export interface ClassValueFlowAnalysisV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly contextSensitivity: string;
  readonly contextKey?: string;
  readonly converged: boolean;
  readonly iterationCount: number;
  readonly nodes: readonly ClassValueFlowNodeResultV0[];
}

export interface ClassValueFlowNodeResultV0 {
  readonly id: string;
  readonly predecessorIds: readonly string[];
  readonly transferKind: string;
  readonly valueKind: string;
  readonly value: Readonly<Record<string, unknown>>;
}

export interface RustFlowAnalysisConsumerV0 {
  readonly schemaVersion: "0";
  readonly product: "omena/checker.rust-flow-analysis-consumer";
  readonly inputVersion: string;
  readonly graphCount: number;
  readonly nodeCount: number;
  readonly convergedGraphCount: number;
  readonly unconvergedGraphCount: number;
  readonly maxIterationCount: number;
  readonly flowAnalysis: ExpressionDomainFlowAnalysisV0;
}

export async function buildRustFlowAnalysisConsumer(
  workspace: WorkspaceCheckOptions,
): Promise<RustFlowAnalysisConsumerV0> {
  const workspaceRoot = workspace.workspaceRoot;
  const classnameTransform = workspace.classnameTransform ?? "asIs";
  const pathAlias = workspace.pathAlias ?? {};
  const { sourceFiles, styleFiles } = await resolveWorkspaceCheckFiles({
    workspaceRoot,
    ...(workspace.sourceFilePaths ? { sourceFilePaths: workspace.sourceFilePaths } : {}),
    ...(workspace.styleFilePaths ? { styleFilePaths: workspace.styleFilePaths } : {}),
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
    ...(workspace.typeBackend ? { typeBackend: workspace.typeBackend } : {}),
    env: workspace.env ?? process.env,
  });
  const sourceDocuments = collectSourceDocuments(sourceFiles, analysisHost.analysisCache);
  const input = buildEngineInputV2({
    workspaceRoot,
    classnameTransform,
    pathAlias,
    sourceDocuments,
    styleFiles,
    analysisCache: analysisHost.analysisCache,
    styleDocumentForPath: styleHost.styleDocumentForPath,
    typeResolver: analysisHost.typeResolver,
    ...(workspace.typeBackend ? { typeBackend: workspace.typeBackend } : {}),
    env: workspace.env ?? process.env,
  });

  const flowAnalysis = await runRustExpressionDomainFlowAnalysis(input);
  const analyses = flowAnalysis.analyses;
  const nodeCount = analyses.reduce((sum, entry) => sum + entry.analysis.nodes.length, 0);
  const convergedGraphCount = analyses.filter((entry) => entry.analysis.converged).length;
  const maxIterationCount = analyses.reduce(
    (max, entry) => Math.max(max, entry.analysis.iterationCount),
    0,
  );

  return {
    schemaVersion: "0",
    product: "omena/checker.rust-flow-analysis-consumer",
    inputVersion: flowAnalysis.inputVersion,
    graphCount: analyses.length,
    nodeCount,
    convergedGraphCount,
    unconvergedGraphCount: analyses.length - convergedGraphCount,
    maxIterationCount,
    flowAnalysis,
  };
}

function runRustExpressionDomainFlowAnalysis(
  input: unknown,
): Promise<ExpressionDomainFlowAnalysisV0> {
  return new Promise((resolve, reject) => {
    const child = spawn(
      "cargo",
      [
        "run",
        "--manifest-path",
        RUST_MANIFEST,
        "-p",
        "engine-shadow-runner",
        "--quiet",
        "--",
        "input-expression-domain-flow-analysis",
      ],
      {
        cwd: REPO_ROOT,
        stdio: ["pipe", "pipe", "pipe"],
      },
    );

    const stdout: string[] = [];
    const stderr: string[] = [];
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.once("error", reject);
    child.once("close", (code) => {
      if (code !== 0) {
        reject(
          new Error(
            [`engine-shadow-runner exited with code ${code}`, stderr.join("").trim()]
              .filter(Boolean)
              .join("\n"),
          ),
        );
        return;
      }
      try {
        resolve(JSON.parse(stdout.join("")) as ExpressionDomainFlowAnalysisV0);
      } catch (error) {
        reject(error);
      }
    });

    child.stdin.end(JSON.stringify(input));
  });
}
