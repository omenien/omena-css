import { bench, describe } from "vitest";
import { SourceFileCache } from "../../server/engine-core-ts/src/core/ts/source-file-cache";
import { DocumentAnalysisCache } from "../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { parseClassExpressions } from "../../server/engine-core-ts/src/core/cx/class-ref-parser";
import { scanCxImports } from "../../server/engine-core-ts/src/core/cx/binding-detector";
import type { ProviderDeps } from "../../server/lsp-server/src/providers/cursor-dispatch";
import { handleDefinition } from "../../server/lsp-server/src/providers/definition";
import { handleHover } from "../../server/lsp-server/src/providers/hover";
import { handleCompletion } from "../../server/lsp-server/src/providers/completion";
import { computeDiagnostics } from "../../server/lsp-server/src/providers/diagnostics";
import {
  createTestSourceFrontendAnalysis,
  EMPTY_ALIAS_RESOLVER,
  info,
  makeBaseDeps,
} from "../_fixtures/test-helpers";

const BUTTON_TSX = `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
export function Button() {
  return <div className={cx('indicator')}>hi</div>;
}
`;

const LARGE_TSX = `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
export function Component() {
  return (
    <div>
${Array.from({ length: 100 }, (_, i) => `      <div className={cx('class-${i}')}>item ${i}</div>`).join("\n")}
    </div>
  );
}
`;

function makeSelectorMap() {
  const entries = new Map<string, ReturnType<typeof info>>();
  entries.set("indicator", info("indicator"));
  for (let i = 0; i < 200; i += 1) {
    entries.set(`class-${i}`, info(`class-${i}`));
  }
  return entries;
}

/**
 * Build a `ProviderDeps` wired through the real `scanCxImports` +
 * source-expression parser (the previous bench shape stubbed both
 * to a single hardcoded ref, which meant the numbers it printed
 * were dominated by cache bookkeeping rather than AST traversal).
 * Delegates every non-analysis field to `makeBaseDeps` so bench
 * inherits the canonical ProviderDeps shape — new required fields
 * added to the interface propagate into the bench automatically.
 */
function makeDeps(): ProviderDeps {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
    scanCxImports,
    parseClassExpressions,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
  });
  const analysisCache = new DocumentAnalysisCache({
    sourceFileCache,
    sourceFrontendAnalysis,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    max: 10,
  });
  return makeBaseDeps({
    analysisCache,
    selectorMapForPath: () => makeSelectorMap(),
    workspaceRoot: "/bench",
  });
}

describe("provider cold hover", () => {
  bench("handleHover cold", () => {
    const deps = makeDeps();
    handleHover(
      {
        documentUri: "file:///bench/Button.tsx",
        content: BUTTON_TSX,
        filePath: "/bench/Button.tsx",
        line: 4,
        character: 30,
        version: 1,
      },
      deps,
    );
  });

  bench("handleHover warm (cache hit)", () => {
    const deps = makeDeps();
    const params = {
      documentUri: "file:///bench/Button.tsx",
      content: BUTTON_TSX,
      filePath: "/bench/Button.tsx",
      line: 4,
      character: 30,
      version: 1,
    };
    handleHover(params, deps); // warm
    handleHover(params, deps); // hit
  });

  bench("handleDefinition cold", () => {
    const deps = makeDeps();
    handleDefinition(
      {
        documentUri: "file:///bench/Button.tsx",
        content: BUTTON_TSX,
        filePath: "/bench/Button.tsx",
        line: 4,
        character: 30,
        version: 1,
      },
      deps,
    );
  });

  bench("handleCompletion cold", () => {
    const deps = makeDeps();
    handleCompletion(
      {
        documentUri: "file:///bench/Button.tsx",
        content: BUTTON_TSX,
        filePath: "/bench/Button.tsx",
        line: 4,
        character: 30,
        version: 1,
      },
      deps,
    );
  });
});

describe("diagnostics document-wide scan", () => {
  bench("computeDiagnostics — small file (1 call)", () => {
    const deps = makeDeps();
    computeDiagnostics(
      {
        documentUri: "file:///bench/Button.tsx",
        content: BUTTON_TSX,
        filePath: "/bench/Button.tsx",
        version: 1,
      },
      deps,
    );
  });

  bench("computeDiagnostics — large file (100 calls, large classMap)", () => {
    const deps = makeDeps();
    computeDiagnostics(
      {
        documentUri: "file:///bench/Large.tsx",
        content: LARGE_TSX,
        filePath: "/bench/Large.tsx",
        version: 1,
      },
      deps,
    );
  });
});
