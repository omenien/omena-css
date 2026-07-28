import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
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
  TSGO_EXACT_DOMAIN_MEMBER_LIMIT,
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
  const exactSpanParity = await verifyExactSpanTypeFactParity();
  const reasonVocabulary = auditExactSpanReasonVocabulary();

  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "3",
        tool: "omena/type-fact-backend-parity",
        backends: ["node-api", "rust-json-rpc"],
        results,
        exactSpanParity,
        reasonVocabulary,
        unionPolicy: {
          maximumMemberCount: TSGO_EXACT_DOMAIN_MEMBER_LIMIT,
          basis: "bounded hot-path fanout; domains above the limit remain unresolved",
        },
      },
      null,
      2,
    )}\n`,
  );

  for (const result of results) {
    assert.equal(result.matches, true, `${result.fixture}: tsgo type fact contract mismatch`);
  }
})();

async function verifyExactSpanTypeFactParity(): Promise<Readonly<Record<string, unknown>>> {
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
    const nodeResults: TsgoSpanTypeFactResultEntry[] = [];
    for (const target of targets) {
      try {
        nodeResults.push(await resolveTsgoSpanTypeFact(project, target));
      } catch (error) {
        throw new Error(`exact-span parity failed for ${target.expressionId}`, { cause: error });
      }
    }

    assert.deepEqual(nodeResults[0]?.resolvedType.values, ["call-primary", "call-secondary"]);
    assert.deepEqual(nodeResults[1]?.resolvedType.values, ["computed-alpha", "computed-beta"]);
    assert.deepEqual(nodeResults[2]?.resolvedType.values, [
      "nested-large-strong",
      "nested-small-soft",
    ]);
    assert.equal(nodeResults[3]?.outcome, "refused");
    assert.equal(nodeResults[3]?.reason, "nonExactDomain");
    assert.equal(nodeResults[4]?.outcome, "refused");
    assert.equal(nodeResults[4]?.reason, "nonExactUnionMember");
    assert.equal(nodeResults[4]?.nonNullishMemberCount, 2);
    assert.equal(nodeResults[4]?.resolvedMemberCount, 1);

    const callTarget = targets[0];
    const wrongSpan = await resolveTsgoSpanTypeFact(project, {
      ...callTarget,
      endPosition: callTarget.endPosition - 1,
    });
    assert.equal(wrongSpan.outcome, "refused");
    assert.equal(wrongSpan.reason, "nodeSpanMismatch");
    assert.equal(wrongSpan.spanExact, false);

    const allTargets = [
      ...targets,
      {
        ...callTarget,
        expressionId: "wrong-span",
        endPosition: callTarget.endPosition - 1,
      },
    ];
    const allNodeResults = [...nodeResults, { ...wrongSpan, expressionId: "wrong-span" }];
    const rustResults = captureRustExactSpanTypeFacts(
      workspaceRoot,
      path.join(workspaceRoot, "tsconfig.json"),
      allTargets,
    );
    const normalizedNodeResults = allNodeResults.map(normalizeExactSpanResult);
    const normalizedRustResults = rustResults.map(normalizeExactSpanResult);
    assert.deepEqual(
      normalizedRustResults,
      normalizedNodeResults,
      "Node API and Rust JSON-RPC exact-span backends must agree",
    );

    const admittedDeltaClasses = ["backendOnlyDiagnosticField", "orderingNonSemantic"] as const;
    const admittedDeltas: readonly {
      readonly fixture: string;
      readonly field: string;
      readonly class: (typeof admittedDeltaClasses)[number];
    }[] = [];
    for (const delta of admittedDeltas) {
      assert(
        admittedDeltaClasses.includes(delta.class),
        `unclassified backend parity delta: ${delta.fixture}:${delta.field}`,
      );
      assert.notEqual(delta.field, "reason", "refusal-reason deltas are never admissible");
    }
    const oversizedUnion = await verifyOversizedUnionParity();

    return {
      nodeResults: normalizedNodeResults,
      rustResults: normalizedRustResults,
      admittedDeltaClasses,
      admittedDeltas,
      oversizedUnion,
    };
  } finally {
    await snapshot?.dispose();
    await api.close();
  }
}

async function verifyOversizedUnionParity(): Promise<Readonly<Record<string, unknown>>> {
  const workspaceRoot = mkdtempSync(path.join(tmpdir(), "omena-tsgo-union-limit-"));
  const sourceDirectory = path.join(workspaceRoot, "src");
  const filePath = path.join(sourceDirectory, "App.ts");
  const configPath = path.join(workspaceRoot, "tsconfig.json");
  const members = Array.from(
    { length: TSGO_EXACT_DOMAIN_MEMBER_LIMIT + 1 },
    (_, index) => `"tone-${index.toString().padStart(3, "0")}"`,
  );
  mkdirSync(sourceDirectory, { recursive: true });
  writeFileSync(
    configPath,
    `${JSON.stringify({ compilerOptions: { strict: true }, include: ["src/**/*.ts"] }, null, 2)}\n`,
  );
  writeFileSync(
    filePath,
    `declare const oversizedTone: ${members.join(" | ")};\nexport const selectedTone = oversizedTone;\n`,
  );
  const source = readFileSync(filePath, "utf8");
  const target = spanTarget(source, filePath, "oversized-union", "oversizedTone");
  const api = new API(buildTsgoTypeFactApiOptions(workspaceRoot));
  let snapshot: Awaited<ReturnType<API["updateSnapshot"]>> | undefined;
  try {
    snapshot = await api.updateSnapshot({ openProject: configPath });
    const project = await snapshot.getDefaultProjectForFile(filePath);
    assert(project, `oversized-union fixture has no tsgo project: ${filePath}`);
    const nodeResult = await resolveTsgoSpanTypeFact(project, target);
    const rustResult = captureRustExactSpanTypeFacts(workspaceRoot, configPath, [target])[0];
    const normalizedNode = normalizeExactSpanResult(nodeResult);
    const normalizedRust = normalizeExactSpanResult(rustResult);
    assert.deepEqual(
      normalizedRust,
      normalizedNode,
      "Node API and Rust JSON-RPC backends must apply the same union fanout limit",
    );
    assert.equal(normalizedNode.outcome, "refused");
    assert.equal(normalizedNode.reason, "unionMemberLimitExceeded");
    assert.equal(normalizedNode.nonNullishMemberCount, TSGO_EXACT_DOMAIN_MEMBER_LIMIT + 1);
    return {
      memberCount: members.length,
      nodeResult: normalizedNode,
      rustResult: normalizedRust,
    };
  } finally {
    await snapshot?.dispose();
    await api.close();
    rmSync(workspaceRoot, { recursive: true, force: true });
  }
}

function captureRustExactSpanTypeFacts(
  workspaceRoot: string,
  configPath: string,
  targets: readonly {
    readonly filePath: string;
    readonly expressionId: string;
    readonly startPosition: number;
    readonly endPosition: number;
  }[],
): TsgoSpanTypeFactResultEntry[] {
  const tsgoPath = path.join(
    repoRoot,
    "node_modules/.bin",
    process.platform === "win32" ? "tsgo.cmd" : "tsgo",
  );
  const capture = spawnSync(
    "cargo",
    ["run", "--quiet", "-p", "omena-tsgo-client", "--bin", "omena-tsgo-span-type-fact-capture"],
    {
      cwd: path.join(repoRoot, "rust"),
      encoding: "utf8",
      input: JSON.stringify({
        tsgoPath,
        request: {
          workspaceRoot,
          configPath,
          targets,
        },
      }),
      maxBuffer: 16 * 1024 * 1024,
    },
  );
  assert.equal(
    capture.status,
    0,
    `Rust exact-span capture failed:\n${capture.stderr || capture.stdout}`,
  );
  return JSON.parse(capture.stdout) as TsgoSpanTypeFactResultEntry[];
}

function normalizeExactSpanResult(entry: TsgoSpanTypeFactResultEntry) {
  return {
    expressionId: entry.expressionId,
    outcome: entry.outcome,
    reason: entry.reason,
    spanExact: entry.spanExact,
    nonNullishMemberCount: entry.nonNullishMemberCount,
    resolvedMemberCount: entry.resolvedMemberCount,
    resolvedType: {
      kind: entry.resolvedType.kind,
      values: [...entry.resolvedType.values].sort(),
    },
  };
}

function auditExactSpanReasonVocabulary(): Readonly<Record<string, unknown>> {
  const rustSource = readFileSync(
    path.join(repoRoot, "rust/crates/omena-tsgo-client/src/lib.rs"),
    "utf8",
  );
  const nodeSource = readFileSync(
    path.join(repoRoot, "server/engine-host-node/src/tsgo-type-fact-collector.ts"),
    "utf8",
  );
  const rustConstants = new Map(
    [
      ...rustSource.matchAll(
        /^pub const (TSGO_SPAN_TYPE_FACT_REASON_[A-Z0-9_]+_V0): &str =\s*"([^"]+)";$/gmu,
      ),
    ].map((match) => [match[1], match[2]] as const),
  );
  const vocabularyBlock =
    /pub const TSGO_SPAN_TYPE_FACT_REASON_VOCABULARY_V0: \[&str; \d+\] = \[([\s\S]*?)\];/u.exec(
      rustSource,
    );
  assert(vocabularyBlock, "missing Rust exact-span reason vocabulary");
  const rustVocabulary = [
    ...vocabularyBlock[1].matchAll(/TSGO_SPAN_TYPE_FACT_REASON_[A-Z0-9_]+_V0/gu),
  ].map((match) => {
    const value = rustConstants.get(match[0]);
    assert(value, `missing Rust exact-span reason constant ${match[0]}`);
    return value;
  });
  const nodeVocabularyBlock =
    /export const TSGO_SPAN_TYPE_FACT_REASON = \{([\s\S]*?)\} as const;/u.exec(nodeSource);
  assert(nodeVocabularyBlock, "missing Node exact-span reason vocabulary");
  const nodeReasonByKey = new Map(
    [...nodeVocabularyBlock[1].matchAll(/^\s*([a-zA-Z0-9_]+): "([^"]+)",$/gmu)].map(
      (match) => [match[1], match[2]] as const,
    ),
  );
  const nodeVocabulary = [...nodeReasonByKey.values()];
  assert.deepEqual(
    [...nodeVocabulary].sort(),
    [...rustVocabulary].sort(),
    "Rust and Node exact-span reason vocabularies must agree",
  );
  const rustLimitMatch = /^pub const TSGO_EXACT_DOMAIN_MEMBER_LIMIT_V0: usize = (\d+);$/mu.exec(
    rustSource,
  );
  assert(rustLimitMatch, "missing Rust exact-domain member limit");
  const rustMemberLimit = Number(rustLimitMatch[1]);
  assert.equal(
    rustMemberLimit,
    TSGO_EXACT_DOMAIN_MEMBER_LIMIT,
    "Rust and Node exact-domain member limits must agree",
  );

  const rustProducerSource = [
    "collect_span_type_facts_for_snapshot",
    "resolve_exact_span_type_response",
    "resolved_span_type_fact",
  ]
    .map((name) => extractRustFunctionSource(rustSource, name))
    .join("\n");
  const rustEmitted = [
    ...new Set(
      [...rustProducerSource.matchAll(/TSGO_SPAN_TYPE_FACT_REASON_[A-Z0-9_]+_V0/gu)].map(
        (match) => {
          const value = rustConstants.get(match[0]);
          assert(value, `unbound Rust reason reference ${match[0]}`);
          return value;
        },
      ),
    ),
  ].sort();
  const nodeProducerSource = ["resolveTsgoSpanTypeFact", "extractExactStringDomain"]
    .map((name) => extractFunctionBody(nodeSource, `function ${name}`))
    .join("\n");
  const nodeEmitted = [
    ...new Set(
      [...nodeProducerSource.matchAll(/TSGO_SPAN_TYPE_FACT_REASON\.([a-zA-Z0-9_]+)/gu)].map(
        (match) => {
          const value = nodeReasonByKey.get(match[1]);
          assert(value, `unbound Node reason reference ${match[1]}`);
          return value;
        },
      ),
    ),
  ].sort();
  const declared = [...rustVocabulary].sort();
  assert(
    rustEmitted.every((reason) => declared.includes(reason)),
    "Rust emitted undeclared reason",
  );
  assert(
    nodeEmitted.every((reason) => declared.includes(reason)),
    "Node emitted undeclared reason",
  );
  assert.deepEqual(
    [...new Set([...rustEmitted, ...nodeEmitted])].sort(),
    declared,
    "declared exact-span reason vocabulary must equal the source-enumerated producer union",
  );

  return {
    declared,
    rustEmitted,
    nodeEmitted,
    maximumMemberCount: rustMemberLimit,
  };
}

function extractRustFunctionSource(source: string, name: string): string {
  for (const indentation of ["    ", ""]) {
    const signature = `\n${indentation}fn ${name}`;
    const start = source.indexOf(signature);
    if (start < 0) continue;
    const nextFunction = source.indexOf(`\n${indentation}fn `, start + signature.length);
    const nextImplEnd =
      indentation.length > 0 ? source.indexOf("\n}", start + signature.length) : -1;
    const candidates = [nextFunction, nextImplEnd].filter((index) => index >= 0);
    const end = candidates.length > 0 ? Math.min(...candidates) : source.length;
    return source.slice(start, end);
  }
  throw new Error(`missing Rust function ${name}`);
}

function extractFunctionBody(source: string, signature: string): string {
  const signatureIndex = source.indexOf(signature);
  assert(signatureIndex >= 0, `missing function ${signature}`);
  const open = source.indexOf("{", signatureIndex);
  assert(open >= 0, `missing function body ${signature}`);
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(open + 1, index);
    }
  }
  throw new Error(`unterminated function body ${signature}`);
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
