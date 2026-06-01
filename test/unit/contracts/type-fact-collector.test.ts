import { describe, expect, it } from "vitest";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import type { TypeResolver } from "../../../server/engine-core-ts/src/core/ts/type-resolver";
import { selectTypeFactCollector } from "../../../server/engine-host-node/src/type-fact-collector";
import {
  buildTsgoTypeFactWorkerInvocation,
  collectTypeFactTableV2WithTsgo,
  createTsgoTypeFactResolvedTypesCache,
} from "../../../server/engine-host-node/src/tsgo-type-fact-collector";
import type { TypeFactSourceEntry } from "../../../server/engine-host-node/src/historical/type-fact-table-v1";
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

  it("routes tsgo collection through the tsgo worker", () => {
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
      runTsgoTypeFactWorker: (input) => {
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

    expect(collector.collectV1({ workspaceRoot: "/repo", sourceEntries })[0]?.facts.kind).toBe(
      "finiteSet",
    );
    expect(collector.collectV2({ workspaceRoot: "/repo", sourceEntries })[0]?.facts.kind).toBe(
      "finiteSet",
    );
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

  it("returns unknown facts when tsgo has no project for a target file", () => {
    const collector = selectTypeFactCollector({
      typeBackend: "tsgo",
      typeResolver: finiteSetResolver(["primary", "secondary"]),
      findTsgoConfigFile: (workspaceRoot) => `${workspaceRoot}/tsconfig.json`,
      runTsgoTypeFactWorker: () => {
        throw new Error(
          "tsgo type fact worker failed\nstderr: no project found for file /repo/src/App.tsx",
        );
      },
    });

    const [entry] = collector.collectV2({
      workspaceRoot: "/repo",
      sourceEntries: createSourceEntries(),
    });

    expect(entry?.facts).toEqual({ kind: "unknown" });
  });

  it("returns unknown facts when the tsgo worker fails operationally", () => {
    const collector = selectTypeFactCollector({
      typeBackend: "tsgo",
      typeResolver: finiteSetResolver(["fallback"]),
      findTsgoConfigFile: (workspaceRoot) => `${workspaceRoot}/tsconfig.json`,
      runTsgoTypeFactWorker: () => {
        throw new Error("tsgo type fact worker failed\nstderr: spawn tsgo ENOENT");
      },
    });
    const sourceEntries = createSourceEntries();

    expect(collector.collectV1({ workspaceRoot: "/repo", sourceEntries })[0]?.facts).toEqual({
      kind: "unknown",
    });
    expect(collector.collectV2({ workspaceRoot: "/repo", sourceEntries })[0]?.facts).toEqual({
      kind: "unknown",
    });
  });

  it("reuses cached tsgo type facts for identical source snapshots", () => {
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
        runWorker: (input) => {
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

    expect(collect()[0]?.facts).toEqual({ kind: "finiteSet", values: ["primary", "secondary"] });
    expect(collect()[0]?.facts).toEqual({ kind: "finiteSet", values: ["primary", "secondary"] });
    expect(workerCalls).toHaveLength(1);
  });

  it("invalidates cached tsgo type facts when the source snapshot changes", () => {
    const workerCalls: unknown[] = [];
    const cache = createTsgoTypeFactResolvedTypesCache();

    const collect = (contentHash: string) =>
      collectTypeFactTableV2WithTsgo({
        workspaceRoot: "/repo",
        sourceEntries: createSourceEntries({ contentHash }),
        typeResolver: finiteSetResolver(["fallback"]),
        findConfigFile: (workspaceRoot) => `${workspaceRoot}/tsconfig.json`,
        workerCache: cache,
        runWorker: (input) => {
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

    collect("hash-1");
    collect("hash-2");

    expect(workerCalls).toHaveLength(2);
  });

  it("expires cached tsgo type facts after the burst window", () => {
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
        runWorker: (input) => {
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

    collect();
    now = 10;
    collect();

    expect(workerCalls).toHaveLength(2);
  });

  it("invalidates cached tsgo type facts when tsconfig content changes", () => {
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
        runWorker: (input) => {
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
      collect();
      writeFileSync(configPath, '{"compilerOptions":{"strict":false}}');
      collect();
    } finally {
      rmSync(workspaceRoot, { recursive: true, force: true });
    }

    expect(workerCalls).toHaveLength(2);
  });

  it("passes the packaged tsgo binary to the self-contained type fact worker", () => {
    const projectRoot = path.join("/extension", "css-module-explainer");
    const platformDir = `${process.platform}-${process.arch}`;
    const binaryName = process.platform === "win32" ? "tsgo.exe" : "tsgo";
    const packagedTsgoPath = path.join(projectRoot, "dist", "bin", platformDir, binaryName);

    const invocation = buildTsgoTypeFactWorkerInvocation(
      "/workspace",
      { OMENA_PROJECT_ROOT: projectRoot } as NodeJS.ProcessEnv,
      (filePath) => filePath === packagedTsgoPath,
    );

    expect(invocation.command).toBe(process.execPath);
    expect(invocation.args[0]).toBe("-e");
    expect(invocation.cwd).toBe("/workspace");
    expect(invocation.env.OMENA_TSGO_PATH).toBe(packagedTsgoPath);
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
  } = {},
): readonly TypeFactSourceEntry[] {
  return [
    {
      document: {
        uri: "file:///repo/src/App.tsx",
        filePath: "/repo/src/App.tsx",
        content: "variant",
        version: 1,
      },
      analysis: {
        version: 1,
        contentHash: options.contentHash ?? "hash",
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
}
