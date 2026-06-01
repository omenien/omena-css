import { describe, expect, it } from "vitest";
import { DiagnosticSeverity, DiagnosticTag } from "vscode-languageserver-protocol/node";
import { AliasResolver } from "../../../server/engine-core-ts/src/core/cx/alias-resolver";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import { WorkspaceStyleDependencyGraph } from "../../../server/engine-core-ts/src/core/semantic/style-dependency-graph";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import { computeScssUnusedDiagnostics } from "../../../server/lsp-server/src/providers/scss-diagnostics";
import type { StyleSemanticGraphSummaryV0 } from "../../../server/engine-host-node/src/style-semantic-graph-query-backend";
import { workspace } from "../../../packages/vitest-cme/src";
import { infoAtLine as info, makeBaseDeps, semanticSiteAt } from "../../_fixtures/test-helpers";
import {
  buildStyleDocumentFromSelectorMap,
  expandSelectorMapWithTransform,
  makeTestSelector,
  parseStyleSelectorMap,
} from "../../_fixtures/style-documents";

const SCSS_PATH = "/fake/Button.module.scss";
const SOURCE_URI = "file:///a.tsx";
const SOURCE_PATH = "/a.tsx";
const SOURCE_WORKSPACE = workspace({
  [SOURCE_PATH]: "const value = cx(/*<dynamic>*/token/*</dynamic>*/);\n",
});
const DYNAMIC_REF_RANGE = SOURCE_WORKSPACE.range("dynamic", SOURCE_PATH).range;
const COMPOSES_WORKSPACE = workspace({
  [SCSS_PATH]: `.button {
  composes: /*<class>*/base/*</class>*/ from "./Base.module.scss";
}
`,
});
const COMPOSED_CLASS_TOKEN_RANGE = COMPOSES_WORKSPACE.range("class", SCSS_PATH).range;

function styleDocument(selectors: ReadonlyMap<string, ReturnType<typeof info>>) {
  return buildStyleDocumentFromSelectorMap(SCSS_PATH, selectors);
}

function unresolvedDynamicRef(expressionKind: "symbolRef" | "template", refId: string) {
  return {
    refId,
    uri: SOURCE_URI,
    filePath: SOURCE_PATH,
    range: DYNAMIC_REF_RANGE,
    origin: "cxCall" as const,
    scssModulePath: SCSS_PATH,
    expressionKind,
    hasResolvedTargets: false,
    isDynamic: true,
  };
}

function resolvedDynamicRef(expressionKind: "symbolRef" | "template", refId: string) {
  return {
    ...unresolvedDynamicRef(expressionKind, refId),
    hasResolvedTargets: true,
  };
}

function dynamicSemanticSite(
  className: string,
  options: Parameters<typeof semanticSiteAt>[5] = {},
  canonicalName = className,
) {
  return semanticSiteAt(
    SOURCE_URI,
    className,
    DYNAMIC_REF_RANGE.start.line,
    SCSS_PATH,
    canonicalName,
    {
      start: DYNAMIC_REF_RANGE.start.character,
      end: DYNAMIC_REF_RANGE.end.character,
      ...options,
    },
  );
}

function composedClassToken() {
  return {
    className: "base",
    range: COMPOSED_CLASS_TOKEN_RANGE,
  };
}

describe("computeScssUnusedDiagnostics", () => {
  it("flags a selector with zero references as Unnecessary", () => {
    const classMap = new Map([
      ["indicator", info("indicator", 1)],
      ["active", info("active", 3)],
    ]);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///a.tsx", [
      semanticSiteAt("file:///a.tsx", "indicator", 5, SCSS_PATH),
    ]);
    // "active" has no references.
    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      semanticReferenceIndex,
    );
    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0]!.message).toContain("'.active'");
    expect(diagnostics[0]!.severity).toBe(DiagnosticSeverity.Hint);
    expect(diagnostics[0]!.tags).toContain(DiagnosticTag.Unnecessary);
    expect(diagnostics[0]!.source).toBe("css-module-explainer");
  });

  it("returns [] when all selectors have references", () => {
    const classMap = new Map([
      ["indicator", info("indicator", 1)],
      ["active", info("active", 3)],
    ]);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///a.tsx", [
      semanticSiteAt("file:///a.tsx", "indicator", 5, SCSS_PATH),
      semanticSiteAt("file:///a.tsx", "active", 7, SCSS_PATH),
    ]);
    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      semanticReferenceIndex,
    );
    expect(diagnostics).toEqual([]);
  });

  it("uses rust style semantic graph references through the provider runtime deps", () => {
    const classMap = new Map([
      ["indicator", info("indicator", 1)],
      ["active", info("active", 3)],
    ]);
    const styleDoc = styleDocument(classMap);
    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
    });
    const styleSemanticGraphCache = new Map([[SCSS_PATH, makeReferenceGraph(SCSS_PATH)]]);

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      deps.semanticReferenceIndex,
      deps.styleDependencyGraph,
      deps.styleDocumentForPath,
      {
        analysisCache: deps.analysisCache,
        readStyleFile: deps.readStyleFile,
        typeResolver: deps.typeResolver,
        workspaceRoot: deps.workspaceRoot,
        settings: deps.settings,
        aliasResolver: deps.aliasResolver,
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        styleSemanticGraphCache,
      },
    );

    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0]!.message).toContain("'.active'");
    expect(diagnostics[0]!.severity).toBe(DiagnosticSeverity.Hint);
  });

  it("surfaces query-owned style diagnostics with severity and provenance data", async () => {
    const styleSource = `.button { color: red; color: blue; }\n`;
    const sourceDocuments = [
      {
        sourcePath: SOURCE_PATH,
        sourceSource: 'import styles from "./Button.module.scss";\nstyles.button;\n',
      },
    ];
    const styleDoc = parseStyleDocument(styleSource, SCSS_PATH);
    let forwardedInput: unknown = null;

    const diagnostics = await computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      undefined,
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        styleSource,
        sourceDocuments,
        runRustSelectedQueryBackendJsonAsync: async <T>(command: string, input: unknown) => {
          expect(command).toBe("style-diagnostics-for-file");
          forwardedInput = input;
          return {
            product: "omena-query.diagnostics-for-file",
            fileKind: "style",
            diagnostics: [
              {
                code: "unreachableDeclaration",
                severity: "hint",
                provenance: ["omena-checker.cascade-rules", "omena-query.cascade-checker"],
                range: {
                  start: { line: 0, character: 23 },
                  end: { line: 0, character: 34 },
                },
                message: "Declaration is unreachable because a later declaration wins.",
                tags: [DiagnosticTag.Unnecessary],
              },
            ],
          } as T;
        },
      },
    );

    expect(forwardedInput).toMatchObject({
      targetStylePath: SCSS_PATH,
      styles: [{ stylePath: SCSS_PATH, styleSource }],
      sourceDocuments,
    });
    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0]).toMatchObject({
      code: "unreachableDeclaration",
      severity: DiagnosticSeverity.Hint,
      tags: [DiagnosticTag.Unnecessary],
      data: {
        querySeverity: "hint",
        provenance: ["omena-checker.cascade-rules", "omena-query.cascade-checker"],
      },
    });
  });

  it("forwards external SIFs and mode to the engine wire when supplied", async () => {
    const styleSource = `@use "design-system" as ds;\n.button { color: ds.$brand; }\n`;
    const styleDoc = parseStyleDocument(styleSource, SCSS_PATH);
    const externalSifs = [
      {
        canonicalUrl: "pkg:design-system/_tokens.scss",
        sif: {
          sifVersion: "1",
          canonicalUrl: "pkg:design-system/_tokens.scss",
          exports: { variables: [{ name: "brand", defaulted: false, valueRepr: "#0af" }] },
        },
      },
    ];
    let forwardedInput: Record<string, unknown> | null = null;

    await computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      undefined,
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        styleSource,
        externalMode: "sif",
        externalSifs,
        runRustSelectedQueryBackendJsonAsync: async <T>(_command: string, input: unknown) => {
          forwardedInput = input as Record<string, unknown>;
          return {
            product: "omena-query.diagnostics-for-file",
            fileKind: "style",
            diagnostics: [],
          } as T;
        },
      },
    );

    expect(forwardedInput).toMatchObject({
      targetStylePath: SCSS_PATH,
      externalMode: "sif",
      externalSifs,
    });
  });

  it("over-correction: omits externalMode/externalSifs when no SIFs are supplied", async () => {
    const styleSource = `.button { color: red; }\n`;
    const styleDoc = parseStyleDocument(styleSource, SCSS_PATH);
    let forwardedInput: Record<string, unknown> | null = null;

    await computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      undefined,
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        styleSource,
        runRustSelectedQueryBackendJsonAsync: async <T>(_command: string, input: unknown) => {
          forwardedInput = input as Record<string, unknown>;
          return {
            product: "omena-query.diagnostics-for-file",
            fileKind: "style",
            diagnostics: [],
          } as T;
        },
      },
    );

    expect(forwardedInput).not.toBeNull();
    expect(forwardedInput!).not.toHaveProperty("externalMode");
    expect(forwardedInput!).not.toHaveProperty("externalSifs");
  });

  it("keeps query-owned same-file Sass symbol diagnostics in the selected-query path", async () => {
    const styleSource = `.button { color: $missing; }\n`;
    const styleDoc = parseStyleDocument(styleSource, SCSS_PATH);

    const diagnostics = await computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      undefined,
      {
        env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
        styleSource,
        runRustSelectedQueryBackendJsonAsync: async <T>() =>
          ({
            product: "omena-query.diagnostics-for-file",
            fileKind: "style",
            diagnostics: [
              {
                code: "missingSassSymbol",
                severity: "warning",
                provenance: ["omena-parser.sass-symbol-facts", "omena-query.style-diagnostics"],
                range: {
                  start: { line: 0, character: 17 },
                  end: { line: 0, character: 25 },
                },
                message: "Sass variable '$missing' not found in this file.",
                tags: [],
              },
            ],
          }) as T,
      },
    );

    expect(diagnostics).toEqual([
      expect.objectContaining({
        code: "missingSassSymbol",
        message: "Sass variable '$missing' not found in this file.",
        data: expect.objectContaining({
          createSassSymbol: expect.objectContaining({
            newText: "$missing: ;\n\n",
          }),
        }),
      }),
    ]);
  });

  it("suppresses all diagnostics when an unresolvable variable targets the module", () => {
    const classMap = new Map([
      ["indicator", info("indicator", 1)],
      ["active", info("active", 3)],
    ]);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record(
      SOURCE_URI,
      [],
      [unresolvedDynamicRef("symbolRef", "ref:variable")],
    );
    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      semanticReferenceIndex,
    );
    expect(diagnostics).toEqual([]);
  });

  it("suppresses all diagnostics when a template call targets the module", () => {
    const classMap = new Map([
      ["btn-primary", info("btn-primary", 1)],
      ["btn-secondary", info("btn-secondary", 3)],
    ]);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record(
      SOURCE_URI,
      [],
      [unresolvedDynamicRef("template", "ref:template")],
    );
    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      semanticReferenceIndex,
    );
    expect(diagnostics).toEqual([]);
  });

  it("does not suppress diagnostics when dynamic refs were resolved", () => {
    const classMap = new Map([
      ["btn-primary", info("btn-primary", 1)],
      ["btn-secondary", info("btn-secondary", 3)],
      ["btn-tertiary", info("btn-tertiary", 5)],
    ]);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record(
      SOURCE_URI,
      [
        dynamicSemanticSite("btn-primary", {
          certainty: "inferred",
          reason: "flowBranch",
        }),
      ],
      [resolvedDynamicRef("symbolRef", "ref:branch")],
    );
    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      semanticReferenceIndex,
    );
    expect(diagnostics).toHaveLength(2);
    expect(diagnostics.some((diagnostic) => diagnostic.message.includes("btn-secondary"))).toBe(
      true,
    );
    expect(diagnostics.some((diagnostic) => diagnostic.message.includes("btn-tertiary"))).toBe(
      true,
    );
  });

  it("does not flag a class that is composed by another class in the same file", () => {
    const classMap = new Map([
      ["base", info("base", 1, "font-size: 14px")],
      ["button", { ...info("button", 5, "color: red"), composes: [{ classNames: ["base"] }] }],
    ]);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    semanticReferenceIndex.record("file:///a.tsx", [
      semanticSiteAt("file:///a.tsx", "button", 10, SCSS_PATH),
    ]);
    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      semanticReferenceIndex,
      styleDependencyGraph,
    );
    expect(diagnostics).toEqual([]);
  });

  it("does not exempt a same-file composed class when the composing selector is also unused", () => {
    const classMap = new Map([
      ["base", info("base", 1, "font-size: 14px")],
      ["button", { ...info("button", 5, "color: red"), composes: [{ classNames: ["base"] }] }],
    ]);
    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
    );
    expect(diagnostics).toHaveLength(2);
    expect(diagnostics.some((diagnostic) => diagnostic.message.includes("'.base'"))).toBe(true);
    expect(diagnostics.some((diagnostic) => diagnostic.message.includes("'.button'"))).toBe(true);
  });

  it("returns [] for an empty classMap", () => {
    const classMap = new Map();
    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      new WorkspaceSemanticWorkspaceReferenceIndex(),
    );
    expect(diagnostics).toEqual([]);
  });

  it("does exempt a class composed via cross-file composes when the composing selector is used", () => {
    const classMap = new Map([["base", info("base", 1, "font-size: 14px")]]);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(
      "/fake/other.module.css",
      buildStyleDocumentFromSelectorMap(
        "/fake/other.module.css",
        new Map([
          [
            "button",
            {
              ...info("button", 5, "color: red"),
              composes: [{ classNames: ["base"], from: "./Button.module.scss" }],
            },
          ],
        ]),
      ),
    );
    semanticReferenceIndex.record("file:///a.tsx", [
      semanticSiteAt("file:///a.tsx", "button", 10, "/fake/other.module.css"),
    ]);
    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      semanticReferenceIndex,
      styleDependencyGraph,
    );
    expect(diagnostics).toEqual([]);
  });

  it("classnameTransform: does not double-count unused alias entries", async () => {
    // `.btn-primary` is used via its original name; `.orphan` is unused.
    const base = parseStyleSelectorMap(
      `.btn-primary { color: red; }\n.orphan { color: blue; }`,
      SCSS_PATH,
    );
    const classMap = expandSelectorMapWithTransform(base, "camelCase");
    // Sanity: alias exists for btn-primary; `.orphan` is already a
    // valid JS identifier so the transform yields no distinct alias.
    expect(classMap.has("btn-primary")).toBe(true);
    expect(classMap.has("btnPrimary")).toBe(true);

    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///a.tsx", [
      semanticSiteAt("file:///a.tsx", "btn-primary", 10, SCSS_PATH),
    ]);
    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      semanticReferenceIndex,
    );
    // Exactly one warning — for `.orphan` — not a second copy for
    // any alias entry.
    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0]!.message).toContain("'.orphan'");
  });

  it("classnameTransform: original-is-unused still emits exactly one warning", async () => {
    // Consumer references only via the CAMEL alias — the original
    // `btn-primary` has zero direct refs, but we still want exactly
    // one warning, not zero (via alias double-count) and not two.
    const base = parseStyleSelectorMap(`.btn-primary { color: red; }`, SCSS_PATH);
    const classMap = expandSelectorMapWithTransform(base, "camelCase");

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      new WorkspaceSemanticWorkspaceReferenceIndex(),
    );
    expect(diagnostics.filter((d) => d.message.includes("btn-primary"))).toHaveLength(1);
  });

  it("classnameTransform: alias-form TSX access keeps the original marked as used", async () => {
    // `.btn-primary` is accessed ONLY via the camelCase alias from
    // a TSX file (`styles.btnPrimary`) — no canonical `cx('btn-primary')`
    // call exists. The reverse index canonicalises the alias access
    // under `btn-primary`, so the unused-selector check must find the
    // reference and skip the warning.
    const base = parseStyleSelectorMap(`.btn-primary { color: red; }`, SCSS_PATH);
    const classMap = expandSelectorMapWithTransform(base, "camelCase");

    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///a.tsx", [
      semanticSiteAt("file:///a.tsx", "btnPrimary", 5, SCSS_PATH, "btn-primary", {
        reason: "styleAccess",
        origin: "styleAccess",
      }),
    ]);
    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      semanticReferenceIndex,
    );
    expect(diagnostics).toHaveLength(0);
  });

  it("classnameTransform (camelCaseOnly): alias-only class still gets unused detection", async () => {
    // `camelCaseOnly` drops the original from the class map entirely;
    // only `btnPrimary` remains with `originalName: "btn-primary"`.
    // The unused check must still emit exactly one warning for the
    // canonical `btn-primary` even though no entry keyed on that
    // name exists in the map.
    const base = parseStyleSelectorMap(`.btn-primary { color: red; }`, SCSS_PATH);
    const classMap = expandSelectorMapWithTransform(base, "camelCaseOnly");
    expect(classMap.has("btn-primary")).toBe(false);
    expect(classMap.has("btnPrimary")).toBe(true);

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      new WorkspaceSemanticWorkspaceReferenceIndex(),
    );
    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0]!.message).toContain("'.btn-primary'");
  });

  it("semantic direct alias access keeps the canonical selector marked as used", async () => {
    const base = parseStyleSelectorMap(`.btn-primary { color: red; }`, SCSS_PATH);
    const classMap = expandSelectorMapWithTransform(base, "camelCase");

    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///a.tsx", [
      dynamicSemanticSite(
        "btnPrimary",
        {
          origin: "styleAccess",
          reason: "styleAccess",
        },
        "btn-primary",
      ),
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      semanticReferenceIndex,
    );
    expect(diagnostics).toEqual([]);
  });

  it("semantic inferred matches suppress unused diagnostics for resolved dynamic refs", () => {
    const classMap = new Map([
      ["btn-primary", info("btn-primary", 1)],
      ["btn-secondary", info("btn-secondary", 3)],
    ]);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///a.tsx", [
      dynamicSemanticSite("btn-primary", {
        certainty: "inferred",
        reason: "templatePrefix",
      }),
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      semanticReferenceIndex,
    );
    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0]!.message).toContain("'.btn-secondary'");
  });

  it("reports an unresolved cross-file composed module", () => {
    const classMap = new Map([
      [
        "button",
        makeTestSelector("button", 1, {
          declarations: "color: red",
          composes: [
            {
              classNames: ["base"],
              classTokens: [composedClassToken()],
              from: "./Base.module.scss",
            },
          ],
        }),
      ],
    ]);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///a.tsx", [
      semanticSiteAt("file:///a.tsx", "button", 5, SCSS_PATH),
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      semanticReferenceIndex,
      undefined,
      () => null,
    );

    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0]!.severity).toBe(DiagnosticSeverity.Warning);
    expect(diagnostics[0]!.message).toContain(
      "Cannot resolve composed CSS Module './Base.module.scss'.",
    );
    expect(diagnostics[0]!.data).toMatchObject({
      createModuleFile: {
        uri: "file:///fake/Base.module.scss",
      },
    });
  });

  it("reports a missing selector in a composed module", () => {
    const classMap = new Map([
      [
        "button",
        makeTestSelector("button", 1, {
          declarations: "color: red",
          composes: [
            {
              classNames: ["base"],
              classTokens: [composedClassToken()],
              from: "./Base.module.scss",
            },
          ],
        }),
      ],
    ]);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///a.tsx", [
      semanticSiteAt("file:///a.tsx", "button", 5, SCSS_PATH),
    ]);
    const targetDocument = buildStyleDocumentFromSelectorMap(
      "/fake/Base.module.scss",
      new Map([["other", info("other", 1, "color: blue")]]),
    );

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDocument(classMap),
      semanticReferenceIndex,
      undefined,
      (filePath) => (filePath === "/fake/Base.module.scss" ? targetDocument : null),
    );

    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0]!.severity).toBe(DiagnosticSeverity.Warning);
    expect(diagnostics[0]!.message).toContain(
      "Selector '.base' not found in composed module './Base.module.scss'.",
    );
    expect(diagnostics[0]!.data).toMatchObject({
      createSelector: {
        uri: "file:///fake/Base.module.scss",
      },
    });
  });

  it("reports a missing imported @value module with create-file data", () => {
    const styleDoc = parseStyleDocument(
      `@value primary from "./tokens.module.scss";
.button { color: primary; }`,
      SCSS_PATH,
    );

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      () => null,
    );

    const diagnostic = diagnostics.find((entry) =>
      entry.message.includes("Cannot resolve imported @value module './tokens.module.scss'."),
    );
    expect(diagnostic).toBeDefined();
    expect(diagnostic!.data).toMatchObject({
      createModuleFile: {
        uri: "file:///fake/tokens.module.scss",
      },
    });
  });

  it("reports a missing imported @value binding", () => {
    const styleDoc = parseStyleDocument(
      `@value primary, secondary as accent from "./tokens.module.scss";
.button { color: accent; }`,
      SCSS_PATH,
    );

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      (filePath) =>
        filePath === "/fake/tokens.module.scss"
          ? parseStyleDocument(`@value primary: #ff3355;`, filePath)
          : null,
    );

    expect(
      diagnostics.some((entry) =>
        entry.message.includes(
          "@value 'secondary' not found in './tokens.module.scss' for local binding 'accent'.",
        ),
      ),
    ).toBe(true);
  });

  it("reports missing keyframes with create-keyframes data", () => {
    const styleDoc = parseStyleDocument(
      `.button {
  animation: fade 200ms ease-in;
}`,
      SCSS_PATH,
    );

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
    );

    const diagnostic = diagnostics.find((entry) =>
      entry.message.includes("@keyframes 'fade' not found in this file."),
    );
    expect(diagnostic).toBeDefined();
    expect(diagnostic).toMatchObject({
      severity: DiagnosticSeverity.Warning,
      data: {
        createKeyframes: {
          uri: "file:///fake/Button.module.scss",
          newText: "@keyframes fade {\n}\n\n",
        },
      },
    });
  });

  it("reports missing CSS custom property refs when indexed token declarations exist", () => {
    const styleDoc = parseStyleDocument(
      `:root { --brand: #0af; }
.button {
  color: var(--missing);
}`,
      SCSS_PATH,
    );

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
    );

    const diagnostic = diagnostics.find((entry) =>
      entry.message.includes("CSS custom property '--missing' not found in indexed style tokens."),
    );
    expect(diagnostic).toBeDefined();
    expect(diagnostic).toMatchObject({
      severity: DiagnosticSeverity.Warning,
      source: "css-module-explainer",
      data: {
        createCustomProperty: {
          uri: "file:///fake/Button.module.scss",
          newText: "\n\n:root {\n  --missing: ;\n}\n",
          propertyName: "--missing",
        },
      },
    });
  });

  it("does not report imported package CSS custom property refs as missing", () => {
    const stylePath = "/fake/src/Button.module.scss";
    const packagePath = "/fake/node_modules/@design/tokens/variables.css";
    const styleDoc = parseStyleDocument(
      `@use "@design/tokens/variables.css";

.title {
  color: var(--color-gray-700);
}`,
      stylePath,
    );
    const packageDoc = parseStyleDocument(`:root { --color-gray-700: #767678; }`, packagePath);
    const byPath = new Map([
      [stylePath, styleDoc],
      [packagePath, packageDoc],
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      stylePath,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      (filePath) => byPath.get(filePath) ?? null,
    );

    expect(diagnostics.filter((entry) => entry.message.includes("CSS custom property"))).toEqual(
      [],
    );
  });

  it("does not report package-root CSS custom property refs through package.json style entries", () => {
    const stylePath = "/fake/src/Button.module.scss";
    const packageRoot = "/fake/node_modules/@design/tokens";
    const packageJsonPath = `${packageRoot}/package.json`;
    const packagePath = `${packageRoot}/variables.css`;
    const styleDoc = parseStyleDocument(
      `@use "@design/tokens";

.title {
  color: var(--color-gray-700);
}`,
      stylePath,
    );
    const packageDoc = parseStyleDocument(`:root { --color-gray-700: #767678; }`, packagePath);
    const byPath = new Map([
      [stylePath, styleDoc],
      [packagePath, packageDoc],
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      stylePath,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      (filePath) => byPath.get(filePath) ?? null,
      {
        readStyleFile: (filePath) =>
          filePath === packageJsonPath ? `{"style":"variables.css"}` : null,
      },
    );

    expect(diagnostics.filter((entry) => entry.message.includes("CSS custom property"))).toEqual(
      [],
    );
  });

  it("does not report CSS custom property refs forwarded from a package root through local utilities", () => {
    const stylePath = "/fake/src/Button.module.scss";
    const utilsPath = "/fake/src/_utils.scss";
    const packageRoot = "/fake/node_modules/@design/tokens";
    const packageJsonPath = `${packageRoot}/package.json`;
    const packagePath = `${packageRoot}/variables.css`;
    const styleDoc = parseStyleDocument(
      `@use "utils" as *;

.title {
  color: var(--color-gray-700);
}`,
      stylePath,
    );
    const utilsDoc = parseStyleDocument(`@forward "@design/tokens" as ds_*;`, utilsPath);
    const packageDoc = parseStyleDocument(`:root { --color-gray-700: #767678; }`, packagePath);
    const byPath = new Map([
      [stylePath, styleDoc],
      [utilsPath, utilsDoc],
      [packagePath, packageDoc],
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      stylePath,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      (filePath) => byPath.get(filePath) ?? null,
      {
        readStyleFile: (filePath) =>
          filePath === packageJsonPath ? `{"style":"variables.css"}` : null,
      },
    );

    expect(diagnostics.filter((entry) => entry.message.includes("CSS custom property"))).toEqual(
      [],
    );
  });

  it("reports CSS custom property refs when only unmatched workspace theme declarations exist", () => {
    const stylePath = "/fake/src/Button.module.scss";
    const themePath = "/fake/src/theme.module.scss";
    const styleDoc = parseStyleDocument(
      `.button {
  color: var(--brand);
}`,
      stylePath,
    );
    const themeDoc = parseStyleDocument(`.theme { --brand: #222; }`, themePath);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(themePath, themeDoc);

    const diagnostics = computeScssUnusedDiagnostics(
      stylePath,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      styleDependencyGraph,
      (filePath) => (filePath === themePath ? themeDoc : null),
    );

    expect(
      diagnostics.filter((entry) =>
        entry.message.includes("CSS custom property '--brand' not found in indexed style tokens."),
      ),
    ).toHaveLength(1);
  });

  it("reports missing Sass symbols with create-symbol data", () => {
    const styleDoc = parseStyleDocument(
      `.button {
  color: $missing;
  @include absent();
}`,
      SCSS_PATH,
    );

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
    );

    const variableDiagnostic = diagnostics.find((entry) =>
      entry.message.includes("Sass variable '$missing' not found in this file."),
    );
    expect(variableDiagnostic).toBeDefined();
    expect(variableDiagnostic).toMatchObject({
      severity: DiagnosticSeverity.Warning,
      data: {
        createSassSymbol: {
          uri: "file:///fake/Button.module.scss",
          newText: "$missing: ;\n\n",
        },
      },
    });

    const mixinDiagnostic = diagnostics.find((entry) =>
      entry.message.includes("Sass mixin '@mixin absent' not found in this file."),
    );
    expect(mixinDiagnostic).toBeDefined();
    expect(mixinDiagnostic).toMatchObject({
      severity: DiagnosticSeverity.Warning,
      data: {
        createSassSymbol: {
          uri: "file:///fake/Button.module.scss",
          newText: "@mixin absent() {\n}\n\n",
        },
      },
    });
  });

  it("reports missing mixins consistently inside nested pseudo rules", () => {
    const styleDoc = parseStyleDocument(
      `.icon {
  &::before {
    @include position(absolute, all);
  }
  .dot {
    @include position(absolute, (top -1px, left -1px));
  }
}
.image {
  &::after {
    @include position(absolute, all);
  }
}
.link-detail {
  @include position(absolute, all);
}
`,
      SCSS_PATH,
    );

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
    );

    expect(
      diagnostics.filter((entry) =>
        entry.message.includes("Sass mixin '@mixin position' not found in this file."),
      ),
    ).toHaveLength(4);
  });

  it("reports missing Less variables with create-symbol data", () => {
    const lessPath = "/fake/Button.module.less";
    const styleDoc = parseStyleDocument(
      `.button {
  color: @missing;
}`,
      lessPath,
    );

    const diagnostics = computeScssUnusedDiagnostics(
      lessPath,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
    );

    const variableDiagnostic = diagnostics.find((entry) =>
      entry.message.includes("Less variable '@missing' not found in this file."),
    );
    expect(variableDiagnostic).toBeDefined();
    expect(variableDiagnostic).toMatchObject({
      severity: DiagnosticSeverity.Warning,
      data: {
        createSassSymbol: {
          uri: "file:///fake/Button.module.less",
          newText: "@missing: ;\n\n",
        },
      },
    });
  });

  it("does not report Sass symbols resolved through wildcard module imports", () => {
    const tokensPath = "/fake/tokens.module.scss";
    const styleDoc = parseStyleDocument(
      `@use "./tokens.module" as *;

.button {
  color: $gap;
  @include raised();
  border-color: tone($gap);
}`,
      SCSS_PATH,
    );
    const targetDoc = parseStyleDocument(
      `$gap: 1rem;
@mixin raised() {}
@function tone($value) { @return $value; }`,
      tokensPath,
    );
    const byPath = new Map([
      [SCSS_PATH, styleDoc],
      [tokensPath, targetDoc],
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      (filePath) => byPath.get(filePath) ?? null,
    );

    expect(diagnostics.filter((entry) => entry.message.includes("Sass "))).toEqual([]);
  });

  it("does not report Sass symbols resolved through legacy @import", () => {
    const tokensPath = "/fake/tokens.module.scss";
    const styleDoc = parseStyleDocument(
      `@import "./tokens.module";

.button {
  color: $gap;
  @include raised();
}`,
      SCSS_PATH,
    );
    const targetDoc = parseStyleDocument(
      `$gap: 1rem;
@mixin raised() {}`,
      tokensPath,
    );
    const byPath = new Map([
      [SCSS_PATH, styleDoc],
      [tokensPath, targetDoc],
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      (filePath) => byPath.get(filePath) ?? null,
    );

    expect(diagnostics.filter((entry) => entry.message.includes("Sass "))).toEqual([]);
  });

  it("does not report Sass symbols resolved through package imports", () => {
    const packagePath = "/fake/node_modules/@design/foundation/scss/_utils.scss";
    const styleDoc = parseStyleDocument(
      `@use "@design/foundation/scss/utils" as *;

.button {
  color: $gray-900;
  @include typo-18;
}`,
      SCSS_PATH,
    );
    const targetDoc = parseStyleDocument(
      `$gray-900: #111;
@mixin typo-18 {}`,
      packagePath,
    );
    const byPath = new Map([
      [SCSS_PATH, styleDoc],
      [packagePath, targetDoc],
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      (filePath) => byPath.get(filePath) ?? null,
    );

    expect(diagnostics.filter((entry) => entry.message.includes("Sass "))).toEqual([]);
  });

  it("does not report Sass symbols resolved through package.json sass entries", () => {
    const packageRoot = "/fake/node_modules/@design/tokens";
    const packageJsonPath = `${packageRoot}/package.json`;
    const packagePath = `${packageRoot}/src/index.scss`;
    const styleDoc = parseStyleDocument(
      `@use "@design/tokens" as *;

.button {
  color: $gray700;
  @include typography16;
}`,
      SCSS_PATH,
    );
    const targetDoc = parseStyleDocument(
      `$gray700: #767678;
@mixin typography16 {}`,
      packagePath,
    );
    const byPath = new Map([
      [SCSS_PATH, styleDoc],
      [packagePath, targetDoc],
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      (filePath) => byPath.get(filePath) ?? null,
      {
        readStyleFile: (filePath) =>
          filePath === packageJsonPath ? `{"sass":"src/index.scss"}` : null,
      },
    );

    expect(diagnostics.filter((entry) => entry.message.includes("Sass "))).toEqual([]);
  });

  it("does not report Sass symbols resolved through aliased forwarded package imports", () => {
    const themePath = "/fake/src/shared/theme.module.scss";
    const packagePath = "/fake/node_modules/@design/foundation/scss/_utils.scss";
    const styleDoc = parseStyleDocument(
      `@use "$shared/theme.module" as *;

.button {
  color: $gray-900;
  @include typo-18;
}`,
      SCSS_PATH,
    );
    const themeDoc = parseStyleDocument(`@forward "@design/foundation/scss/utils";`, themePath);
    const targetDoc = parseStyleDocument(
      `$gray-900: #111;
@mixin typo-18 {}`,
      packagePath,
    );
    const byPath = new Map([
      [SCSS_PATH, styleDoc],
      [themePath, themeDoc],
      [packagePath, targetDoc],
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      (filePath) => byPath.get(filePath) ?? null,
      {
        aliasResolver: new AliasResolver("/fake", { $shared: "src/shared" }),
      },
    );

    expect(diagnostics.filter((entry) => entry.message.includes("Sass "))).toEqual([]);
  });

  it("does not report Sass symbols resolved through prefixed package forwards from a local utility module", () => {
    const stylePath = "/fake/src/Button.module.scss";
    const utilsPath = "/fake/src/_utils.scss";
    const colorsPath = "/fake/node_modules/@design/tokens/_colors.scss";
    const typographyPath = "/fake/node_modules/@design/tokens/_typography.scss";
    const styleDoc = parseStyleDocument(
      `@use "utils" as *;

.title {
  color: $ds_gray700;
  @include ds_typography16;
}`,
      stylePath,
    );
    const utilsDoc = parseStyleDocument(
      `@forward "@design/tokens/colors" as ds_*;
@forward "@design/tokens/typography" as ds_*;`,
      utilsPath,
    );
    const colorsDoc = parseStyleDocument(`$gray700: #767678;`, colorsPath);
    const typographyDoc = parseStyleDocument(`@mixin typography16 {}`, typographyPath);
    const byPath = new Map([
      [stylePath, styleDoc],
      [utilsPath, utilsDoc],
      [colorsPath, colorsDoc],
      [typographyPath, typographyDoc],
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      stylePath,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      (filePath) => byPath.get(filePath) ?? null,
    );

    expect(diagnostics.filter((entry) => entry.message.includes("Sass "))).toEqual([]);
  });

  it("does not report Sass symbols resolved through forwarded wildcard imports", () => {
    const themePath = "/fake/theme.module.scss";
    const tokensPath = "/fake/tokens.module.scss";
    const styleDoc = parseStyleDocument(
      `@use "./theme.module" as *;

.button {
  color: $gap;
  @include raised();
  border-color: tone($gap);
}`,
      SCSS_PATH,
    );
    const themeDoc = parseStyleDocument(`@forward "./tokens.module";`, themePath);
    const targetDoc = parseStyleDocument(
      `$gap: 1rem;
@mixin raised() {}
@function tone($value) { @return $value; }`,
      tokensPath,
    );
    const byPath = new Map([
      [SCSS_PATH, styleDoc],
      [themePath, themeDoc],
      [tokensPath, targetDoc],
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      (filePath) => byPath.get(filePath) ?? null,
    );

    expect(diagnostics.filter((entry) => entry.message.includes("Sass "))).toEqual([]);
  });

  it("honors prefixed forward hide policies for wildcard Sass diagnostics", () => {
    const themePath = "/fake/theme.module.scss";
    const tokensPath = "/fake/tokens.module.scss";
    const styleDoc = parseStyleDocument(
      `@use "./theme.module" as *;

.button {
  color: $theme-gap;
  border-color: theme-tone($theme-gap);
  background: theme-hidden($theme-gap);
  margin: $theme-secret;
}`,
      SCSS_PATH,
    );
    const themeDoc = parseStyleDocument(
      `@forward "./tokens.module" as theme-* hide $secret, hidden;`,
      themePath,
    );
    const targetDoc = parseStyleDocument(
      `$gap: 1rem;
$secret: 2rem;
@function tone($value) { @return $value; }
@function hidden($value) { @return $value; }`,
      tokensPath,
    );
    const byPath = new Map([
      [SCSS_PATH, styleDoc],
      [themePath, themeDoc],
      [tokensPath, targetDoc],
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      (filePath) => byPath.get(filePath) ?? null,
    );

    const sassMessages = diagnostics
      .map((entry) => entry.message)
      .filter((message) => message.includes("Sass "));
    expect(sassMessages).toEqual([
      "Sass function 'theme-hidden()' not found in this file.",
      "Sass variable '$theme-secret' not found in this file.",
    ]);
  });

  it("keeps reporting wildcard Sass symbols that are not exported by the module", () => {
    const tokensPath = "/fake/tokens.module.scss";
    const styleDoc = parseStyleDocument(
      `@use "./tokens.module" as *;

.button {
  color: $missing;
}`,
      SCSS_PATH,
    );
    const targetDoc = parseStyleDocument(`$gap: 1rem;`, tokensPath);
    const byPath = new Map([
      [SCSS_PATH, styleDoc],
      [tokensPath, targetDoc],
    ]);

    const diagnostics = computeScssUnusedDiagnostics(
      SCSS_PATH,
      styleDoc,
      new WorkspaceSemanticWorkspaceReferenceIndex(),
      new WorkspaceStyleDependencyGraph(),
      (filePath) => byPath.get(filePath) ?? null,
    );

    expect(
      diagnostics.find((entry) =>
        entry.message.includes("Sass variable '$missing' not found in this file."),
      ),
    ).toBeDefined();
  });
});

function makeReferenceGraph(stylePath: string): StyleSemanticGraphSummaryV0 {
  return {
    schemaVersion: "0",
    product: "omena-semantic.style-semantic-graph",
    language: "scss",
    parserFacts: {},
    semanticFacts: {},
    selectorIdentityEngine: {
      schemaVersion: "0",
      product: "omena-semantic.selector-identity",
      canonicalIdCount: 2,
      canonicalIds: [
        {
          canonicalId: "selector:indicator",
          localName: "indicator",
          identityKind: "localClass",
          rewriteSafety: "safe",
          blockers: [],
        },
        {
          canonicalId: "selector:active",
          localName: "active",
          identityKind: "localClass",
          rewriteSafety: "safe",
          blockers: [],
        },
      ],
      rewriteSafety: {
        allCanonicalIdsRewriteSafe: true,
        safeCanonicalIds: ["selector:indicator", "selector:active"],
        blockedCanonicalIds: [],
        blockers: [],
      },
    },
    selectorReferenceEngine: {
      schemaVersion: "0",
      product: "omena-semantic.selector-references",
      stylePath,
      selectorCount: 2,
      referencedSelectorCount: 1,
      unreferencedSelectorCount: 1,
      totalReferenceSites: 1,
      selectors: [
        makeSelectorReferenceSummary(stylePath, "indicator", true),
        makeSelectorReferenceSummary(stylePath, "active", false),
      ],
    },
    sourceInputEvidence: {},
    promotionEvidence: {},
    losslessCstContract: {},
  };
}

function makeSelectorReferenceSummary(
  stylePath: string,
  localName: string,
  hasAnyReferences: boolean,
) {
  const referenceCount = hasAnyReferences ? 1 : 0;
  return {
    canonicalId: `selector:${localName}`,
    filePath: stylePath,
    localName,
    totalReferences: referenceCount,
    directReferenceCount: referenceCount,
    editableDirectReferenceCount: referenceCount,
    exactReferenceCount: referenceCount,
    inferredOrBetterReferenceCount: referenceCount,
    hasExpandedReferences: false,
    hasStyleDependencyReferences: false,
    hasAnyReferences,
    sites: hasAnyReferences
      ? [
          {
            filePath: "/fake/App.tsx",
            range: {
              start: { line: 8, character: 10 },
              end: { line: 8, character: 19 },
            },
            expansion: "direct",
            referenceKind: "source",
          },
        ]
      : [],
    editableDirectSites: hasAnyReferences
      ? [
          {
            filePath: "/fake/App.tsx",
            range: {
              start: { line: 8, character: 10 },
              end: { line: 8, character: 19 },
            },
            className: localName,
          },
        ]
      : [],
  };
}
