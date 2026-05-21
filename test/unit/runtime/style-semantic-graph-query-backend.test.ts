import { describe, expect, it } from "vitest";
import type { EngineInputV2 } from "../../../server/engine-core-ts/src/contracts";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import { DEFAULT_SETTINGS } from "../../../server/engine-core-ts/src/settings";
import {
  buildStyleSemanticGraphDesignTokenRankedReferenceReadModels,
  buildStyleSemanticGraphSelectorIdentityReadModels,
  resolveRustCssModulesCrossFileResolutionForWorkspaceTarget,
  resolveRustSassModuleCrossFileResolutionForWorkspaceTarget,
  resolveRustStyleSemanticGraph,
  resolveRustStyleSemanticGraphForWorkspaceTarget,
  type StyleSemanticGraphBatchOutputCache,
  type StyleSemanticGraphBatchRunnerInputV0,
  type StyleSemanticGraphCssModulesCrossFileResolutionV0,
  type StyleSemanticGraphSassModuleCrossFileResolutionV0,
  type StyleSemanticGraphSummaryV0,
  type StyleSemanticGraphRunnerInputV0,
} from "../../../server/engine-host-node/src/style-semantic-graph-query-backend";
import { EngineShadowRunnerCancelledError } from "../../../server/engine-host-node/src/selected-query-backend";
import { infoAtLine, makeBaseDeps } from "../../_fixtures/test-helpers";
import { buildStyleDocumentFromSelectorMap } from "../../_fixtures/style-documents";

const SCSS_PATH = "/fake/ws/src/Button.module.scss";
const CARD_SCSS_PATH = "/fake/ws/src/Card.module.scss";
const SCSS_SOURCE = ".button { color: red; }";
const CARD_SCSS_SOURCE = ".card { color: red; }";

describe("style semantic graph query backend", () => {
  it("routes host style semantic graph reads through the selected-query runner", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: (filePath) =>
        filePath === SCSS_PATH ? new Map([["button", infoAtLine("button", 1)]]) : null,
      readStyleFile: (filePath) => (filePath === SCSS_PATH ? SCSS_SOURCE : null),
      workspaceRoot: "/fake/ws",
    });
    let runnerCommand: string | null = null;
    let runnerInput: StyleSemanticGraphRunnerInputV0 | null = null;

    const graph = resolveRustStyleSemanticGraph(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
        sourceDocuments: [],
        styleFiles: [],
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      {
        runRustSelectedQueryBackendJson: <T>(command: string, input: unknown): T => {
          runnerCommand = command;
          runnerInput = input as StyleSemanticGraphRunnerInputV0;
          return makeGraph() as T;
        },
      },
    );

    expect(graph?.product).toBe("omena-semantic.style-semantic-graph");
    expect(runnerCommand).toBe("style-semantic-graph");
    expect(runnerInput).toMatchObject({
      stylePath: SCSS_PATH,
      styleSource: SCSS_SOURCE,
    });
    expect(runnerInput?.engineInput.styles).toHaveLength(1);
    expect(runnerInput?.engineInput.styles[0]?.filePath).toBe(SCSS_PATH);
  });

  it("attaches host HIR ranges to rust selector identity graph entries", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      SCSS_PATH,
      new Map([
        ["button", infoAtLine("button", 2)],
        ["icon", infoAtLine("icon", 5)],
      ]),
    );

    expect(buildStyleSemanticGraphSelectorIdentityReadModels(makeGraph(), styleDocument)).toEqual([
      {
        canonicalId: "selector:button",
        canonicalName: "button",
        identityKind: "localClass",
        rewriteSafety: "safe",
        blockers: [],
        range: { start: { line: 2, character: 1 }, end: { line: 2, character: 7 } },
        ruleRange: { start: { line: 2, character: 0 }, end: { line: 4, character: 1 } },
        viewKind: "canonical",
      },
    ]);
  });

  it("omits rust selector identities that have no current host HIR range", () => {
    const styleDocument = buildStyleDocumentFromSelectorMap(
      SCSS_PATH,
      new Map([["icon", infoAtLine("icon", 5)]]),
    );

    expect(buildStyleSemanticGraphSelectorIdentityReadModels(makeGraph(), styleDocument)).toEqual(
      [],
    );
  });

  it("exposes rust design token ranked references through a host read model", () => {
    expect(buildStyleSemanticGraphDesignTokenRankedReferenceReadModels(makeGraph())).toEqual([
      {
        referenceName: "--brand",
        referenceSourceOrder: 0,
        winnerDeclarationSourceOrder: 2,
        shadowedDeclarationSourceOrders: [0, 1],
        candidateDeclarationCount: 3,
        crossFileCandidateDeclarationCount: 0,
        crossFileShadowedDeclarationCount: 0,
      },
    ]);
  });

  it("attaches host HIR ranges to rust design token ranked references", () => {
    const styleDocument = parseStyleDocument(
      [
        ":root { --brand: red; }",
        ".theme { --brand: blue; }",
        ".button { --brand: green; color: var(--brand); }",
      ].join("\n"),
      SCSS_PATH,
    );

    const [readModel] = buildStyleSemanticGraphDesignTokenRankedReferenceReadModels(
      makeGraph(),
      styleDocument,
    );

    expect(readModel?.reference?.name).toBe("--brand");
    expect(readModel?.winnerDeclaration?.value).toBe("green");
    expect(readModel?.shadowedDeclarations?.map((declaration) => declaration.value)).toEqual([
      "red",
      "blue",
    ]);
  });

  it("does not attach local HIR declarations to external rust design token winners", () => {
    const styleDocument = parseStyleDocument(".button { color: var(--brand); }", SCSS_PATH);
    const graph = makeGraph();
    const externalGraph: StyleSemanticGraphSummaryV0 = {
      ...graph,
      designTokenSemantics: graph.designTokenSemantics
        ? {
            ...graph.designTokenSemantics,
            resolutionScope: "cross-file-import-candidate",
            cascadeRankingSignal: {
              ...graph.designTokenSemantics.cascadeRankingSignal,
              rankedReferences: [
                {
                  referenceName: "--brand",
                  referenceSourceOrder: 0,
                  winnerDeclarationSourceOrder: 0,
                  winnerDeclarationFilePath: "/fake/ws/src/tokens.scss",
                  winnerDeclarationRange: {
                    start: { line: 0, character: 8 },
                    end: { line: 0, character: 15 },
                  },
                  shadowedDeclarationSourceOrders: [],
                  candidateDeclarationCount: 1,
                  crossFileCandidateDeclarationCount: 1,
                  crossFileShadowedDeclarationCount: 0,
                },
              ],
            },
          }
        : undefined,
    };

    const [readModel] = buildStyleSemanticGraphDesignTokenRankedReferenceReadModels(
      externalGraph,
      styleDocument,
    );

    expect(readModel?.reference?.name).toBe("--brand");
    expect(readModel?.winnerDeclarationFilePath).toBe("/fake/ws/src/tokens.scss");
    expect(readModel?.winnerDeclarationRange).toEqual({
      start: { line: 0, character: 8 },
      end: { line: 0, character: 15 },
    });
    expect(readModel?.crossFileCandidateScope).toBe("cross-file-import-candidate");
    expect(readModel?.winnerDeclaration).toBeUndefined();
  });

  it("does not spawn rust when the target style source is unavailable", () => {
    const deps = makeBaseDeps({ readStyleFile: () => null });

    const graph = resolveRustStyleSemanticGraph(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
        sourceDocuments: [],
        styleFiles: [SCSS_PATH],
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      {
        runRustSelectedQueryBackendJson: () => {
          throw new Error("unexpected rust runner call");
        },
      },
    );

    expect(graph).toBeNull();
  });

  it("uses precomputed workspace inputs for workspace target graph reads", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: (filePath) =>
        filePath === SCSS_PATH ? new Map([["button", infoAtLine("button", 1)]]) : null,
      readStyleFile: (filePath) => (filePath === SCSS_PATH ? SCSS_SOURCE : null),
      workspaceRoot: "/fake/ws",
    });
    let runnerInput: StyleSemanticGraphRunnerInputV0 | null = null;

    const graph = resolveRustStyleSemanticGraphForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      {
        sourceDocuments: [
          {
            uri: "file:///fake/ws/src/App.tsx",
            filePath: "/fake/ws/src/App.tsx",
            content: "const app = true;",
            version: 1,
          },
        ],
        styleFiles: [SCSS_PATH],
        runRustSelectedQueryBackendJson: <T>(_command: string, input: unknown): T => {
          runnerInput = input as StyleSemanticGraphRunnerInputV0;
          return makeGraph() as T;
        },
      },
    );

    expect(graph?.product).toBe("omena-semantic.style-semantic-graph");
    expect(runnerInput?.engineInput.sources).toHaveLength(1);
    expect(runnerInput?.engineInput.sources[0]?.filePath).toBe("/fake/ws/src/App.tsx");
    expect(runnerInput?.engineInput.styles.map((style) => style.filePath)).toEqual([SCSS_PATH]);
  });

  it("reuses a precomputed engine input for style semantic graph runner calls", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: (filePath) =>
        filePath === SCSS_PATH ? new Map([["button", infoAtLine("button", 1)]]) : null,
      readStyleFile: (filePath) => (filePath === SCSS_PATH ? SCSS_SOURCE : null),
      workspaceRoot: "/fake/ws",
    });
    const engineInput: EngineInputV2 = {
      version: "2",
      workspace: {
        root: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        settingsKey: "precomputed",
      },
      sources: [],
      styles: [],
      typeFacts: [],
    };
    let runnerInput: StyleSemanticGraphRunnerInputV0 | null = null;

    resolveRustStyleSemanticGraph(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
        sourceDocuments: [],
        styleFiles: [SCSS_PATH],
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      {
        engineInput,
        runRustSelectedQueryBackendJson: <T>(_command: string, input: unknown): T => {
          runnerInput = input as StyleSemanticGraphRunnerInputV0;
          return makeGraph() as T;
        },
      },
    );

    expect(runnerInput?.engineInput).toBe(engineInput);
  });

  it("reuses cached workspace target graph reads for the same style path", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: (filePath) =>
        filePath === SCSS_PATH ? new Map([["button", infoAtLine("button", 1)]]) : null,
      readStyleFile: (filePath) => (filePath === SCSS_PATH ? SCSS_SOURCE : null),
      workspaceRoot: "/fake/ws",
    });
    let runnerCalls = 0;
    const styleSemanticGraphCache = new Map<string, StyleSemanticGraphSummaryV0 | null>();
    const queryOptions = {
      sourceDocuments: [],
      styleFiles: [SCSS_PATH],
      styleSemanticGraphCache,
      runRustSelectedQueryBackendJson: <T>(): T => {
        runnerCalls += 1;
        return makeGraph() as T;
      },
    };

    const first = resolveRustStyleSemanticGraphForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      queryOptions,
    );
    const second = resolveRustStyleSemanticGraphForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      queryOptions,
    );

    expect(first).toBe(second);
    expect(runnerCalls).toBe(1);
  });

  it("seeds workspace target graph cache from a batch runner read", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: (filePath) =>
        filePath === SCSS_PATH
          ? new Map([["button", infoAtLine("button", 1)]])
          : filePath === CARD_SCSS_PATH
            ? new Map([["card", infoAtLine("card", 1)]])
            : null,
      readStyleFile: (filePath) =>
        filePath === SCSS_PATH
          ? SCSS_SOURCE
          : filePath === CARD_SCSS_PATH
            ? CARD_SCSS_SOURCE
            : null,
      workspaceRoot: "/fake/ws",
    });
    const engineInput = makeEngineInput();
    const styleSemanticGraphCache = new Map<string, StyleSemanticGraphSummaryV0 | null>();
    const commands: string[] = [];
    let batchInput: StyleSemanticGraphBatchRunnerInputV0 | null = null;
    const queryOptions = {
      engineInput,
      sourceDocuments: [],
      styleFiles: [SCSS_PATH, CARD_SCSS_PATH],
      styleSemanticGraphCache,
      runRustSelectedQueryBackendJson: <T>(command: string, input: unknown): T => {
        commands.push(command);
        if (command !== "style-semantic-graph-batch") {
          throw new Error(`unexpected runner command: ${command}`);
        }
        batchInput = input as StyleSemanticGraphBatchRunnerInputV0;
        return {
          schemaVersion: "0",
          product: "omena-semantic.style-semantic-graph-batch",
          graphs: [
            { stylePath: SCSS_PATH, graph: makeGraph(SCSS_PATH) },
            { stylePath: CARD_SCSS_PATH, graph: makeGraph(CARD_SCSS_PATH) },
          ],
        } as T;
      },
    };

    const buttonGraph = resolveRustStyleSemanticGraphForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      queryOptions,
    );
    const cardGraph = resolveRustStyleSemanticGraphForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      CARD_SCSS_PATH,
      queryOptions,
    );

    expect(commands).toEqual(["style-semantic-graph-batch"]);
    expect(batchInput?.styles.map((style) => style.stylePath)).toEqual([SCSS_PATH, CARD_SCSS_PATH]);
    expect(buttonGraph?.selectorReferenceEngine.stylePath).toBe(SCSS_PATH);
    expect(cardGraph?.selectorReferenceEngine.stylePath).toBe(CARD_SCSS_PATH);
  });

  it("preserves CSS Modules relation closure from batch graph reads", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: (filePath) =>
        filePath === SCSS_PATH
          ? new Map([["button", infoAtLine("button", 1)]])
          : filePath === CARD_SCSS_PATH
            ? new Map([["card", infoAtLine("card", 1)]])
            : null,
      readStyleFile: (filePath) =>
        filePath === SCSS_PATH
          ? SCSS_SOURCE
          : filePath === CARD_SCSS_PATH
            ? CARD_SCSS_SOURCE
            : null,
      workspaceRoot: "/fake/ws",
    });
    const styleSemanticGraphCache = new Map<string, StyleSemanticGraphSummaryV0 | null>();
    const styleSemanticGraphBatchOutputCache: StyleSemanticGraphBatchOutputCache = new Map();
    const commands: string[] = [];
    const cssModulesResolution = makeCssModulesResolution();
    const queryOptions = {
      sourceDocuments: [],
      styleFiles: [SCSS_PATH, CARD_SCSS_PATH],
      styleSemanticGraphCache,
      styleSemanticGraphBatchOutputCache,
      runRustSelectedQueryBackendJson: <T>(command: string): T => {
        commands.push(command);
        if (command !== "style-semantic-graph-batch") {
          throw new Error(`unexpected runner command: ${command}`);
        }
        return {
          schemaVersion: "0",
          product: "omena-semantic.style-semantic-graph-batch",
          cssModulesResolution,
          graphs: [
            { stylePath: SCSS_PATH, graph: makeGraph(SCSS_PATH) },
            { stylePath: CARD_SCSS_PATH, graph: makeGraph(CARD_SCSS_PATH) },
          ],
        } as T;
      },
    };

    const first = resolveRustCssModulesCrossFileResolutionForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      queryOptions,
    );
    const second = resolveRustCssModulesCrossFileResolutionForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      queryOptions,
    );

    expect(commands).toEqual(["style-semantic-graph-batch"]);
    expect(first?.composesClosureEdgeCount).toBe(1);
    expect(second).toBe(first);
    expect(styleSemanticGraphBatchOutputCache.size).toBe(1);
    expect([...styleSemanticGraphBatchOutputCache.values()][0]?.cssModulesResolution).toBe(
      cssModulesResolution,
    );
    expect(styleSemanticGraphCache.get(SCSS_PATH)?.selectorReferenceEngine.stylePath).toBe(
      SCSS_PATH,
    );
  });

  it("preserves Sass module resolution from batch graph reads", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: (filePath) =>
        filePath === SCSS_PATH
          ? new Map([["button", infoAtLine("button", 1)]])
          : filePath === CARD_SCSS_PATH
            ? new Map([["card", infoAtLine("card", 1)]])
            : null,
      readStyleFile: (filePath) =>
        filePath === SCSS_PATH
          ? SCSS_SOURCE
          : filePath === CARD_SCSS_PATH
            ? CARD_SCSS_SOURCE
            : null,
      workspaceRoot: "/fake/ws",
    });
    const styleSemanticGraphCache = new Map<string, StyleSemanticGraphSummaryV0 | null>();
    const styleSemanticGraphBatchOutputCache: StyleSemanticGraphBatchOutputCache = new Map();
    const commands: string[] = [];
    const sassModuleResolution = makeSassModuleResolution();
    const queryOptions = {
      sourceDocuments: [],
      styleFiles: [SCSS_PATH, CARD_SCSS_PATH],
      styleSemanticGraphCache,
      styleSemanticGraphBatchOutputCache,
      runRustSelectedQueryBackendJson: <T>(command: string): T => {
        commands.push(command);
        if (command !== "style-semantic-graph-batch") {
          throw new Error(`unexpected runner command: ${command}`);
        }
        return {
          schemaVersion: "0",
          product: "omena-semantic.style-semantic-graph-batch",
          sassModuleResolution,
          graphs: [
            { stylePath: SCSS_PATH, graph: makeGraph(SCSS_PATH) },
            { stylePath: CARD_SCSS_PATH, graph: makeGraph(CARD_SCSS_PATH) },
          ],
        } as T;
      },
    };

    const first = resolveRustSassModuleCrossFileResolutionForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      queryOptions,
    );
    const second = resolveRustSassModuleCrossFileResolutionForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      queryOptions,
    );

    expect(commands).toEqual(["style-semantic-graph-batch"]);
    expect(first?.moduleEdgeCount).toBe(1);
    expect(second).toBe(first);
    expect(styleSemanticGraphBatchOutputCache.size).toBe(1);
    expect([...styleSemanticGraphBatchOutputCache.values()][0]?.sassModuleResolution).toBe(
      sassModuleResolution,
    );
    expect(styleSemanticGraphCache.get(SCSS_PATH)?.selectorReferenceEngine.stylePath).toBe(
      SCSS_PATH,
    );
  });

  it("builds one workspace engine input for cached multi-style workspace target reads", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: (filePath) =>
        filePath === SCSS_PATH
          ? new Map([["button", infoAtLine("button", 1)]])
          : filePath === CARD_SCSS_PATH
            ? new Map([["card", infoAtLine("card", 1)]])
            : null,
      readStyleFile: (filePath) =>
        filePath === SCSS_PATH
          ? SCSS_SOURCE
          : filePath === CARD_SCSS_PATH
            ? CARD_SCSS_SOURCE
            : null,
      workspaceRoot: "/fake/ws",
    });
    const styleSemanticGraphCache = new Map<string, StyleSemanticGraphSummaryV0 | null>();
    const commands: string[] = [];
    let batchInput: StyleSemanticGraphBatchRunnerInputV0 | null = null;
    const queryOptions = {
      sourceDocuments: [],
      styleFiles: [SCSS_PATH, CARD_SCSS_PATH],
      styleSemanticGraphCache,
      runRustSelectedQueryBackendJson: <T>(command: string, input: unknown): T => {
        commands.push(command);
        if (command !== "style-semantic-graph-batch") {
          throw new Error(`unexpected runner command: ${command}`);
        }
        batchInput = input as StyleSemanticGraphBatchRunnerInputV0;
        return {
          schemaVersion: "0",
          product: "omena-semantic.style-semantic-graph-batch",
          graphs: [
            { stylePath: SCSS_PATH, graph: makeGraph(SCSS_PATH) },
            { stylePath: CARD_SCSS_PATH, graph: makeGraph(CARD_SCSS_PATH) },
          ],
        } as T;
      },
    };

    const buttonGraph = resolveRustStyleSemanticGraphForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      queryOptions,
    );
    const cardGraph = resolveRustStyleSemanticGraphForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      CARD_SCSS_PATH,
      queryOptions,
    );

    expect(commands).toEqual(["style-semantic-graph-batch"]);
    expect(batchInput?.engineInput.styles.map((style) => style.filePath)).toEqual([
      SCSS_PATH,
      CARD_SCSS_PATH,
    ]);
    expect(buttonGraph?.selectorReferenceEngine.stylePath).toBe(SCSS_PATH);
    expect(cardGraph?.selectorReferenceEngine.stylePath).toBe(CARD_SCSS_PATH);
  });

  it("passes package manifests into cached multi-style graph batch reads", () => {
    const packageJsonPath = "/fake/ws/node_modules/@design/tokens/package.json";
    const packageJsonSource = `{"exports":{"./theme":{"style":"./dist/theme.css","sass":"./dist/theme.scss"}}}`;
    const packageStylePath = "/fake/ws/node_modules/@design/tokens/dist/theme.css";
    const deps = makeBaseDeps({
      selectorMapForPath: (filePath) =>
        filePath === SCSS_PATH
          ? new Map([["button", infoAtLine("button", 1)]])
          : filePath === CARD_SCSS_PATH
            ? new Map([["card", infoAtLine("card", 1)]])
            : null,
      readStyleFile: (filePath) =>
        filePath === SCSS_PATH
          ? `@use "@design/tokens/theme";\n.button { color: var(--brand); }`
          : filePath === CARD_SCSS_PATH
            ? CARD_SCSS_SOURCE
            : filePath === packageJsonPath
              ? packageJsonSource
              : filePath === packageStylePath
                ? `:root { --brand: #0af; }`
                : null,
      workspaceRoot: "/fake/ws",
    });
    let batchInput: StyleSemanticGraphBatchRunnerInputV0 | null = null;

    resolveRustStyleSemanticGraphForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      {
        sourceDocuments: [],
        styleFiles: [SCSS_PATH, CARD_SCSS_PATH],
        styleSemanticGraphCache: new Map(),
        runRustSelectedQueryBackendJson: <T>(command: string, input: unknown): T => {
          if (command === "omena-parser-style-facts") {
            expect(input).toMatchObject({
              styleSource: `@use "@design/tokens/theme";\n.button { color: var(--brand); }`,
              dialect: "scss",
            });
            return {
              schemaVersion: "0",
              product: "omena-parser.style-facts",
              dialect: "scss",
              sassModuleUseSources: ["@design/tokens/theme"],
              sassModuleForwardSources: [],
              sassModuleImportSources: [],
            } as T;
          }
          if (command !== "style-semantic-graph-batch") {
            throw new Error(`unexpected runner command: ${command}`);
          }
          batchInput = input as StyleSemanticGraphBatchRunnerInputV0;
          return {
            schemaVersion: "0",
            product: "omena-semantic.style-semantic-graph-batch",
            graphs: [
              { stylePath: SCSS_PATH, graph: makeGraph(SCSS_PATH) },
              { stylePath: CARD_SCSS_PATH, graph: makeGraph(CARD_SCSS_PATH) },
            ],
          } as T;
        },
      },
    );

    expect(batchInput?.packageManifests).toEqual([
      {
        packageJsonPath,
        packageJsonSource,
      },
    ]);
    expect(batchInput?.styles.map((style) => style.stylePath)).toContain(packageStylePath);
  });

  it("falls back to single graph reads when batch output omits the exact target path", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: (filePath) =>
        filePath === SCSS_PATH
          ? new Map([["button", infoAtLine("button", 1)]])
          : filePath === CARD_SCSS_PATH
            ? new Map([["card", infoAtLine("card", 1)]])
            : null,
      readStyleFile: (filePath) =>
        filePath === SCSS_PATH
          ? SCSS_SOURCE
          : filePath === CARD_SCSS_PATH
            ? CARD_SCSS_SOURCE
            : null,
      workspaceRoot: "/fake/ws",
    });
    const commands: string[] = [];

    const graph = resolveRustStyleSemanticGraphForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      {
        engineInput: makeEngineInput(),
        sourceDocuments: [],
        styleFiles: [SCSS_PATH, CARD_SCSS_PATH],
        styleSemanticGraphCache: new Map(),
        runRustSelectedQueryBackendJson: <T>(command: string): T => {
          commands.push(command);
          if (command === "style-semantic-graph-batch") {
            return {
              schemaVersion: "0",
              product: "omena-semantic.style-semantic-graph-batch",
              graphs: [{ stylePath: "src/Button.module.scss", graph: makeGraph(SCSS_PATH) }],
            } as T;
          }
          return makeGraph(SCSS_PATH) as T;
        },
      },
    );

    expect(commands).toEqual(["style-semantic-graph-batch", "style-semantic-graph"]);
    expect(graph?.selectorReferenceEngine.stylePath).toBe(SCSS_PATH);
  });

  it("treats cancelled batch graph reads as no graph instead of retrying per target", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: (filePath) =>
        filePath === SCSS_PATH
          ? new Map([["button", infoAtLine("button", 1)]])
          : filePath === CARD_SCSS_PATH
            ? new Map([["card", infoAtLine("card", 1)]])
            : null,
      readStyleFile: (filePath) =>
        filePath === SCSS_PATH
          ? SCSS_SOURCE
          : filePath === CARD_SCSS_PATH
            ? CARD_SCSS_SOURCE
            : null,
      workspaceRoot: "/fake/ws",
    });
    const commands: string[] = [];
    const styleSemanticGraphCache = new Map<string, StyleSemanticGraphSummaryV0 | null>();

    const graph = resolveRustStyleSemanticGraphForWorkspaceTarget(
      {
        workspaceRoot: "/fake/ws",
        classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
        pathAlias: DEFAULT_SETTINGS.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      SCSS_PATH,
      {
        sourceDocuments: [],
        styleFiles: [SCSS_PATH, CARD_SCSS_PATH],
        styleSemanticGraphCache,
        runRustSelectedQueryBackendJson: (command: string): never => {
          commands.push(command);
          throw new EngineShadowRunnerCancelledError("SIGTERM", {
            command: "engine-shadow-runner",
            args: [command],
            cwd: "/fake/ws",
          });
        },
      },
    );

    expect(graph).toBeNull();
    expect(commands).toEqual(["style-semantic-graph-batch"]);
    expect(styleSemanticGraphCache.get(SCSS_PATH)).toBeNull();
    expect(styleSemanticGraphCache.get(CARD_SCSS_PATH)).toBeNull();
  });
});

function makeEngineInput(): EngineInputV2 {
  return {
    version: "2",
    workspace: {
      root: "/fake/ws",
      classnameTransform: DEFAULT_SETTINGS.scss.classnameTransform,
      settingsKey: "precomputed",
    },
    sources: [],
    styles: [],
    typeFacts: [],
  };
}

function makeCssModulesResolution(): StyleSemanticGraphCssModulesCrossFileResolutionV0 {
  return {
    schemaVersion: "0",
    product: "omena-query.css-modules-cross-file-resolution",
    status: "icssExportImportClosureSeed",
    resolutionScope: "batchImportGraph",
    styleCount: 2,
    importEdgeCount: 1,
    resolvedImportEdgeCount: 1,
    unresolvedImportEdgeCount: 0,
    matchedNameCount: 1,
    edges: [
      {
        fromStylePath: SCSS_PATH,
        importKind: "composes",
        source: "./Card.module.scss",
        resolvedStylePath: CARD_SCSS_PATH,
        status: "resolved",
        importGraphDistance: 1,
        importGraphOrder: 0,
        importedNames: ["card"],
        exportedNames: ["card"],
        matchedNames: ["card"],
      },
    ],
    composesClosureEdgeCount: 1,
    valueClosureEdgeCount: 1,
    icssClosureEdgeCount: 1,
    composesCycleCount: 0,
    valueCycleCount: 0,
    icssCycleCount: 0,
    composesClosureEdges: [
      {
        fromStylePath: SCSS_PATH,
        ownerSelectorName: "button",
        targetStylePath: CARD_SCSS_PATH,
        targetSelectorName: "card",
        depth: 1,
        path: [`${SCSS_PATH}#button`, `${CARD_SCSS_PATH}#card`],
      },
    ],
    valueClosureEdges: [
      {
        fromStylePath: SCSS_PATH,
        valueName: "accent",
        targetStylePath: CARD_SCSS_PATH,
        targetValueName: "primary",
        depth: 1,
        path: [`${SCSS_PATH}#accent`, `${CARD_SCSS_PATH}#primary`],
      },
    ],
    icssClosureEdges: [
      {
        fromStylePath: SCSS_PATH,
        name: "forwarded",
        targetStylePath: CARD_SCSS_PATH,
        targetName: "raw",
        depth: 1,
        path: [`${SCSS_PATH}#forwarded`, `${CARD_SCSS_PATH}#raw`],
      },
    ],
    cycles: [],
    capabilities: {
      importSourceResolutionReady: true,
      composesNameMatchReady: true,
      valueNameMatchReady: true,
      icssNameMatchReady: true,
      transitiveClosureReady: true,
      valueGraphClosureReady: true,
      icssExportImportClosureReady: true,
      cycleDetectionReady: true,
    },
    nextPriorities: [],
  };
}

function makeSassModuleResolution(): StyleSemanticGraphSassModuleCrossFileResolutionV0 {
  return {
    schemaVersion: "0",
    product: "omena-query.sass-module-cross-file-resolution",
    status: "crossFileModuleResolution",
    resolutionScope: "batchSassModuleGraph",
    styleCount: 2,
    moduleEdgeCount: 1,
    resolvedModuleEdgeCount: 1,
    unresolvedModuleEdgeCount: 0,
    externalModuleEdgeCount: 0,
    edges: [
      {
        fromStylePath: SCSS_PATH,
        edgeKind: "use",
        source: "./Card.module.scss",
        namespaceKind: "named",
        namespace: "card",
        resolvedStylePath: CARD_SCSS_PATH,
        status: "resolved",
        resolutionKind: "relative",
        candidateCount: 1,
      },
    ],
    capabilities: {
      omenaParserModuleEdgeConsumptionReady: true,
      resolverBackedSourceResolutionReady: true,
      packageManifestResolutionReady: true,
      externalModuleFilteringReady: true,
    },
    nextPriorities: [],
  };
}

function makeGraph(stylePath = SCSS_PATH): StyleSemanticGraphSummaryV0 {
  return {
    schemaVersion: "0",
    product: "omena-semantic.style-semantic-graph",
    language: "scss",
    parserFacts: {},
    semanticFacts: {},
    designTokenSemantics: {
      schemaVersion: "0",
      product: "omena-semantic.design-token-semantics",
      status: "same-file-cascade-ranking-seed",
      resolutionScope: "same-file",
      declarationCount: 3,
      referenceCount: 1,
      resolvedReferenceCount: 1,
      unresolvedReferenceCount: 0,
      selectorsWithReferencesCount: 1,
      contextSignal: {
        declarationContextSelectorCount: 1,
        declarationWrapperContextCount: 0,
        mediaContextSelectorCount: 0,
        supportsContextSelectorCount: 0,
        layerContextSelectorCount: 0,
        wrapperContextCount: 0,
      },
      resolutionSignal: {
        declarationFactCount: 3,
        referenceFactCount: 1,
        sourceOrderedDeclarationCount: 3,
        sourceOrderedReferenceCount: 1,
        occurrenceResolvedReferenceCount: 1,
        occurrenceUnresolvedReferenceCount: 0,
        contextMatchedReferenceCount: 1,
        contextUnmatchedReferenceCount: 0,
        rootDeclarationCount: 1,
        selectorScopedDeclarationCount: 2,
        wrapperScopedDeclarationCount: 0,
      },
      cascadeRankingSignal: {
        rankedReferenceCount: 1,
        unrankedReferenceCount: 0,
        sourceOrderWinnerDeclarationCount: 1,
        sourceOrderShadowedDeclarationCount: 2,
        repeatedNameDeclarationCount: 3,
        rankedReferences: [
          {
            referenceName: "--brand",
            referenceSourceOrder: 0,
            winnerDeclarationSourceOrder: 2,
            shadowedDeclarationSourceOrders: [0, 1],
            candidateDeclarationCount: 3,
          },
        ],
      },
      capabilities: {
        sameFileResolutionReady: true,
        wrapperContextSignalReady: false,
        sourceOrderSignalReady: true,
        sourceOrderCascadeRankingReady: true,
        occurrenceResolutionSignalReady: true,
        selectorContextResolutionReady: true,
        themeOverrideContextSignalReady: true,
        crossFileImportGraphReady: false,
        crossPackageCascadeRankingReady: false,
        themeOverrideContextReady: false,
      },
      blockingGaps: ["crossFileImportGraph", "crossPackageCascadeRanking", "themeOverrideContext"],
      nextPriorities: [
        "crossFileImportGraph",
        "crossPackageCascadeRanking",
        "themeOverrideContext",
      ],
    },
    selectorIdentityEngine: {
      schemaVersion: "0",
      product: "omena-semantic.selector-identity",
      canonicalIdCount: 1,
      canonicalIds: [
        {
          canonicalId: "selector:button",
          localName: "button",
          identityKind: "localClass",
          rewriteSafety: "safe",
          blockers: [],
        },
      ],
      rewriteSafety: {
        allCanonicalIdsRewriteSafe: true,
        safeCanonicalIds: ["selector:button"],
        blockedCanonicalIds: [],
        blockers: [],
      },
    },
    selectorReferenceEngine: {
      schemaVersion: "0",
      product: "omena-semantic.selector-references",
      stylePath,
      selectorCount: 1,
      referencedSelectorCount: 0,
      unreferencedSelectorCount: 1,
      totalReferenceSites: 0,
      selectors: [],
    },
    sourceInputEvidence: {},
    promotionEvidence: {},
    losslessCstContract: {},
  };
}
