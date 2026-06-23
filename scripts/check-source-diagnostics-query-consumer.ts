import assert from "node:assert/strict";
import { DiagnosticSeverity } from "vscode-languageserver-protocol/node";
import { cssModulesClassnamesBinderPluginV0 } from "../server/engine-core-ts/src/core/binder/binder-plugin";
import { AliasResolver } from "../server/engine-core-ts/src/core/cx/alias-resolver";
import { DocumentAnalysisCache } from "../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { parseStyleDocument } from "../server/engine-core-ts/src/core/scss/scss-parser";
import { WorkspaceStyleDependencyGraph } from "../server/engine-core-ts/src/core/semantic/style-dependency-graph";
import { NullSemanticWorkspaceReferenceIndex } from "../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import { SourceFileCache } from "../server/engine-core-ts/src/core/ts/source-file-cache";
import {
  UNRESOLVABLE_TYPE,
  type TypeResolver,
} from "../server/engine-core-ts/src/core/ts/type-resolver";
import { DEFAULT_SETTINGS } from "../server/engine-core-ts/src/settings";
import { runRustSelectedQueryBackendJsonAsync } from "../server/engine-host-node/src/selected-query-backend";
import { computeDiagnostics } from "../server/lsp-server/src/providers/diagnostics";
import type { ProviderDeps } from "../server/lsp-server/src/providers/provider-deps";

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
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const analysisCache = new DocumentAnalysisCache({
      sourceFileCache,
      binderPlugin: cssModulesClassnamesBinderPluginV0,
      fileExists: (filePath) => filePath === STYLE_PATH,
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
      fileExists: (filePath: string) => filePath === STYLE_PATH,
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
    assert.deepEqual(byCode.get("missingStaticClass")?.data?.createSelector?.selectorName, "ghost");

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
