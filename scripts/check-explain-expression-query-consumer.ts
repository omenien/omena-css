import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import assert from "node:assert/strict";

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
      jsx: "react-jsx",
      module: "esnext",
      moduleResolution: "bundler",
      strict: true,
      target: "es2022",
      types: [],
    },
    include: ["App.tsx", "types/**/*.d.ts"],
  };
  const cssModuleDeclaration = [
    'declare module "*.module.scss" {',
    "  const classes: Record<string, string>;",
    "  export default classes;",
    "}",
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
        CME_SELECTED_QUERY_BACKEND: "rust-selected-query",
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
