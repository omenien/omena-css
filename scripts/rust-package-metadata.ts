import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

interface CargoMetadata {
  readonly packages: readonly {
    readonly name: string;
    readonly version: string;
    readonly repository: string | null;
  }[];
}

export interface RustPackageMetadata {
  readonly name: string;
  readonly version: string;
  readonly repository: string;
}

export function readRustPackageMetadata(
  packageName: string,
  repoRoot = process.cwd(),
): RustPackageMetadata {
  const result = spawnSync(
    "cargo",
    ["metadata", "--manifest-path", "rust/Cargo.toml", "--format-version", "1", "--no-deps"],
    { cwd: repoRoot, encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr);
  const metadata = JSON.parse(result.stdout) as CargoMetadata;
  const packageMetadata = metadata.packages.find(({ name }) => name === packageName);
  assert.ok(packageMetadata, `cargo metadata is missing ${packageName}`);
  assert.ok(packageMetadata.repository, `${packageName} must declare its repository`);
  return {
    name: packageMetadata.name,
    version: packageMetadata.version,
    repository: packageMetadata.repository,
  };
}
