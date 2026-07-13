import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";

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
