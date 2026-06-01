import path from "node:path";
import { spawnSync } from "node:child_process";
import fg from "fast-glob";

type BackendTypecheckVariant = "typescript-current" | "tsgo";

const variant = readVariant();
const fixtureRoot = path.resolve(process.cwd(), "test/_fixtures/backend-typecheck-smoke");
const fixtureTsconfigs = fg
  .sync("*/tsconfig.json", {
    cwd: fixtureRoot,
    absolute: true,
    onlyFiles: true,
  })
  .toSorted();

let exitCode = 0;

for (const fixtureTsconfig of fixtureTsconfigs) {
  const label = path.basename(path.dirname(fixtureTsconfig));
  process.stdout.write(`== backend-typecheck-smoke:${label} (${variant}) ==\n`);
  const child = spawnSync("pnpm", commandForVariant(variant, fixtureTsconfig), {
    cwd: process.cwd(),
    stdio: "inherit",
    env: process.env,
  });
  if ((child.status ?? 1) !== 0) {
    exitCode = child.status ?? 1;
  }
}

process.exit(exitCode);

function readVariant(): BackendTypecheckVariant {
  const value =
    process.env.OMENA_TYPECHECK_VARIANT ?? process.env.OMENA_TYPE_FACT_BACKEND ?? "tsgo";
  if (value === "typescript-current" || value === "tsgo") {
    return value;
  }

  throw new Error(`Unknown backend typecheck variant: ${value}`);
}

function commandForVariant(
  selectedVariant: BackendTypecheckVariant,
  tsconfigPath: string,
): readonly string[] {
  if (selectedVariant === "tsgo") {
    return [
      "exec",
      "tsgo",
      "-p",
      tsconfigPath,
      "--pretty",
      "false",
      "--noEmit",
      ...tsgoCheckerArgs(),
    ];
  }

  return ["exec", "tsc", "-p", tsconfigPath, "--pretty", "false", "--noEmit"];
}

function tsgoCheckerArgs(): readonly string[] {
  const value = process.env.OMENA_TSGO_CHECKERS?.trim();
  if (!value) {
    return [];
  }
  return ["--checkers", value];
}
