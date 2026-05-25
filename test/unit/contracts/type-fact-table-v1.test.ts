import { describe, expect, it } from "vitest";
import type { SourceDocumentSnapshot } from "../../../server/engine-host-node/src/checker-host/workspace-check-support";
import { collectTypeFactTableV1 } from "../../../server/engine-host-node/src/historical/type-fact-table-v1";
import type { AnalysisEntry } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import type { TypeResolver } from "../../../server/engine-core-ts/src/core/ts/type-resolver";
import {
  makeLiteralClassExpression,
  makeSourceDocumentHIR,
  makeSymbolRefClassExpression,
} from "../../../server/engine-core-ts/src/core/hir/source-types";

describe("collectTypeFactTableV1", () => {
  it("collects facts only for symbol-ref expressions and sorts deterministically", () => {
    const sourceEntries = [
      makeSourceEntry("/repo/src/B.tsx", [
        makeSymbolRefClassExpression(
          "expr-b",
          "cxCall",
          "/repo/src/App.module.scss",
          "size",
          "size",
          [],
          {
            start: { line: 1, character: 2 },
            end: { line: 1, character: 6 },
          },
        ),
      ]),
      makeSourceEntry("/repo/src/A.tsx", [
        makeLiteralClassExpression(
          "expr-a-literal",
          "cxCall",
          "/repo/src/App.module.scss",
          "static",
          {
            start: { line: 0, character: 0 },
            end: { line: 0, character: 6 },
          },
        ),
        makeSymbolRefClassExpression(
          "expr-a-symbol",
          "cxCall",
          "/repo/src/App.module.scss",
          "variant",
          "variant",
          [],
          {
            start: { line: 2, character: 1 },
            end: { line: 2, character: 8 },
          },
        ),
      ]),
    ];

    const table = collectTypeFactTableV1({
      workspaceRoot: "/repo",
      typeResolver: {
        resolve(filePath, variableName) {
          if (filePath.endsWith("A.tsx") && variableName === "variant") {
            return { kind: "union", values: ["primary", "secondary"] };
          }
          return { kind: "unresolvable", values: [] };
        },
        invalidate() {},
        clear() {},
      } satisfies TypeResolver,
      sourceEntries,
    });

    expect(table).toEqual([
      {
        filePath: "/repo/src/A.tsx",
        expressionId: "expr-a-symbol",
        facts: {
          kind: "finiteSet",
          values: ["primary", "secondary"],
        },
      },
      {
        filePath: "/repo/src/B.tsx",
        expressionId: "expr-b",
        facts: {
          kind: "unknown",
        },
      },
    ]);
  });
});

function makeSourceEntry(
  filePath: string,
  classExpressions: AnalysisEntry["sourceDocument"]["classExpressions"],
): {
  readonly document: SourceDocumentSnapshot;
  readonly analysis: AnalysisEntry;
} {
  return {
    document: {
      uri: `file://${filePath}`,
      filePath,
      content: "",
      version: 1,
    },
    analysis: {
      version: 1,
      contentHash: "hash",
      sourceFile: {} as AnalysisEntry["sourceFile"],
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
        classExpressions,
      }),
      stylesBindings: new Map(),
      classUtilNames: [],
      classValueUniverses: [],
      sourceDependencyPaths: [],
    },
  };
}
