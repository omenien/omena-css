import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { nativeRustBuildEnv } from "./lib/native-rust-toolchain";
import { pnpmCliCommand } from "./lib/pnpm-cli";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const packageDir = path.join(repoRoot, "rust/crates/omena-napi/pkg");
const env = nativeRustBuildEnv();
const buildCommand = pnpmCliCommand(
  [
    "exec",
    "napi",
    "build",
    "--manifest-path",
    "rust/crates/omena-napi/Cargo.toml",
    "--platform",
    "--js-package-name",
    "@omena/napi",
    "--release",
    "-o",
    "rust/crates/omena-napi/pkg",
  ],
  { env },
);

if (process.platform === "darwin") {
  const xcodeVersion = execFileSync("xcodebuild", ["-version"], {
    env,
    encoding: "utf8",
  })
    .trim()
    .replaceAll("\n", "; ");
  console.log(`Building @omena/napi with ${xcodeVersion}`);
}

execFileSync(buildCommand.executable, [...buildCommand.args], {
  cwd: repoRoot,
  env,
  stdio: "inherit",
});

execFileSync(process.execPath, ["./scripts/finalize-omena-napi-pkg.mjs"], {
  cwd: repoRoot,
  env,
  stdio: "inherit",
});

// A successful linker exit is insufficient if dyld rejects the generated Mach-O image.
const loadVerificationEnv = { ...env };
delete loadVerificationEnv.NAPI_RS_NATIVE_LIBRARY_PATH;
execFileSync(process.execPath, ["-e", "require(process.argv[1])", packageDir], {
  cwd: repoRoot,
  env: loadVerificationEnv,
  stdio: "inherit",
});
console.log("Verified @omena/napi native module loading");
