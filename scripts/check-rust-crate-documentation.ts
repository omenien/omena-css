import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface CargoTarget {
  readonly kind: readonly string[];
  readonly src_path: string;
}

interface CargoPackage {
  readonly name: string;
  readonly description: string | null;
  readonly readme: string | null;
  readonly targets: readonly CargoTarget[];
  readonly manifest_path: string;
}

interface CargoMetadata {
  readonly packages: readonly CargoPackage[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--manifest-path", "rust/Cargo.toml", "--no-deps", "--format-version", "1"],
    { cwd: repoRoot, encoding: "utf8" },
  ),
) as CargoMetadata;

for (const crate of metadata.packages) {
  assert.ok(crate.description?.trim(), `${crate.name} must declare a package description`);
  assert.ok(crate.readme, `${crate.name} must declare package.readme`);
  const crateDir = path.dirname(crate.manifest_path);
  const readmePath = path.resolve(crateDir, crate.readme);
  assert.ok(existsSync(readmePath), `${crate.name} README does not exist: ${readmePath}`);
  const readme = readFileSync(readmePath, "utf8");
  assert.ok(readme.length >= 100, `${crate.name} README must explain the crate's role`);
  assert.ok(
    readme.toLowerCase().includes(crate.name.toLowerCase()),
    `${crate.name} README must name the crate`,
  );

  for (const target of crate.targets.filter((candidate) => candidate.kind.includes("lib"))) {
    const source = readFileSync(target.src_path, "utf8");
    const head = source.split(/\r?\n/).slice(0, 12).join("\n");
    assert.ok(
      /^\/\/!|^#!\[doc/m.test(head),
      `${crate.name} library must begin with crate-level rustdoc`,
    );
  }
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "1",
      product: "rust.crate-documentation",
      crateCount: metadata.packages.length,
      readmeCount: metadata.packages.filter((crate) => crate.readme).length,
      libraryTargetCount: metadata.packages.flatMap((crate) =>
        crate.targets.filter((target) => target.kind.includes("lib")),
      ).length,
      missingDocumentationCount: 0,
    },
    null,
    2,
  )}\n`,
);
