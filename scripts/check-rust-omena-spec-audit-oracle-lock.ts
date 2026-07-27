import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

interface PackageManifest {
  readonly devDependencies?: Readonly<Record<string, string>>;
}

interface SassSpecManifest {
  readonly source?: {
    readonly pin?: string;
  };
}

interface OracleSourceLock {
  readonly schemaVersion: string;
  readonly product: string;
  readonly npmPackages: Readonly<
    Record<string, { readonly version: string; readonly declaredVersionSource: string }>
  >;
  readonly sassSpecArchive: {
    readonly pin: string;
    readonly declaredVersionSource: string;
  };
}

const lockPath = "rust/crates/omena-spec-audit/data/oracle-source-lock.json";
const sassSpecManifestPath = "rust/crates/omena-diff-test/sass-spec-corpus/manifest.json";
const rootPackage = readJson<PackageManifest>("package.json");
const sassSpecManifest = readJson<SassSpecManifest>(sassSpecManifestPath);
const lock = readJson<OracleSourceLock>(lockPath);

assert.equal(lock.schemaVersion, "0");
assert.equal(lock.product, "omena-spec-audit.oracle-source-lock");
assert.deepEqual(Object.keys(lock.npmPackages).toSorted(), ["lightningcss", "sass"]);

for (const packageName of ["lightningcss", "sass"] as const) {
  const packageVersion = rootPackage.devDependencies?.[packageName];
  assert.ok(packageVersion, `package.json must pin devDependency ${packageName}`);
  assert.deepEqual(lock.npmPackages[packageName], {
    version: packageVersion,
    declaredVersionSource: `package.json#devDependencies.${packageName}`,
  });
}

assert.ok(sassSpecManifest.source?.pin, `${sassSpecManifestPath} must declare source.pin`);
assert.deepEqual(lock.sassSpecArchive, {
  pin: sassSpecManifest.source.pin,
  declaredVersionSource: `${sassSpecManifestPath}#source.pin`,
});

process.stdout.write(
  `${JSON.stringify({
    product: "omena-spec-audit.oracle-source-lock-check",
    npmPackageCount: Object.keys(lock.npmPackages).length,
    sassSpecPin: lock.sassSpecArchive.pin,
    drift: false,
  })}\n`,
);

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}
