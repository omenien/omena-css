import { afterEach, describe, expect, it } from "vitest";
import { FileChangeType } from "vscode-languageserver-protocol/node";
import { createInProcessServer, type LspTestClient } from "./_harness/in-process-server";
import type { Range, ResolvedType } from "@omena/shared";
import { textDocumentPositionParams, workspace } from "../../packages/vitest-cme/src";
import type {
  ResolveTypeOptions,
  TypeResolver,
} from "../../server/engine-core-ts/src/core/ts/type-resolver";

// Source-file watcher → semantic reference freshness.
//
// When a TS/JS source file is saved on disk, the server receives a
// `didChangeWatchedFiles` notification. The handler must:
//   1. Invalidate the workspace TypeResolver program.
//   2. Invalidate the analysis cache for every open source document
//      so `onAnalyze` re-fires and semantic reference expansions rebuild
//      with fresh type data.
//
// Without step 2, `analysisCache.get()` returns the cached entry
// (the TSX buffer version hasn't changed), `onAnalyze` never
// re-fires, and semantic reference expansions stay frozen against
// the previous type-resolver output.

/**
 * FakeTypeResolver whose return value can be swapped mid-test.
 * `values` is public so the test can mutate it between watcher
 * events.
 */
class MutableFakeTypeResolver implements TypeResolver {
  values: readonly string[];
  invalidations = 0;

  constructor(values: readonly string[] = []) {
    this.values = values;
  }

  resolve(
    _filePath?: string,
    _variableName?: string,
    _workspaceRoot?: string,
    _range?: Range,
    _options?: ResolveTypeOptions,
  ): ResolvedType {
    return this.values.length > 0
      ? { kind: "union", values: [...this.values] }
      : { kind: "unresolvable", values: [] };
  }

  invalidate(): void {
    this.invalidations += 1;
  }
  clear(): void {}
}

// Uses a bare variable `cx(size)` (not a template `cx(\`btn-${size}\`)`)
// so the semantic expansion goes through `expandVariableRef` →
// `typeResolver.resolve`, not `expandTemplateRef` (prefix match).
const TSX_URI = "file:///fake/workspace/src/App.tsx";
const SCSS_URI = "file:///fake/workspace/src/app.module.scss";

const APP_TSX = `import classNames from 'classnames/bind';
import styles from './app.module.scss';
import { size } from './theme';
const cx = classNames.bind(styles);
export function App() {
  return <div className={cx(size)}>hi</div>;
}
`;

const SCSS_WORKSPACE = workspace({
  [SCSS_URI]: ".s/*at:small*/mall { color: red; }\n.l/*at:large*/arge { color: blue; }\n",
});

function referenceParams(markerName: string) {
  return {
    ...textDocumentPositionParams({
      workspace: SCSS_WORKSPACE,
      documentUri: SCSS_URI,
      filePath: SCSS_URI,
      markerName,
    }),
    context: { includeDeclaration: false },
  };
}

describe("source-watcher → semantic reference freshness", () => {
  let client: LspTestClient | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
  });

  it("source file save invalidates analysis cache and refreshes semantic reference expansions", async () => {
    // TypeResolver initially resolves `size` to "small", so
    // `expandVariableRef` records a semantic reference for `small`.
    const typeResolver = new MutableFakeTypeResolver(["small"]);
    const scssContent = SCSS_WORKSPACE.file(SCSS_URI).content;

    client = createInProcessServer({
      readStyleFile: () => scssContent,
      typeResolver,
    });
    await client.initialize();
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: TSX_URI,
        languageId: "typescriptreact",
        version: 1,
        text: APP_TSX,
      },
    });

    // Initial diagnostics — triggers onAnalyze, populates
    // semantic references with the `small` expansion via expandVariableRef.
    await client.waitForDiagnostics(TSX_URI);

    // Sanity: Find References on `.small` returns the TSX site.
    const refsSmall = await client.references(referenceParams("small"));
    expect(refsSmall).not.toBeNull();
    expect(refsSmall!.some((loc) => loc.uri === TSX_URI)).toBe(true);

    // `.large` should NOT have a reference yet — typeResolver
    // only returned ["small"].
    const refsLargeBefore = await client.references(referenceParams("large"));
    const hadLargeBefore =
      refsLargeBefore !== null && refsLargeBefore.some((loc) => loc.uri === TSX_URI);
    expect(hadLargeBefore).toBe(false);

    // Simulate source change: `size` now resolves to "large".
    typeResolver.values = ["large"];

    // Fire a source-file watcher event (e.g. the file defining
    // `size` was edited and saved).
    client.didChangeWatchedFiles({
      changes: [
        {
          uri: "file:///fake/workspace/src/theme.ts",
          type: FileChangeType.Changed,
        },
      ],
    });

    // Wait for the debounced re-diagnostics — the analysis cache
    // must have been invalidated so `onAnalyze` re-fires with the
    // new type-resolver output.
    await client.waitForDiagnostics(TSX_URI);

    // `.large` should now have a reference from the variable
    // expansion.
    const refsLargeAfter = await client.references(referenceParams("large"));
    expect(refsLargeAfter).not.toBeNull();
    expect(refsLargeAfter!.length).toBeGreaterThan(0);
    expect(refsLargeAfter!.some((loc) => loc.uri === TSX_URI)).toBe(true);
    expect(typeResolver.invalidations).toBe(1);
  });

  it("unrelated source file save does not reschedule open docs", async () => {
    const typeResolver = new MutableFakeTypeResolver(["small"]);
    const scssContent = ".small { color: red; }\n.large { color: blue; }\n";

    client = createInProcessServer({
      readStyleFile: () => scssContent,
      typeResolver,
    });
    await client.initialize();
    client.initialized();

    client.didOpen({
      textDocument: {
        uri: TSX_URI,
        languageId: "typescriptreact",
        version: 1,
        text: APP_TSX,
      },
    });

    await client.waitForDiagnostics(TSX_URI);

    client.didChangeWatchedFiles({
      changes: [
        {
          uri: "file:///fake/workspace/src/unrelated.ts",
          type: FileChangeType.Changed,
        },
      ],
    });

    await expect(client.waitForDiagnostics(TSX_URI, 200)).rejects.toThrow(/timed out/u);
    expect(typeResolver.invalidations).toBe(0);
  });
});
