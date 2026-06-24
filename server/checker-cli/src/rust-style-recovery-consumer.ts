import { spawn } from "node:child_process";
import path from "node:path";
import type { CheckerReportV1 } from "../../engine-core-ts/src/contracts";
import { buildCheckerEngineParitySnapshotV2 } from "../../engine-host-node/src/engine-parity-v2";
import type { WorkspaceCheckOptions } from "../../engine-host-node/src/checker-host";
import {
  collectSourceDocuments,
  createWorkspaceAnalysisHost,
  createWorkspaceStyleHost,
  resolveWorkspaceCheckFiles,
} from "../../engine-host-node/src/checker-host/workspace-check-support";

const REPO_ROOT = path.resolve(__dirname, "../../..");
const RUST_MANIFEST = path.join(REPO_ROOT, "rust/Cargo.toml");

export interface CheckerStyleRecoveryCanonicalProducerSignalV0 {
  readonly schemaVersion: "0";
  readonly inputVersion: string;
  readonly canonicalCandidate: {
    readonly schemaVersion: "0";
    readonly inputVersion: string;
    readonly reportVersion: string;
    readonly bundle: "style-recovery";
    readonly distinctFileCount: number;
    readonly codeCounts: Readonly<Record<string, number>>;
    readonly summary: {
      readonly warnings: number;
      readonly hints: number;
      readonly total: number;
    };
    readonly findings: readonly {
      readonly filePath: string;
      readonly code: string;
      readonly severity: string;
      readonly range: {
        readonly start: {
          readonly line: number;
          readonly character: number;
        };
        readonly end: {
          readonly line: number;
          readonly character: number;
        };
      };
      readonly message: string;
      readonly analysisReason?: string;
      readonly valueCertaintyShapeLabel?: string;
    }[];
  };
  readonly boundedCheckerGate: {
    readonly canonicalCandidateCommand: "pnpm check:rust-checker-style-recovery-canonical-candidate";
    readonly canonicalProducerCommand: "pnpm check:rust-checker-style-recovery-canonical-producer";
    readonly consumerBoundaryCommand: "pnpm check:rust-checker-style-recovery-consumer-boundary";
    readonly boundedCheckerLaneCommand: "pnpm check:rust-checker-bounded-lanes";
    readonly promotionReviewCommand: "pnpm check:rust-checker-promotion-review";
    readonly promotionEvidenceCommand: "pnpm check:rust-checker-promotion-evidence";
    readonly broaderRustLaneCommand: "pnpm check:rust-lane-bundle";
    readonly releaseGateReadinessCommand: "pnpm check:rust-checker-release-gate-readiness";
    readonly releaseGateShadowCommand: "pnpm check:rust-checker-release-gate-shadow";
    readonly releaseGateShadowReviewCommand: "pnpm check:rust-checker-release-gate-shadow-review";
    readonly releaseBundleCommand: "pnpm check:rust-release-bundle";
    readonly minimumBoundedLaneCountForRustLaneBundle: 3;
    readonly minimumBoundedLaneCountForRustReleaseBundle: 3;
    readonly minimumSuccessfulShadowRunsForRustReleaseBundle: 3;
    readonly checkerBundle: "style-recovery";
    readonly releaseGateStage: "enforced";
    readonly includedInRustLaneBundle: true;
    readonly includedInRustReleaseBundle: true;
  };
}

export async function buildRustStyleRecoveryCanonicalProducer(
  workspace: WorkspaceCheckOptions,
  checkerReport: CheckerReportV1,
): Promise<CheckerStyleRecoveryCanonicalProducerSignalV0> {
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

  const snapshot = await buildCheckerEngineParitySnapshotV2({
    workspaceRoot,
    classnameTransform,
    pathAlias,
    sourceDocuments,
    styleFiles,
    analysisCache: analysisHost.analysisCache,
    styleDocumentForPath: styleHost.styleDocumentForPath,
    typeResolver: analysisHost.typeResolver,
    semanticReferenceIndex: analysisHost.semanticReferenceIndex,
    styleDependencyGraph: styleHost.styleDependencyGraph,
    checkerReport,
    ...(workspace.typeBackend ? { typeBackend: workspace.typeBackend } : {}),
    env: workspace.env ?? process.env,
  });

  return runRustStyleRecoveryCanonicalProducer(snapshot);
}

function runRustStyleRecoveryCanonicalProducer(
  snapshot: unknown,
): Promise<CheckerStyleRecoveryCanonicalProducerSignalV0> {
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
        "output-checker-style-recovery-canonical-producer",
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
        resolve(JSON.parse(stdout.join("")) as CheckerStyleRecoveryCanonicalProducerSignalV0);
      } catch (error) {
        reject(error);
      }
    });

    child.stdin.end(JSON.stringify(snapshot));
  });
}
