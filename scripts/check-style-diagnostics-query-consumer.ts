import assert from "node:assert/strict";
import { DiagnosticSeverity, DiagnosticTag } from "vscode-languageserver-protocol/node";
import { parseStyleDocument } from "../server/engine-core-ts/src/core/scss/scss-parser";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import { WorkspaceStyleDependencyGraph } from "../server/engine-core-ts/src/core/semantic/style-dependency-graph";
import { runRustSelectedQueryBackendJsonAsync } from "../server/engine-host-node/src/selected-query-backend";
import { computeScssUnusedDiagnostics } from "../server/lsp-server/src/providers/scss-diagnostics";

const STYLE_PATH = "/workspace/src/Cascade.module.scss";
const STYLE_SOURCE = `
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

const previousBackend = process.env.CME_SELECTED_QUERY_BACKEND;
const previousDaemon = process.env.CME_ENGINE_SHADOW_RUNNER_DAEMON;
process.env.CME_SELECTED_QUERY_BACKEND = "rust-selected-query";
process.env.CME_ENGINE_SHADOW_RUNNER_DAEMON = "0";

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
    ]);
    assert.deepEqual(missingCustomProperty.data?.createCustomProperty?.propertyName, "--missing");

    const unreachable = findDiagnostic(diagnostics, "unreachableDeclaration");
    assert.equal(unreachable.severity, DiagnosticSeverity.Hint);
    assert.deepEqual(unreachable.tags, [DiagnosticTag.Unnecessary]);
    assert.deepEqual(unreachable.data?.provenance, [
      "omena-checker.cascade-rules",
      "omena-query.cascade-checker",
    ]);

    const deadLayer = findDiagnostic(diagnostics, "deadCascadeLayer");
    assert.equal(deadLayer.severity, DiagnosticSeverity.Hint);
    assert.deepEqual(deadLayer.tags, [DiagnosticTag.Unnecessary]);
    assert.deepEqual(deadLayer.data?.provenance, [
      "omena-checker.cascade-rules",
      "omena-query.cascade-checker",
    ]);

    process.stdout.write(
      [
        "validated style diagnostics query consumer:",
        "provider=LSP",
        "rules=missingCustomProperty,unreachableDeclaration,deadCascadeLayer",
        "provenance=omena-query",
      ].join(" ") + "\n",
    );
  } finally {
    if (previousBackend === undefined) {
      delete process.env.CME_SELECTED_QUERY_BACKEND;
    } else {
      process.env.CME_SELECTED_QUERY_BACKEND = previousBackend;
    }
    if (previousDaemon === undefined) {
      delete process.env.CME_ENGINE_SHADOW_RUNNER_DAEMON;
    } else {
      process.env.CME_ENGINE_SHADOW_RUNNER_DAEMON = previousDaemon;
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
