// g131-S6: the editor query-consumer thin-driver family (6 -> 1 driver).
/* oxlint-disable no-await-in-loop */

import { AliasResolver } from "../../server/engine-core-ts/src/core/cx/alias-resolver";
import { DocumentAnalysisCache } from "../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { parseStyleDocument } from "../../server/engine-core-ts/src/core/scss/scss-parser";
import { WorkspaceStyleDependencyGraph } from "../../server/engine-core-ts/src/core/semantic/style-dependency-graph";
import {
  NullSemanticWorkspaceReferenceIndex,
  WorkspaceSemanticWorkspaceReferenceIndex,
} from "../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import {
  UNRESOLVABLE_TYPE,
  type TypeResolver,
} from "../../server/engine-core-ts/src/core/ts/type-resolver";
import { DEFAULT_SETTINGS } from "../../server/engine-core-ts/src/settings";
import { runRustSelectedQueryBackendJsonAsync } from "../../server/engine-host-node/src/selected-query-backend";
import { createRequiredRustSourceFrontendAnalysisProvider } from "../../server/engine-host-node/src/source-frontend-analysis-provider";
import { handleCodeAction } from "../../server/lsp-server/src/providers/code-actions";
import { handleCompletion } from "../../server/lsp-server/src/providers/completion";
import { computeDiagnostics } from "../../server/lsp-server/src/providers/diagnostics";
import type {
  CursorParams,
  ProviderDeps,
} from "../../server/lsp-server/src/providers/provider-deps";
import { computeScssUnusedDiagnostics } from "../../server/lsp-server/src/providers/scss-diagnostics";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  CodeActionKind,
  CompletionItemKind,
  DiagnosticSeverity,
  DiagnosticTag,
  type CodeAction,
} from "vscode-languageserver-protocol/node";

async function run_code_action_query_consumer(): Promise<void> {
  const STYLE_URI = "file:///workspace/src/Button.module.scss";
  const STYLE_PATH = "/workspace/src/Button.module.scss";
  const STYLE_SOURCE = ".button { color: #fff; }\n";
  const RANGE = {
    start: { line: 0, character: 17 },
    end: { line: 0, character: 21 },
  };
  const INLINE_STYLE_SOURCE = [
    ".base { color: red; padding: 4px; }",
    ".button {",
    "  composes: base;",
    "  background: blue;",
    "}",
    "",
  ].join("\n");
  const INLINE_RANGE = {
    start: { line: 2, character: 12 },
    end: { line: 2, character: 16 },
  };

  const previousBackend = process.env.OMENA_SELECTED_QUERY_BACKEND;
  process.env.OMENA_SELECTED_QUERY_BACKEND = "rust-selected-query";

  try {
    const errors: unknown[] = [];
    const actions = handleCodeAction(
      {
        textDocument: { uri: STYLE_URI },
        range: RANGE,
        context: { diagnostics: [], triggerKind: 1 },
      },
      {
        fileExists: () => true,
        buildStyleDocument: (filePath: string, content: string) =>
          parseStyleDocument(content, filePath),
        readStyleFile: () => null,
        logError: (_message: string, err: unknown) => errors.push(err),
      } as ProviderDeps,
      STYLE_SOURCE,
    ) as CodeAction[] | null;

    assert.deepEqual(errors, []);
    assert(actions, "code-action provider should return refactor actions");
    const extract = actions.find(
      (action) => action.kind === CodeActionKind.RefactorExtract && action.title.includes("--"),
    );
    assert(extract, "expected query-owned CSS custom property extract refactor");
    assert.equal(extract.title, "Extract CSS custom property '--extracted-color'");
    assert.deepEqual(extract.edit?.changes?.[STYLE_URI], [
      {
        range: {
          start: { line: 0, character: 0 },
          end: { line: 0, character: 0 },
        },
        newText: ":root {\n  --extracted-color: #fff;\n}\n\n",
      },
      {
        range: RANGE,
        newText: "var(--extracted-color)",
      },
    ]);
    const inlineActions = handleCodeAction(
      {
        textDocument: { uri: STYLE_URI },
        range: INLINE_RANGE,
        context: { diagnostics: [], triggerKind: 1 },
      },
      {
        fileExists: () => true,
        buildStyleDocument: (filePath: string, content: string) =>
          parseStyleDocument(content, filePath),
        readStyleFile: () => null,
        logError: (_message: string, err: unknown) => errors.push(err),
      } as ProviderDeps,
      INLINE_STYLE_SOURCE,
    ) as CodeAction[] | null;

    assert.deepEqual(errors, []);
    assert(inlineActions, "code-action provider should return inline refactor actions");
    const inline = inlineActions.find((action) => action.kind === CodeActionKind.RefactorInline);
    assert(inline, "expected query-owned composed-class inline refactor");
    assert.equal(inline.title, "Inline composed class 'base'");
    assert.deepEqual(inline.edit?.changes?.[STYLE_URI], [
      {
        range: {
          start: { line: 2, character: 2 },
          end: { line: 2, character: 17 },
        },
        newText: "color: red;\n  padding: 4px;",
      },
    ]);
    process.stdout.write(
      `validated code-action query consumer: provider=LSP actions=refactor.extract,refactor.inline source=${STYLE_PATH}\n`,
    );
  } finally {
    if (previousBackend === undefined) {
      delete process.env.OMENA_SELECTED_QUERY_BACKEND;
    } else {
      process.env.OMENA_SELECTED_QUERY_BACKEND = previousBackend;
    }
  }
}

async function run_completion_query_consumer(): Promise<void> {
  const SOURCE_PATH = "/workspace/src/App.tsx";
  const SOURCE_URI = "file:///workspace/src/App.tsx";
  const STYLE_PATH = "/workspace/src/Button.module.scss";
  const STYLE_URI = "file:///workspace/src/Button.module.scss";
  const SOURCE_WITH_MARKER = [
    'import classNames from "classnames/bind";',
    'import styles from "./Button.module.scss";',
    "const cx = classNames.bind(styles);",
    'export const view = cx("item--/*|*/ive");',
    "",
  ].join("\n");
  const STYLE_WITH_SELECTORS = [
    ".item--large { color: red; }",
    ".item--active { color: green; }",
    ".item--passive { color: blue; }",
    "",
  ].join("\n");
  const STYLE_WITH_CUSTOM_PROPERTIES = [
    ":root { --alpha: red; }",
    ".card { --zeta: blue; color: var(--/*|*/); }",
    ".next { --omega: red; }",
    "",
  ].join("\n");

  const previousBackend = process.env.OMENA_SELECTED_QUERY_BACKEND;
  const previousDaemon = process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON;
  process.env.OMENA_SELECTED_QUERY_BACKEND = "rust-selected-query";
  process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON = "0";

  main().catch((err: unknown) => {
    console.error(err);
    process.exitCode = 1;
  });

  async function main(): Promise<void> {
    try {
      const sourceFixture = stripMarker(SOURCE_WITH_MARKER);
      const sourceDeps = makeDeps(STYLE_WITH_SELECTORS);
      const sourceItems = await handleCompletion(
        {
          documentUri: SOURCE_URI,
          content: sourceFixture.content,
          filePath: SOURCE_PATH,
          line: sourceFixture.line,
          character: sourceFixture.character,
          version: 1,
        },
        sourceDeps,
      );
      assert(sourceItems, "source completion should return query-ranked selector items");
      assert.deepEqual(
        sourceItems.slice(0, 3).map((item) => item.label),
        ["item--active", "item--passive", "item--large"],
      );
      assert.equal(sourceItems[0]?.data?.product, "omena-query.completion-at");
      assert.equal(sourceItems[0]?.data?.rankingSource, "valueDomainSelectorProjection");
      assert.equal(sourceItems[0]?.kind, CompletionItemKind.Value);

      const styleFixture = stripMarker(STYLE_WITH_CUSTOM_PROPERTIES);
      const styleDeps = makeDeps(styleFixture.content);
      const styleItems = await handleCompletion(
        {
          documentUri: STYLE_URI,
          content: styleFixture.content,
          filePath: STYLE_PATH,
          line: styleFixture.line,
          character: styleFixture.character,
          version: 1,
        },
        styleDeps,
      );
      assert(styleItems, "style completion should return query-ranked custom properties");
      assert.deepEqual(
        styleItems.slice(0, 3).map((item) => item.label),
        ["--zeta", "--alpha", "--omega"],
      );
      assert.equal(styleItems[0]?.data?.product, "omena-query.completion-at");
      assert.equal(styleItems[0]?.data?.rankingSource, "sameFileSourceOrderCascade");
      assert.equal(styleItems[0]?.kind, CompletionItemKind.Variable);

      process.stdout.write(
        [
          "validated completion query consumer:",
          "provider=LSP",
          "sourceRanking=valueDomainSelectorProjection",
          "styleRanking=sameFileSourceOrderCascade",
        ].join(" ") + "\n",
      );
    } finally {
      if (previousBackend === undefined) {
        delete process.env.OMENA_SELECTED_QUERY_BACKEND;
      } else {
        process.env.OMENA_SELECTED_QUERY_BACKEND = previousBackend;
      }
      if (previousDaemon === undefined) {
        delete process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON;
      } else {
        process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON = previousDaemon;
      }
    }
  }

  function makeDeps(styleSource: string): ProviderDeps & {
    readonly runRustSelectedQueryBackendJsonAsync: typeof runRustSelectedQueryBackendJsonAsync;
  } {
    const aliasResolver = new AliasResolver("/workspace", {});
    const fileExists = (filePath: string) => filePath === STYLE_PATH;
    const sourceFrontendAnalysis = createRequiredRustSourceFrontendAnalysisProvider({
      aliasResolver: () => aliasResolver,
      fileExists,
    });
    const analysisCache = new DocumentAnalysisCache({
      sourceFrontendAnalysis,
      fileExists,
      aliasResolver,
      max: 10,
    });
    const typeResolver: TypeResolver = {
      resolve: () => UNRESOLVABLE_TYPE,
      invalidate: () => {},
      clear: () => {},
    };
    return {
      analysisCache,
      aliasResolver,
      styleDocumentForPath: (filePath: string) =>
        filePath === STYLE_PATH ? parseStyleDocument(styleSource, STYLE_PATH) : null,
      typeResolver,
      semanticReferenceIndex: new NullSemanticWorkspaceReferenceIndex(),
      styleDependencyGraph: new WorkspaceStyleDependencyGraph(),
      workspaceRoot: "/workspace",
      workspaceFolderUri: "file:///workspace",
      logError: (_message: string, err: unknown) => {
        throw err;
      },
      invalidateStyle: () => {},
      peekStyleDocument: () => null,
      buildStyleDocument: (filePath: string, content: string) =>
        parseStyleDocument(content, filePath),
      readOpenDocumentText: (filePath: string) => (filePath === STYLE_PATH ? styleSource : null),
      readStyleFile: (filePath: string) => (filePath === STYLE_PATH ? styleSource : null),
      fileExists,
      pushStyleFile: () => {},
      indexerReady: Promise.resolve(),
      stopIndexer: () => {},
      settings: DEFAULT_SETTINGS,
      rebuildAliasResolver: () => {},
      refreshCodeLens: () => {},
      runRustSelectedQueryBackendJsonAsync,
    };
  }

  function stripMarker(source: string): Pick<CursorParams, "content" | "line" | "character"> {
    const marker = "/*|*/";
    const offset = source.indexOf(marker);
    assert(offset >= 0, "fixture must contain cursor marker");
    const content = source.slice(0, offset) + source.slice(offset + marker.length);
    const prefix = source.slice(0, offset);
    const lines = prefix.split("\n");
    return {
      content,
      line: lines.length - 1,
      character: lines[lines.length - 1]!.length,
    };
  }
}

async function run_rename_query_consumer(): Promise<void> {
  const root = mkdtempSync(path.join(os.tmpdir(), "cme-rename-query-consumer-"));

  try {
    const srcDir = path.join(root, "src");
    mkdirSync(srcDir, { recursive: true });
    writeFileSync(
      path.join(srcDir, "App.tsx"),
      [
        'import classNames from "classnames/bind";',
        'import styles from "./App.module.scss";',
        "const cx = classNames.bind(styles);",
        'export const view = <div className={cx("root")} />;',
        "",
      ].join("\n"),
    );
    writeFileSync(path.join(srcDir, "App.module.scss"), ".root { color: red; }\n");
    writeFileSync(
      path.join(srcDir, "types.d.ts"),
      [
        'declare module "*.module.scss" {',
        "  const classes: Record<string, string>;",
        "  export default classes;",
        "}",
        "",
      ].join("\n"),
    );

    const result = spawnSync(
      process.execPath,
      [
        "--import",
        "tsx",
        "./scripts/cme.ts",
        "rename",
        "selector",
        "root",
        "shell",
        "--root",
        root,
        "--target-style",
        path.join(srcDir, "App.module.scss"),
        "--dry-run",
        "--json",
      ],
      {
        cwd: path.resolve(__dirname, ".."),
        env: {
          ...process.env,
          OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
          OMENA_ENGINE_SHADOW_RUNNER_DAEMON: "0",
        },
        encoding: "utf8",
        maxBuffer: 16 * 1024 * 1024,
      },
    );

    assert.equal(result.error, undefined);
    assert.equal(
      result.status,
      0,
      ["rename query consumer should succeed", result.stdout, result.stderr].join("\n"),
    );

    const payload = JSON.parse(result.stdout) as {
      readonly consumer?: string;
      readonly product?: string;
      readonly analysisSource?: string;
      readonly dryRun?: boolean;
      readonly successor?: string;
      readonly migrationPlanProduct?: string;
      readonly readySurfaces?: readonly string[];
      readonly editCount?: number;
      readonly edits?: readonly {
        readonly uri: string;
        readonly newText: string;
        readonly range: {
          readonly start: { readonly line: number; readonly character: number };
          readonly end: { readonly line: number; readonly character: number };
        };
      }[];
    };
    assert.equal(payload.consumer, "cme.rename.selector");
    assert.equal(payload.product, "omena-query.rename-plan");
    assert.equal(payload.analysisSource, "omena-query");
    assert.equal(payload.dryRun, true);
    assert.equal(payload.successor, "omena migrate css-modules-rename");
    assert.equal(payload.migrationPlanProduct, "omena-cli.migration-plan");
    assert(payload.readySurfaces?.includes("workspaceWideSelectorRename"));
    assert.equal(payload.editCount, 2);
    assert.deepEqual(
      payload.edits?.map((edit) => [path.basename(edit.uri), edit.newText]),
      [
        ["App.module.scss", "shell"],
        ["App.tsx", "shell"],
      ],
    );
    assert.deepEqual(payload.edits?.[0]?.range, {
      start: { line: 0, character: 1 },
      end: { line: 0, character: 5 },
    });
    const debtLedger = JSON.parse(
      readFileSync(path.join(path.resolve(__dirname, ".."), "rust/omena-debt-ledger.json"), "utf8"),
    ) as {
      readonly entries: readonly {
        readonly id: string;
        readonly mechanism: string;
        readonly expiry: { readonly after_reference_date: string };
      }[];
    };
    const successionWindow = debtLedger.entries.find(
      (entry) => entry.id === "cme-rename-migration-succession-window",
    );
    assert.equal(successionWindow?.mechanism, "cme-rename-selector-migration-shim");
    assert.ok((successionWindow?.expiry.after_reference_date ?? "") > "2026-07-14");

    process.stdout.write(
      "validated rename query consumer: consumer=cme.rename.selector product=omena-query.rename-plan edits=2\n",
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

async function run_source_diagnostics_query_consumer(): Promise<void> {
  const SOURCE_PATH = "/workspace/src/Button.tsx";
  const SOURCE_URI = "file:///workspace/src/Button.tsx";
  const STYLE_PATH = "/workspace/src/Button.module.scss";
  const STYLE_SOURCE = ".root {}\n.chip {}\n";
  const SOURCE = [
    'import bind from "classnames/bind";',
    'import styles from "./Button.module.scss";',
    'import missing from "./Missing.module.scss";',
    "const cx = bind.bind(styles);",
    'const variant = Math.random() > 0.5 ? "chip" : "ghost";',
    'const dynamicPrefix = "lost-" + suffix;',
    "export function Button({ suffix }) {",
    '  return <div className={cx("ghost", variant, dynamicPrefix, `empty-${suffix}`)} data-x={styles.ghost} />;',
    "}",
    "",
  ].join("\n");

  const previousBackend = process.env.OMENA_SELECTED_QUERY_BACKEND;
  const previousDaemon = process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON;
  process.env.OMENA_SELECTED_QUERY_BACKEND = "rust-selected-query";
  process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON = "0";

  main().catch((err: unknown) => {
    console.error(err);
    process.exitCode = 1;
  });

  async function main(): Promise<void> {
    try {
      const aliasResolver = new AliasResolver("/workspace", {});
      const fileExists = (filePath: string) => filePath === STYLE_PATH;
      const sourceFrontendAnalysis = createRequiredRustSourceFrontendAnalysisProvider({
        aliasResolver: () => aliasResolver,
        fileExists,
      });
      const analysisCache = new DocumentAnalysisCache({
        sourceFrontendAnalysis,
        fileExists,
        aliasResolver,
        max: 10,
      });
      const typeResolver: TypeResolver = {
        resolve: (_filePath, variableName) =>
          variableName === "variant"
            ? { kind: "union", values: ["chip", "ghost"] }
            : UNRESOLVABLE_TYPE,
        invalidate: () => {},
        clear: () => {},
      };
      const deps = {
        analysisCache,
        aliasResolver,
        styleDocumentForPath: (filePath: string) =>
          filePath === STYLE_PATH ? parseStyleDocument(STYLE_SOURCE, STYLE_PATH) : null,
        typeResolver,
        semanticReferenceIndex: new NullSemanticWorkspaceReferenceIndex(),
        styleDependencyGraph: new WorkspaceStyleDependencyGraph(),
        workspaceRoot: "/workspace",
        workspaceFolderUri: "file:///workspace",
        logError: (_message: string, err: unknown) => {
          throw err;
        },
        invalidateStyle: () => {},
        peekStyleDocument: () => null,
        buildStyleDocument: (filePath: string, content: string) =>
          parseStyleDocument(content, filePath),
        readOpenDocumentText: (filePath: string) => (filePath === STYLE_PATH ? STYLE_SOURCE : null),
        readStyleFile: () => {
          throw new Error("source diagnostics query consumer should prefer open style text");
        },
        fileExists,
        pushStyleFile: () => {},
        indexerReady: Promise.resolve(),
        stopIndexer: () => {},
        settings: DEFAULT_SETTINGS,
        rebuildAliasResolver: () => {},
        refreshCodeLens: () => {},
        runRustSelectedQueryBackendJsonAsync,
      } satisfies ProviderDeps & {
        readonly runRustSelectedQueryBackendJsonAsync: typeof runRustSelectedQueryBackendJsonAsync;
      };

      const diagnostics = await computeDiagnostics(
        {
          documentUri: SOURCE_URI,
          content: SOURCE,
          filePath: SOURCE_PATH,
          version: 1,
        },
        deps,
      );
      const byCode = new Map(diagnostics.map((diagnostic) => [diagnostic.code, diagnostic]));
      for (const code of [
        "missingStaticClass",
        "missingResolvedClassValues",
        "missingResolvedClassDomain",
        "missingTemplatePrefix",
      ]) {
        const diagnostic = byCode.get(code);
        assert(diagnostic, `expected omena-query-owned ${code} diagnostic`);
        assert.equal(diagnostic.severity, DiagnosticSeverity.Warning);
        assert.deepEqual(diagnostic.data?.querySeverity, "warning");
        assert.deepEqual(diagnostic.data?.provenance, [
          "omena-query.source-syntax-index",
          "omena-query.style-selector-definitions",
          "omena-query-checker-orchestrator.product-diagnostic-gate",
          "omena-checker.rule-registry",
        ]);
        assert.equal(diagnostic.data?.precision?.product, "omena-query.analysis-precision");
        assert.equal(
          diagnostic.data?.precision?.revisionAxis,
          "OmenaQuerySourceDiagnosticsForFileV0.input",
        );
      }
      assert.deepEqual(
        byCode.get("missingStaticClass")?.data?.createSelector?.selectorName,
        "ghost",
      );

      process.stdout.write(
        [
          "validated source diagnostics query consumer:",
          "provider=LSP",
          "rules=missingStaticClass,missingResolvedClassValues,missingResolvedClassDomain,missingTemplatePrefix",
          "provenance=omena-query",
          "styleSource=open-document",
        ].join(" ") + "\n",
      );
    } finally {
      if (previousBackend === undefined) {
        delete process.env.OMENA_SELECTED_QUERY_BACKEND;
      } else {
        process.env.OMENA_SELECTED_QUERY_BACKEND = previousBackend;
      }
      if (previousDaemon === undefined) {
        delete process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON;
      } else {
        process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON = previousDaemon;
      }
    }
  }
}

async function run_style_diagnostics_query_consumer(): Promise<void> {
  const STYLE_PATH = "/workspace/src/Cascade.module.scss";
  const SOURCE_PATH = "/workspace/src/App.tsx";
  const SOURCE_SOURCE = [
    'import styles from "./Cascade.module.scss";',
    "export const view = <div className={styles.used} />;",
    "",
  ].join("\n");
  const STYLE_SOURCE = `
.used { color: green; }
.ghost { color: purple; }
@layer base {
  .btn { color: red; }
  .dead { border-color: red; }
}
@layer overrides {
  .btn { color: blue; }
  .dead { border-color: blue; }
}
:root {
  --known: #0af;
  --cycle-a: var(--cycle-b);
  --cycle-b: var(--cycle-a);
  --bad: var(--missing);
}
.card { color: var(--bad); background: var(--absent); }
.tie { color: red; color: green; }
`;

  const previousBackend = process.env.OMENA_SELECTED_QUERY_BACKEND;
  const previousDaemon = process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON;
  process.env.OMENA_SELECTED_QUERY_BACKEND = "rust-selected-query";
  process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON = "0";

  main().catch((err: unknown) => {
    console.error(err);
    process.exitCode = 1;
  });

  async function main(): Promise<void> {
    try {
      const styleDocument = parseStyleDocument(STYLE_SOURCE, STYLE_PATH);
      const diagnostics = await computeScssUnusedDiagnostics(
        STYLE_PATH,
        styleDocument,
        new WorkspaceSemanticWorkspaceReferenceIndex(),
        new WorkspaceStyleDependencyGraph(),
        undefined,
        {
          env: process.env,
          styleSource: STYLE_SOURCE,
          sourceDocuments: [{ sourcePath: SOURCE_PATH, sourceSource: SOURCE_SOURCE }],
          runRustSelectedQueryBackendJsonAsync,
        },
      );

      const missingCustomProperty = findDiagnostic(
        diagnostics,
        "missingCustomProperty",
        (diagnostic) => diagnostic.data?.createCustomProperty?.propertyName === "--missing",
      );
      assert.equal(missingCustomProperty.severity, DiagnosticSeverity.Warning);
      assert.deepEqual(missingCustomProperty.data?.querySeverity, "warning");
      assert.deepEqual(missingCustomProperty.data?.provenance, [
        "omena-parser.custom-property-facts",
        "omena-query.style-diagnostics",
        "omena-query-checker-orchestrator.product-diagnostic-gate",
        "omena-checker.rule-registry",
      ]);
      assert.deepEqual(missingCustomProperty.data?.createCustomProperty?.propertyName, "--missing");

      const unreachable = findDiagnostic(diagnostics, "unreachableDeclaration");
      assert.equal(unreachable.severity, DiagnosticSeverity.Hint);
      assert.deepEqual(unreachable.tags, [DiagnosticTag.Unnecessary]);
      assert.deepEqual(unreachable.data?.provenance, [
        "omena-query-checker-orchestrator.cascade-gate",
        "omena-checker.cascade-rules",
        "omena-query.cascade-checker",
        "omena-query.cascade-narrowing",
        "omena-abstract-value.property-value-narrowing",
        "omena-abstract-value.reduced-product-iteration",
        "omena-cascade.margin",
        "omena-query.cascade-confidence",
        "omena-query-checker-orchestrator.product-diagnostic-gate",
        "omena-checker.rule-registry",
      ]);
      assert.deepEqual(
        unreachable.data?.cascadeConfidence?.product,
        "omena-query.cascade-confidence",
      );
      assert.deepEqual(unreachable.data?.cascadeConfidence?.featureGate, "cascade-confidence-v0");
      assert.deepEqual(
        unreachable.data?.cascadeConfidence?.claimLevel,
        "fixtureWitnessResearchHint",
      );
      assert.deepEqual(unreachable.data?.cascadeConfidence?.theoremClaimed, false);
      assert.deepEqual(unreachable.data?.cascadeConfidence?.publicSafetyClaimReady, false);
      assert.deepEqual(
        unreachable.data?.cascadeConfidence?.calibrationStage,
        "fixtureWitnessTierWeightSigmoidV0",
      );
      assert.deepEqual(
        unreachable.data?.polynomialProvenance?.product,
        "omena-abstract-value.polynomial-provenance",
      );
      assert.deepEqual(
        unreachable.data?.polynomialProvenance?.claimLevel,
        "fixtureWitnessPolynomialProjection",
      );
      assert.deepEqual(unreachable.data?.polynomialProvenance?.theoremClaimed, false);
      assert.deepEqual(
        unreachable.data?.polynomialProvenance?.selectedLadder,
        "diagnosticDefaultThreeTier",
      );

      const deadLayer = findDiagnostic(diagnostics, "deadCascadeLayer");
      assert.equal(deadLayer.severity, DiagnosticSeverity.Hint);
      assert.deepEqual(deadLayer.tags, [DiagnosticTag.Unnecessary]);
      assert.deepEqual(deadLayer.data?.provenance, [
        "omena-query-checker-orchestrator.cascade-gate",
        "omena-checker.cascade-rules",
        "omena-query.cascade-checker",
        "omena-query.cascade-narrowing",
        "omena-abstract-value.property-value-narrowing",
        "omena-abstract-value.reduced-product-iteration",
        "omena-cascade.margin",
        "omena-query.cascade-confidence",
        "omena-query-checker-orchestrator.product-diagnostic-gate",
        "omena-checker.rule-registry",
      ]);
      assert.deepEqual(
        deadLayer.data?.cascadeConfidence?.product,
        "omena-query.cascade-confidence",
      );

      const unusedSelector = findDiagnostic(diagnostics, "unusedSelector", (diagnostic) =>
        diagnostic.message.includes("'.ghost'"),
      );
      assert.equal(unusedSelector.severity, DiagnosticSeverity.Hint);
      assert.deepEqual(unusedSelector.tags, [DiagnosticTag.Unnecessary]);
      assert.deepEqual(unusedSelector.data?.provenance, [
        "omena-parser.selector-facts",
        "omena-query.source-selector-usage",
        "omena-query-checker-orchestrator.product-diagnostic-gate",
        "omena-checker.rule-registry",
      ]);

      process.stdout.write(
        [
          "validated style diagnostics query consumer:",
          "provider=LSP",
          "rules=missingCustomProperty,unreachableDeclaration,deadCascadeLayer,unusedSelector",
          "provenance=omena-query",
        ].join(" ") + "\n",
      );
    } finally {
      if (previousBackend === undefined) {
        delete process.env.OMENA_SELECTED_QUERY_BACKEND;
      } else {
        process.env.OMENA_SELECTED_QUERY_BACKEND = previousBackend;
      }
      if (previousDaemon === undefined) {
        delete process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON;
      } else {
        process.env.OMENA_ENGINE_SHADOW_RUNNER_DAEMON = previousDaemon;
      }
    }
  }

  function findDiagnostic(
    diagnostics: Awaited<ReturnType<typeof computeScssUnusedDiagnostics>>,
    code: string,
    predicate: (diagnostic: (typeof diagnostics)[number]) => boolean = () => true,
  ) {
    const diagnostic = diagnostics.find((entry) => entry.code === code && predicate(entry));
    assert(diagnostic, `expected diagnostic ${code}`);
    return diagnostic;
  }
}

async function run_explain_expression_query_consumer(): Promise<void> {
  const root = mkdtempSync(path.join(os.tmpdir(), "cme-explain-query-consumer-"));

  try {
    const sourcePath = path.join(root, "App.tsx");
    const stylePath = path.join(root, "Button.module.scss");
    mkdirSync(path.join(root, "types"), { recursive: true });
    const source = [
      'import classNames from "classnames/bind";',
      'import styles from "./Button.module.scss";',
      "const cx = classNames.bind(styles);",
      "",
      "export function App(enabled: boolean) {",
      '  const size = enabled ? "small" : "large";',
      "  return <div className={cx(size)} />;",
      "}",
      "",
    ].join("\n");
    const style = [".small {", "  color: var(--brand);", "}", ""].join("\n");
    const tsconfig = {
      compilerOptions: {
        jsx: "preserve",
        module: "esnext",
        moduleResolution: "bundler",
        strict: true,
        target: "es2025",
        types: [],
      },
      include: ["App.tsx", "types/**/*.d.ts"],
    };
    const cssModuleDeclaration = [
      'declare module "*.module.scss" {',
      "  const classes: Record<string, string>;",
      "  export default classes;",
      "}",
      "declare namespace JSX { interface IntrinsicElements { div: any } }",
      "",
    ].join("\n");
    writeFileSync(sourcePath, source);
    writeFileSync(stylePath, style);
    writeFileSync(path.join(root, "tsconfig.json"), `${JSON.stringify(tsconfig, null, 2)}\n`);
    writeFileSync(path.join(root, "types", "css-modules.d.ts"), cssModuleDeclaration);

    const cursorOffset = source.indexOf("cx(size)") + "cx(".length;
    assert(cursorOffset >= "cx(".length, "fixture should contain cx(size)");
    const { line, column } = oneBasedLineColumn(source, cursorOffset);
    const result = spawnSync(
      process.execPath,
      [
        "--import",
        "tsx",
        "./scripts/explain-expression.ts",
        `App.tsx:${line}:${column}`,
        "--root",
        root,
        "--json",
      ],
      {
        cwd: path.resolve(__dirname, ".."),
        env: {
          ...process.env,
          OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
        },
        encoding: "utf8",
        maxBuffer: 16 * 1024 * 1024,
      },
    );

    assert.equal(result.error, undefined);
    assert.equal(
      result.status,
      0,
      [
        "explain-expression CLI should succeed through rust-selected-query",
        result.stdout,
        result.stderr,
      ].join("\n"),
    );

    const payload = JSON.parse(result.stdout) as {
      readonly analysisSource?: string;
      readonly selectorNames?: readonly string[];
      readonly analysisV2?: {
        readonly valueDomainKind?: string;
        readonly selectorCertaintyShapeKind?: string;
        readonly valueDomainDerivation?: {
          readonly product?: string;
        };
      };
    };
    assert.equal(payload.analysisSource, "omena-query");
    assert.deepEqual(payload.selectorNames, ["small"]);
    assert.equal(payload.analysisV2?.valueDomainKind, "finiteSet");
    assert.equal(
      payload.analysisV2?.valueDomainDerivation?.product,
      "omena-abstract-value.reduced-class-value-derivation",
    );
    process.stdout.write(
      "validated explain expression query consumer: analysisSource=omena-query selector=small valueDomain=finiteSet\n",
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }

  function oneBasedLineColumn(source: string, offset: number): { line: number; column: number } {
    const prefix = source.slice(0, offset);
    const lines = prefix.split("\n");
    return {
      line: lines.length,
      column: lines[lines.length - 1]!.length + 1,
    };
  }
}

export const QUERY_CONSUMER_FAMILY: {
  readonly [slug: string]: () => Promise<void>;
} = {
  "code-action-query-consumer": run_code_action_query_consumer,
  "completion-query-consumer": run_completion_query_consumer,
  "rename-query-consumer": run_rename_query_consumer,
  "source-diagnostics-query-consumer": run_source_diagnostics_query_consumer,
  "style-diagnostics-query-consumer": run_style_diagnostics_query_consumer,
  "explain-expression-query-consumer": run_explain_expression_query_consumer,
};
