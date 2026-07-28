import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { API } from "@typescript/native-preview/unstable/async";
import {
  collectSourceDocuments,
  createWorkspaceAnalysisHost,
  createWorkspaceStyleHost,
} from "../server/engine-host-node/src/checker-host/workspace-check-support";
import { buildEngineInputV2Async } from "../server/engine-host-node/src/engine-input-v2";
import {
  buildTsgoTypeFactApiOptions,
  resolveTsgoSpanTypeFact,
  type TsgoSpanTypeFactResultEntry,
} from "../server/engine-host-node/src/tsgo-type-fact-collector";
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
  const exactSpanResults = await verifyExactSpanTypeFactParity();

  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "3",
        tool: "omena/type-fact-backend-parity",
        backend: "tsgo",
        results,
        exactSpanResults,
      },
      null,
      2,
    )}\n`,
  );

  for (const result of results) {
    assert.equal(result.matches, true, `${result.fixture}: tsgo type fact contract mismatch`);
  }
})();

async function verifyExactSpanTypeFactParity(): Promise<readonly TsgoSpanTypeFactResultEntry[]> {
  const workspaceRoot = path.join(
    repoRoot,
    "test/_fixtures/type-fact-backend-parity/literal-union",
  );
  const filePath = path.join(workspaceRoot, "src/App.ts");
  const source = readFileSync(filePath, "utf8");
  const api = new API(buildTsgoTypeFactApiOptions(workspaceRoot));
  let snapshot: Awaited<ReturnType<API["updateSnapshot"]>> | undefined;
  try {
    snapshot = await api.updateSnapshot({
      openProject: path.join(workspaceRoot, "tsconfig.json"),
    });
    const project = await snapshot.getDefaultProjectForFile(filePath);
    assert(project, `exact-span parity fixture has no tsgo project: ${filePath}`);
    const targets = [
      spanTarget(source, filePath, "call", "pickTone()"),
      spanTarget(source, filePath, "computed", "themeByKey[themeKey]"),
      spanTarget(source, filePath, "nested-template", "`nested-${nestedVariant}` as const"),
      spanTarget(source, filePath, "arithmetic", '"arithmetic-" + variant'),
      spanTarget(source, filePath, "logical", "enabled && logicalTone()"),
    ] as const;
    const results: TsgoSpanTypeFactResultEntry[] = [];
    for (const target of targets) {
      try {
        results.push(await resolveTsgoSpanTypeFact(project, target));
      } catch (error) {
        throw new Error(`exact-span parity failed for ${target.expressionId}`, { cause: error });
      }
    }

    assert.deepEqual(results[0]?.resolvedType.values, ["call-primary", "call-secondary"]);
    assert.deepEqual(results[1]?.resolvedType.values, ["computed-alpha", "computed-beta"]);
    assert.deepEqual(results[2]?.resolvedType.values, ["nested-large-strong", "nested-small-soft"]);
    assert.equal(results[3]?.outcome, "refused");
    assert.equal(results[3]?.reason, "nonExactDomain");
    assert.equal(results[4]?.outcome, "refused");
    assert.equal(results[4]?.reason, "nonExactDomain");
    assert.equal(results[4]?.nonNullishMemberCount, 2);
    assert.equal(results[4]?.resolvedMemberCount, 1);

    const callTarget = targets[0];
    const wrongSpan = await resolveTsgoSpanTypeFact(project, {
      ...callTarget,
      endPosition: callTarget.endPosition - 1,
    });
    assert.equal(wrongSpan.outcome, "refused");
    assert.equal(wrongSpan.reason, "nodeSpanMismatch");
    assert.equal(wrongSpan.spanExact, false);

    return [...results, wrongSpan];
  } finally {
    await snapshot?.dispose();
    await api.close();
  }
}

function spanTarget(source: string, filePath: string, expressionId: string, needle: string) {
  const startPosition = source.lastIndexOf(needle);
  assert(startPosition >= 0, `missing exact-span parity expression: ${needle}`);
  return {
    filePath,
    expressionId,
    startPosition,
    endPosition: startPosition + needle.length,
  };
}

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
