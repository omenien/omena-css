import { spawn } from "node:child_process";
import path from "node:path";
import {
  collectSourceDocuments,
  createWorkspaceAnalysisHost,
  createWorkspaceStyleHost,
} from "../server/engine-host-node/src/checker-host/workspace-check-support";
import { buildEngineInputV2 } from "../server/engine-host-node/src/engine-input-v2";
import { stableJsonStringify } from "./contract-parity-runtime";

type TypeBackend = "tsgo";

type OrderingFixture = {
  readonly fixture: string;
  readonly workspaceRoot: string;
  readonly sourceFilePaths: readonly string[];
  readonly styleFilePaths: readonly string[];
};

type CommandSpec = {
  readonly label: string;
  readonly args: readonly string[];
};

type CommandResult = {
  readonly label: string;
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string;
};

type CheckerControlConfig = {
  readonly label: string;
  readonly env: Readonly<Record<string, string>>;
};

const repoRoot = process.cwd();
const ORDERING_RUN_COUNT = 3 as const;
const PARALLEL_ROUND_COUNT = 3 as const;
const CHECKER_CONTROL_RUN_COUNT = 2 as const;
const TYPE_BACKEND: TypeBackend = "tsgo";

const orderingFixtures: readonly OrderingFixture[] = [
  {
    fixture: "literal-union",
    workspaceRoot: path.join(repoRoot, "test/_fixtures/type-fact-backend-parity/literal-union"),
    sourceFilePaths: [
      path.join(repoRoot, "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts"),
    ],
    styleFilePaths: [
      path.join(
        repoRoot,
        "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
      ),
    ],
  },
  {
    fixture: "path-alias",
    workspaceRoot: path.join(repoRoot, "test/_fixtures/type-fact-backend-parity/path-alias"),
    sourceFilePaths: [
      path.join(repoRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.ts"),
    ],
    styleFilePaths: [
      path.join(repoRoot, "test/_fixtures/type-fact-backend-parity/path-alias/src/App.module.scss"),
    ],
  },
  {
    fixture: "composite",
    workspaceRoot: path.join(repoRoot, "test/_fixtures/type-fact-backend-parity/composite"),
    sourceFilePaths: [
      path.join(repoRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
    ],
    styleFilePaths: [
      path.join(repoRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss"),
    ],
  },
] as const;

const parallelCommands: readonly CommandSpec[] = [
  {
    label: "release-batch",
    args: ["check:release-batch"],
  },
  {
    label: "real-project-corpus",
    args: ["check:real-project-corpus"],
  },
] as const;

const checkerControlConfigs: readonly CheckerControlConfig[] = [
  {
    label: "checkers-default",
    env: {},
  },
  {
    label: "checkers-1",
    env: { OMENA_TSGO_CHECKERS: "1" },
  },
  {
    label: "checkers-2",
    env: { OMENA_TSGO_CHECKERS: "2" },
  },
  {
    label: "checkers-4",
    env: { OMENA_TSGO_CHECKERS: "4" },
  },
] as const;

void (async () => {
  const orderingResults = await Promise.all(
    orderingFixtures.map(async (fixture) => {
      const snapshots = await Promise.all(
        Array.from({ length: ORDERING_RUN_COUNT }, () =>
          buildTypeFactSnapshot(fixture, TYPE_BACKEND),
        ),
      );
      const serialized = snapshots.map((snapshot) => stableJsonStringify(snapshot.typeFacts));
      const baseline = serialized[0] ?? "[]";
      const stableOrdering = serialized.every((value) => value === baseline);

      return {
        fixture: fixture.fixture,
        runCount: ORDERING_RUN_COUNT,
        typeFactCount: snapshots[0]?.typeFacts.length ?? 0,
        stableOrdering,
      };
    }),
  );

  const parallelResults = await runParallelRounds();
  const checkerControlResults = await runCheckerControlMatrix();
  const orderingStable = orderingResults.every((result) => result.stableOrdering);
  const parallelStable = parallelResults.every((result) => result.stableOutputs);
  const checkerControlsStable = checkerControlResults.every((result) => result.stableOutputs);
  const ok = orderingStable && parallelStable && checkerControlsStable;

  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "1",
        tool: "css-module-explainer/ts7-phase-a-stability",
        backend: TYPE_BACKEND,
        ordering: {
          runCount: ORDERING_RUN_COUNT,
          fixtures: orderingResults,
          ok: orderingStable,
        },
        parallel: {
          roundCount: PARALLEL_ROUND_COUNT,
          commands: parallelResults,
          ok: parallelStable,
        },
        checkerControls: {
          runCount: CHECKER_CONTROL_RUN_COUNT,
          commands: checkerControlResults,
          ok: checkerControlsStable,
        },
      },
      null,
      2,
    )}\n`,
  );

  process.exitCode = ok ? 0 : 1;
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});

async function buildTypeFactSnapshot(fixture: OrderingFixture, typeBackend: TypeBackend) {
  const styleHost = createWorkspaceStyleHost({
    styleFiles: fixture.styleFilePaths,
    classnameTransform: "asIs",
  });
  const analysisHost = createWorkspaceAnalysisHost({
    workspaceRoot: fixture.workspaceRoot,
    classnameTransform: "asIs",
    pathAlias: {},
    styleDocumentForPath: styleHost.styleDocumentForPath,
    typeBackend,
    env: {
      ...process.env,
      OMENA_TYPE_FACT_BACKEND: typeBackend,
    },
  });
  const sourceDocuments = collectSourceDocuments(
    fixture.sourceFilePaths,
    analysisHost.analysisCache,
  );

  return buildEngineInputV2({
    workspaceRoot: fixture.workspaceRoot,
    classnameTransform: "asIs",
    pathAlias: {},
    sourceDocuments,
    styleFiles: fixture.styleFilePaths,
    analysisCache: analysisHost.analysisCache,
    styleDocumentForPath: styleHost.styleDocumentForPath,
    typeBackend,
    env: {
      ...process.env,
      OMENA_TYPE_FACT_BACKEND: typeBackend,
    },
  });
}

async function runParallelRounds() {
  const baselineByLabel = new Map<string, CommandResult>();
  const results: Array<{
    readonly label: string;
    readonly roundCount: number;
    readonly stableOutputs: boolean;
    readonly baselineExitCode: number;
  }> = [];

  const rounds = await Promise.all(
    Array.from({ length: PARALLEL_ROUND_COUNT }, () =>
      Promise.all(parallelCommands.map((command) => runCommand(command.label, command.args))),
    ),
  );

  for (const [roundIndex, roundResults] of rounds.entries()) {
    for (const result of roundResults) {
      const baseline = baselineByLabel.get(result.label);
      if (!baseline) {
        baselineByLabel.set(result.label, result);
        continue;
      }

      if (
        baseline.exitCode !== result.exitCode ||
        baseline.stdout !== result.stdout ||
        baseline.stderr !== result.stderr
      ) {
        results.push({
          label: result.label,
          roundCount: roundIndex + 1,
          stableOutputs: false,
          baselineExitCode: baseline.exitCode,
        });
      }
    }
  }

  const unstableLabels = new Set(results.map((result) => result.label));

  for (const command of parallelCommands) {
    if (!unstableLabels.has(command.label)) {
      results.push({
        label: command.label,
        roundCount: PARALLEL_ROUND_COUNT,
        stableOutputs: true,
        baselineExitCode: baselineByLabel.get(command.label)?.exitCode ?? 1,
      });
    }
  }

  return results.toSorted((a, b) => a.label.localeCompare(b.label));
}

async function runCheckerControlMatrix() {
  return Promise.all(
    checkerControlConfigs.map(async (config) => {
      const baseline = await runCommand(
        "backend-typecheck-smoke",
        ["check:backend-typecheck-smoke"],
        {
          ...config.env,
          OMENA_TYPECHECK_VARIANT: TYPE_BACKEND,
        },
      );
      const repeats = await Promise.all(
        Array.from({ length: CHECKER_CONTROL_RUN_COUNT - 1 }, () =>
          runCommand("backend-typecheck-smoke", ["check:backend-typecheck-smoke"], {
            ...config.env,
            OMENA_TYPECHECK_VARIANT: TYPE_BACKEND,
          }),
        ),
      );
      const stableOutputs = repeats.every(
        (result) =>
          result.exitCode === baseline.exitCode &&
          result.stdout === baseline.stdout &&
          result.stderr === baseline.stderr,
      );

      return {
        label: config.label,
        runCount: CHECKER_CONTROL_RUN_COUNT,
        stableOutputs,
        baselineExitCode: baseline.exitCode,
      };
    }),
  );
}

function runCommand(
  label: string,
  args: readonly string[],
  envOverrides: Readonly<Record<string, string>> = {},
) {
  return new Promise<CommandResult>((resolve) => {
    const child = spawn("pnpm", [...args], {
      cwd: repoRoot,
      env: {
        ...process.env,
        OMENA_TYPE_FACT_BACKEND: TYPE_BACKEND,
        ...envOverrides,
      },
      stdio: ["ignore", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";

    child.stdout.on("data", (chunk: Buffer | string) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk: Buffer | string) => {
      stderr += chunk.toString();
    });

    child.on("close", (code) => {
      resolve({
        label,
        exitCode: code ?? 1,
        stdout: normalizeVolatileOutput(stdout),
        stderr: normalizeVolatileOutput(stderr),
      });
    });
  });
}

function normalizeVolatileOutput(output: string) {
  return output.replaceAll(/\(node:\d+\)/g, "(node:<pid>)");
}
