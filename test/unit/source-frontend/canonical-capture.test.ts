import { describe, expect, it } from "vitest";
import ts from "typescript";
import { buildSourceBinder } from "../../../server/engine-core-ts/src/core/binder/binder-builder";
import { buildSourceBindingGraph } from "../../../server/engine-core-ts/src/core/binder/source-binding-graph";
import { cssModulesClassnamesBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/binder-plugin";
import { AliasResolver } from "../../../server/engine-core-ts/src/core/cx/alias-resolver";
import { buildSourceDocument } from "../../../server/engine-core-ts/src/core/hir/builders/ts-source-adapter";
import {
  captureTsSourceFrontendFactsV0,
  stringifyCanonicalSourceFrontendJsonV0,
} from "../../../server/engine-core-ts/src/core/source-frontend/canonical-capture";

const aliasResolver = new AliasResolver("/fake/ws", {});

describe("captureTsSourceFrontendFactsV0", () => {
  it("serializes TS frontend syntax, binding, and CFG facts deterministically", () => {
    const first = captureFixture();
    const second = captureFixture();

    expect(stringifyCanonicalSourceFrontendJsonV0(first)).toBe(
      stringifyCanonicalSourceFrontendJsonV0(second),
    );
    expect(first.syntax.importedStyleBindings).toEqual([
      {
        binding: "styles",
        styleUri: "file:///fake/ws/src/Card.module.scss",
      },
    ]);
    expect(first.syntax.stylePropertyAccesses).toEqual([
      {
        byteSpan: { start: 371, end: 375 },
        selectorName: "icon",
        targetStyleUri: "file:///fake/ws/src/Card.module.scss",
      },
    ]);
    expect(first.bindingGraph.nodes.length).toBeGreaterThan(0);
    expect(first.bindingGraph.edges.length).toBeGreaterThan(0);
    expect(first.cfgSnapshot?.snapshot.blocks.some((block) => block.kind === "branch")).toBe(true);
  });

  it("keeps single-field divergence observable in the canonical JSON", () => {
    const capture = captureFixture();
    const divergent = {
      ...capture,
      syntax: {
        ...capture.syntax,
        stylePropertyAccesses: [
          {
            ...capture.syntax.stylePropertyAccesses[0]!,
            selectorName: "wrong",
          },
        ],
      },
    };

    expect(stringifyCanonicalSourceFrontendJsonV0(divergent)).not.toBe(
      stringifyCanonicalSourceFrontendJsonV0(capture),
    );
  });
});

function captureFixture() {
  const source = [
    'import classNames from "classnames/bind";',
    'import clsx from "clsx";',
    'import styles from "./Card.module.scss";',
    "const cx = classNames.bind(styles);",
    'export function Card({ enabled, tone }: { enabled: boolean; tone: "warm" | "cool" }) {',
    '  let size = "card";',
    "  if (enabled) {",
    '    size = "card--active";',
    "  }",
    '  return <div className={clsx(cx("card", `tone-${tone}`, size), styles.icon)} />;',
    "}",
    "",
  ].join("\n");
  const sourceFile = ts.createSourceFile(
    "/fake/ws/src/Card.tsx",
    source,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TSX,
  );
  const sourceBinder = buildSourceBinder(sourceFile);
  const pluginAnalysis = cssModulesClassnamesBinderPluginV0.analyzeSource({
    sourceFile,
    filePath: "/fake/ws/src/Card.tsx",
    sourceBinder,
    fileExists: () => true,
    aliasResolver,
  });
  const sourceDocument = buildSourceDocument({
    filePath: "/fake/ws/src/Card.tsx",
    cxBindings: pluginAnalysis.cxBindings,
    stylesBindings: pluginAnalysis.stylesBindings,
    classUtilNames: pluginAnalysis.classUtilNames,
    sourceBinder,
    classExpressions: pluginAnalysis.classExpressions,
    domainClassReferences: pluginAnalysis.domainClassReferences,
  });
  const sourceBindingGraph = buildSourceBindingGraph(sourceDocument, sourceBinder);
  return captureTsSourceFrontendFactsV0({
    sourceFile,
    sourceBinder,
    sourceDocument,
    sourceBindingGraph,
    cfg: {
      variableName: "size",
      referenceRange: rangeForToken(sourceFile, "size"),
    },
  });
}

function rangeForToken(sourceFile: ts.SourceFile, token: string) {
  const start = sourceFile.text.lastIndexOf(token);
  if (start === -1) throw new Error(`missing token ${token}`);
  const end = start + token.length;
  return {
    start: sourceFile.getLineAndCharacterOfPosition(start),
    end: sourceFile.getLineAndCharacterOfPosition(end),
  };
}
