import { describe, expect, it } from "vitest";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import ts from "typescript";
import type { TypeResolver } from "../../../server/engine-core-ts/src/core/ts/type-resolver";
import type { TypeFactControlFlowGraphV2 } from "../../../server/engine-core-ts/src/contracts";
import { selectTypeFactCollector } from "../../../server/engine-host-node/src/type-fact-collector";
import {
  buildTsgoTypeFactApiOptions,
  collectTypeFactTableV2WithTsgo,
  createTsgoTypeFactResolvedTypesCache,
} from "../../../server/engine-host-node/src/tsgo-type-fact-collector";
import type { TypeFactSourceEntry } from "../../../server/engine-host-node/src/historical/type-fact-table-v1";
import {
  rustTypeFactControlFlowGraphProvider,
  type RustTypeFactControlFlowGraphInput,
} from "../../../server/engine-host-node/src/type-fact-control-flow-graph";
import {
  makeSourceDocumentHIR,
  makeSymbolRefClassExpression,
} from "../../../server/engine-core-ts/src/core/hir/source-types";

describe("selectTypeFactCollector", () => {
  it("collects v1 and v2 facts through the selected resolver", () => {
    const collector = selectTypeFactCollector({
      typeBackend: "typescript-current",
      typeResolver: {
        resolve() {
          return { kind: "union", values: ["primary", "secondary"] };
        },
        invalidate() {},
        clear() {},
      } satisfies TypeResolver,
    });

    const sourceEntries: readonly TypeFactSourceEntry[] = [
      {
        document: {
          uri: "file:///repo/src/App.tsx",
          filePath: "/repo/src/App.tsx",
          content: "",
          version: 1,
        },
        analysis: {
          version: 1,
          contentHash: "hash",
          sourceFile: {} as TypeFactSourceEntry["analysis"]["sourceFile"],
          sourceBinder: {
            filePath: "/repo/src/App.tsx",
            scopes: [],
            decls: [],
          },
          sourceBindingGraph: {
            filePath: "/repo/src/App.tsx",
            nodes: [],
            edges: [],
          },
          sourceDocument: makeSourceDocumentHIR({
            filePath: "/repo/src/App.tsx",
            language: "tsx",
            styleImports: [],
            utilityBindings: [],
            classExpressions: [
              makeSymbolRefClassExpression(
                "expr-1",
                "cxCall",
                "/repo/src/App.module.scss",
                "variant",
                "variant",
                [],
                {
                  start: { line: 0, character: 0 },
                  end: { line: 0, character: 7 },
                },
              ),
            ],
          }),
          stylesBindings: new Map(),
          classUtilNames: [],
          classValueUniverses: [],
          sourceDependencyPaths: [],
        },
      },
    ];

    expect(collector.backend).toBe("typescript-current");
    expect(collector.collectV1({ workspaceRoot: "/repo", sourceEntries })[0]?.facts.kind).toBe(
      "finiteSet",
    );
    expect(collector.collectV2({ workspaceRoot: "/repo", sourceEntries })[0]?.facts.kind).toBe(
      "finiteSet",
    );
  });

  it("carries source control-flow blocks on v2 type facts", () => {
    const collector = selectTypeFactCollector({
      typeBackend: "typescript-current",
      typeResolver: finiteSetResolver(["primary", "secondary"]),
    });
    const source = `
function render(flag: boolean, variant: string) {
  let size = "btn-" + variant;
  if (flag) {
    size = "primary";
  }
  return cx(size);
}
`;

    const [entry] = collector.collectV2({
      workspaceRoot: "/repo",
      sourceEntries: createSourceEntries({
        source,
        range: rangeOf(source, "cx(size)"),
        rootName: "size",
      }),
    });

    expect(entry?.controlFlowGraph?.entryBlockId).toBe("entry");
    expect(entry?.controlFlowGraph?.blocks).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          id: "assignment:0",
          kind: "assignment",
          transferKind: "concatFacts",
          successorBlockIds: ["branch:0"],
          variableName: "size",
        }),
        expect.objectContaining({
          id: "branch:0",
          kind: "branch",
          transferKind: "branch",
          successorBlockIds: expect.arrayContaining(["assignment:1", "join:0"]),
        }),
        expect.objectContaining({
          id: "join:0",
          kind: "join",
          transferKind: "join",
        }),
      ]),
    );
  });

  it("can source v2 control-flow graphs through a Rust frontend provider", () => {
    const graph: TypeFactControlFlowGraphV2 = {
      entryBlockId: "rust-entry",
      blocks: [
        {
          id: "rust-entry",
          kind: "entry",
          transferKind: "entry",
          successorBlockIds: ["rust-ref"],
        },
        {
          id: "rust-ref",
          kind: "assignment",
          transferKind: "assignFacts",
          successorBlockIds: [],
          variableName: "size",
        },
      ],
    };
    const providerCalls: RustTypeFactControlFlowGraphInput[] = [];
    const collector = selectTypeFactCollector({
      typeBackend: "typescript-current",
      typeResolver: finiteSetResolver(["primary", "secondary"]),
      controlFlowGraphProvider: rustTypeFactControlFlowGraphProvider((input) => {
        providerCalls.push(input);
        return JSON.stringify(graph);
      }),
    });
    const source = `
function render(size: string) {
  return cx(size);
}
`;

    const [entry] = collector.collectV2({
      workspaceRoot: "/repo",
      sourceEntries: createSourceEntries({
        source,
        range: rangeOf(source, "cx(size)"),
        rootName: "size",
      }),
    });

    expect(entry?.controlFlowGraph).toEqual(graph);
    expect(providerCalls).toHaveLength(1);
    expect(providerCalls[0]).toEqual(
      expect.objectContaining({
        sourcePath: "/repo/src/App.tsx",
        source,
        sourceLanguage: "typescriptreact",
        variableName: "size",
      }),
    );
    expect(providerCalls[0]?.referenceByteOffset).toBeGreaterThan(0);
  });

  it("routes tsgo collection through the tsgo worker", async () => {
    const workerCalls: Array<{
      workspaceRoot: string;
      configPath: string;
      targets: readonly { filePath: string; expressionId: string; position: number }[];
    }> = [];
    const collector = selectTypeFactCollector({
      typeBackend: "tsgo",
      typeResolver: {
        resolve() {
          return { kind: "unresolvable", values: [] };
        },
        invalidate() {},
        clear() {},
      } satisfies TypeResolver,
      findTsgoConfigFile: (workspaceRoot) => `${workspaceRoot}/tsconfig.json`,
      runTsgoTypeFactWorker: async (input) => {
        workerCalls.push(input);
        return [
          {
            filePath: "/repo/src/App.tsx",
            expressionId: "expr-1",
            resolvedType: { kind: "union", values: ["primary", "secondary"] },
          },
        ];
      },
    });

    const sourceEntries = createSourceEntries();

    expect(
      (await collector.collectV1Async({ workspaceRoot: "/repo", sourceEntries }))[0]?.facts.kind,
    ).toBe("finiteSet");
    expect(
      (await collector.collectV2Async({ workspaceRoot: "/repo", sourceEntries }))[0]?.facts.kind,
    ).toBe("finiteSet");
    expect(workerCalls).toHaveLength(2);
    expect(workerCalls[0]?.configPath).toBe("/repo/tsconfig.json");
    expect(workerCalls[0]?.targets[0]?.position).toBe(0);
  });

  it("honors an explicit non-tsgo resolver even when the ambient default is tsgo", () => {
    const collector = selectTypeFactCollector({
      env: { OMENA_TYPE_FACT_BACKEND: "tsgo" },
      typeResolver: finiteSetResolver(["primary", "secondary"]),
    });

    const [entry] = collector.collectV2({
      workspaceRoot: "/repo",
      sourceEntries: createSourceEntries(),
    });

    expect(collector.backend).toBe("tsgo");
    expect(entry?.facts).toEqual({ kind: "finiteSet", values: ["primary", "secondary"] });
  });

  it("returns unknown facts when tsgo has no project for a target file", async () => {
    const collector = selectTypeFactCollector({
      typeBackend: "tsgo",
      typeResolver: finiteSetResolver(["primary", "secondary"]),
      findTsgoConfigFile: (workspaceRoot) => `${workspaceRoot}/tsconfig.json`,
      runTsgoTypeFactWorker: async () => {
        throw new Error(
          "tsgo type fact worker failed\nstderr: no project found for file /repo/src/App.tsx",
        );
      },
    });

    const [entry] = await collector.collectV2Async({
      workspaceRoot: "/repo",
      sourceEntries: createSourceEntries(),
    });

    expect(entry?.facts).toEqual({ kind: "unknown" });
  });

  it("returns unknown facts when the tsgo worker fails operationally", async () => {
    const collector = selectTypeFactCollector({
      typeBackend: "tsgo",
      typeResolver: finiteSetResolver(["fallback"]),
      findTsgoConfigFile: (workspaceRoot) => `${workspaceRoot}/tsconfig.json`,
      runTsgoTypeFactWorker: async () => {
        throw new Error("tsgo type fact worker failed\nstderr: spawn tsgo ENOENT");
      },
    });
    const sourceEntries = createSourceEntries();

    expect(
      (await collector.collectV1Async({ workspaceRoot: "/repo", sourceEntries }))[0]?.facts,
    ).toEqual({
      kind: "unknown",
    });
    expect(
      (await collector.collectV2Async({ workspaceRoot: "/repo", sourceEntries }))[0]?.facts,
    ).toEqual({
      kind: "unknown",
    });
  });

  it("reuses cached tsgo type facts for identical source snapshots", async () => {
    const workerCalls: unknown[] = [];
    const cache = createTsgoTypeFactResolvedTypesCache();
    const sourceEntries = createSourceEntries();

    const collect = () =>
      collectTypeFactTableV2WithTsgo({
        workspaceRoot: "/repo",
        sourceEntries,
        typeResolver: finiteSetResolver(["fallback"]),
        findConfigFile: (workspaceRoot) => `${workspaceRoot}/tsconfig.json`,
        workerCache: cache,
        runWorker: async (input) => {
          workerCalls.push(input);
          return [
            {
              filePath: "/repo/src/App.tsx",
              expressionId: "expr-1",
              resolvedType: { kind: "union", values: ["primary", "secondary"] },
            },
          ];
        },
      });

    expect((await collect())[0]?.facts).toEqual({
      kind: "finiteSet",
      values: ["primary", "secondary"],
    });
    expect((await collect())[0]?.facts).toEqual({
      kind: "finiteSet",
      values: ["primary", "secondary"],
    });
    expect(workerCalls).toHaveLength(1);
  });

  it("invalidates cached tsgo type facts when the source snapshot changes", async () => {
    const workerCalls: unknown[] = [];
    const cache = createTsgoTypeFactResolvedTypesCache();

    const collect = (contentHash: string) =>
      collectTypeFactTableV2WithTsgo({
        workspaceRoot: "/repo",
        sourceEntries: createSourceEntries({ contentHash }),
        typeResolver: finiteSetResolver(["fallback"]),
        findConfigFile: (workspaceRoot) => `${workspaceRoot}/tsconfig.json`,
        workerCache: cache,
        runWorker: async (input) => {
          workerCalls.push(input);
          return [
            {
              filePath: "/repo/src/App.tsx",
              expressionId: "expr-1",
              resolvedType: { kind: "union", values: ["primary"] },
            },
          ];
        },
      });

    await collect("hash-1");
    await collect("hash-2");

    expect(workerCalls).toHaveLength(2);
  });

  it("expires cached tsgo type facts after the burst window", async () => {
    let now = 0;
    const workerCalls: unknown[] = [];
    const cache = createTsgoTypeFactResolvedTypesCache(64, 10, () => now);
    const sourceEntries = createSourceEntries();

    const collect = () =>
      collectTypeFactTableV2WithTsgo({
        workspaceRoot: "/repo",
        sourceEntries,
        typeResolver: finiteSetResolver(["fallback"]),
        findConfigFile: (workspaceRoot) => `${workspaceRoot}/tsconfig.json`,
        workerCache: cache,
        runWorker: async (input) => {
          workerCalls.push(input);
          return [
            {
              filePath: "/repo/src/App.tsx",
              expressionId: "expr-1",
              resolvedType: { kind: "union", values: ["primary"] },
            },
          ];
        },
      });

    await collect();
    now = 10;
    await collect();

    expect(workerCalls).toHaveLength(2);
  });

  it("invalidates cached tsgo type facts when tsconfig content changes", async () => {
    const workspaceRoot = mkdtempSync(path.join(tmpdir(), "cme-tsgo-cache-"));
    const configPath = path.join(workspaceRoot, "tsconfig.json");
    const workerCalls: unknown[] = [];
    const cache = createTsgoTypeFactResolvedTypesCache();
    const sourceEntries = createSourceEntries();

    const collect = () =>
      collectTypeFactTableV2WithTsgo({
        workspaceRoot,
        sourceEntries,
        typeResolver: finiteSetResolver(["fallback"]),
        findConfigFile: () => configPath,
        workerCache: cache,
        runWorker: async (input) => {
          workerCalls.push(input);
          return [
            {
              filePath: "/repo/src/App.tsx",
              expressionId: "expr-1",
              resolvedType: { kind: "union", values: ["primary"] },
            },
          ];
        },
      });

    try {
      writeFileSync(configPath, '{"compilerOptions":{"strict":true}}');
      await collect();
      writeFileSync(configPath, '{"compilerOptions":{"strict":false}}');
      await collect();
    } finally {
      rmSync(workspaceRoot, { recursive: true, force: true });
    }

    expect(workerCalls).toHaveLength(2);
  });

  it("passes the packaged tsgo binary to the in-process type fact API", () => {
    const projectRoot = path.join("/extension", "css-module-explainer");
    const platformDir = `${process.platform}-${process.arch}`;
    const binaryName = process.platform === "win32" ? "tsgo.exe" : "tsgo";
    const packagedTsgoPath = path.join(projectRoot, "dist", "bin", platformDir, binaryName);

    const apiOptions = buildTsgoTypeFactApiOptions(
      "/workspace",
      { OMENA_PROJECT_ROOT: projectRoot } as NodeJS.ProcessEnv,
      (filePath) => filePath === packagedTsgoPath,
    );

    expect(apiOptions.cwd).toBe("/workspace");
    expect(apiOptions.tsserverPath).toBe(packagedTsgoPath);
  });
});

function finiteSetResolver(values: readonly string[]): TypeResolver {
  return {
    resolve() {
      return { kind: "union", values: [...values] };
    },
    invalidate() {},
    clear() {},
  };
}

function createSourceEntries(
  options: {
    readonly contentHash?: string;
    readonly source?: string;
    readonly rootName?: string;
    readonly range?: {
      readonly start: { readonly line: number; readonly character: number };
      readonly end: { readonly line: number; readonly character: number };
    };
  } = {},
): readonly TypeFactSourceEntry[] {
  const filePath = "/repo/src/App.tsx";
  const content = options.source ?? "variant";
  const sourceFile =
    options.source === undefined
      ? ({} as TypeFactSourceEntry["analysis"]["sourceFile"])
      : ts.createSourceFile(filePath, content, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
  const range = options.range ?? {
    start: { line: 0, character: 0 },
    end: { line: 0, character: 7 },
  };

  return [
    {
      document: {
        uri: "file:///repo/src/App.tsx",
        filePath,
        content,
        version: 1,
      },
      analysis: {
        version: 1,
        contentHash: options.contentHash ?? "hash",
        sourceFile,
        sourceBinder: {
          filePath,
          scopes: [],
          decls: [],
        },
        sourceBindingGraph: {
          filePath,
          nodes: [],
          edges: [],
        },
        sourceDocument: makeSourceDocumentHIR({
          filePath,
          language: "tsx",
          styleImports: [],
          utilityBindings: [],
          classExpressions: [
            makeSymbolRefClassExpression(
              "expr-1",
              "cxCall",
              "/repo/src/App.module.scss",
              options.rootName ?? "variant",
              options.rootName ?? "variant",
              [],
              range,
            ),
          ],
        }),
        stylesBindings: new Map(),
        classUtilNames: [],
        classValueUniverses: [],
        sourceDependencyPaths: [],
      },
    },
  ];
}

function rangeOf(source: string, token: string) {
  const tokenIndex = source.lastIndexOf(token);
  const startIndex = tokenIndex + token.indexOf("size");
  const prefix = source.slice(0, startIndex);
  const line = prefix.split("\n").length - 1;
  const lastLineStart = prefix.lastIndexOf("\n");
  const character = startIndex - (lastLineStart + 1);
  return {
    start: { line, character },
    end: { line, character: character + 4 },
  };
}
