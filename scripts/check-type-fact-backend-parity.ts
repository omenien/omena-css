import { strict as assert } from "node:assert";
import path from "node:path";
import {
  collectSourceDocuments,
  createWorkspaceAnalysisHost,
  createWorkspaceStyleHost,
} from "../server/engine-host-node/src/checker-host/workspace-check-support";
import { buildEngineInputV2Async } from "../server/engine-host-node/src/engine-input-v2";
import { stableJsonStringify } from "./contract-parity-runtime";

const repoRoot = process.cwd();

const parityFixtures = [
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
    expectedFacts: { kind: "finiteSet", values: ["button-primary", "button-secondary"] },
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
    expectedFacts: { kind: "finiteSet", values: ["button-primary", "button-secondary"] },
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
    expectedFacts: {
      kind: "constrained",
      constraintKind: "composite",
      prefix: "btn-",
      suffix: "-active",
      minLen: 12,
      charMust: "-abceintv",
      charMay: "-abcdefghijntv",
      provenance: "finiteSetWideningComposite",
    },
  },
] as const;

void (async () => {
  const results = await Promise.all(
    parityFixtures.map(async (entry) => {
      const snapshot = await buildTsgoTypeFactSnapshot(entry);
      const matches =
        snapshot.typeFacts.length === 1 &&
        stableJsonStringify(snapshot.typeFacts[0]?.facts) ===
          stableJsonStringify(entry.expectedFacts);

      return {
        fixture: entry.fixture,
        matches,
        expectedFacts: entry.expectedFacts,
        actualTypeFacts: snapshot.typeFacts,
      };
    }),
  );

  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "3",
        tool: "omena/type-fact-backend-parity",
        backend: "tsgo",
        results,
      },
      null,
      2,
    )}\n`,
  );

  for (const result of results) {
    assert.equal(result.matches, true, `${result.fixture}: tsgo type fact contract mismatch`);
  }
})();

async function buildTsgoTypeFactSnapshot(fixture: {
  readonly workspaceRoot: string;
  readonly sourceFilePaths: readonly string[];
  readonly styleFilePaths: readonly string[];
}) {
  const styleFiles = fixture.styleFilePaths;
  const styleHost = createWorkspaceStyleHost({
    styleFiles,
    classnameTransform: "asIs",
  });
  const analysisHost = createWorkspaceAnalysisHost({
    workspaceRoot: fixture.workspaceRoot,
    classnameTransform: "asIs",
    pathAlias: {},
    styleDocumentForPath: styleHost.styleDocumentForPath,
    typeBackend: "tsgo",
    env: {
      ...process.env,
      OMENA_TYPE_FACT_BACKEND: "tsgo",
    },
  });
  const sourceDocuments = collectSourceDocuments(
    fixture.sourceFilePaths,
    analysisHost.analysisCache,
  );

  return await buildEngineInputV2Async({
    workspaceRoot: fixture.workspaceRoot,
    classnameTransform: "asIs",
    pathAlias: {},
    sourceDocuments,
    styleFiles,
    analysisCache: analysisHost.analysisCache,
    styleDocumentForPath: styleHost.styleDocumentForPath,
    typeBackend: "tsgo",
    env: {
      ...process.env,
      OMENA_TYPE_FACT_BACKEND: "tsgo",
    },
  });
}
