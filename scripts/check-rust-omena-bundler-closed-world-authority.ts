import { strict as assert } from "node:assert";
import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const cratesRoot = path.join(repoRoot, "rust/crates");
const constructorNeedle = "ClosedWorldBundleV0::try_from_linked_modules(";
const linkedModuleNeedle = "ClosedWorldLinkedModuleV0::new(";

const occurrences = rustSourceFiles(cratesRoot)
  .flatMap((filePath) => occurrencesInFile(filePath, constructorNeedle))
  .filter((occurrence) => !allowedConstructorOccurrence(occurrence));

assert.deepEqual(
  occurrences,
  [],
  `ClosedWorldBundleV0 construction must stay behind the bundler linker or non-product fixtures:\n${formatOccurrences(
    occurrences,
  )}`,
);

const querySource = read("rust/crates/omena-query/src/style/transform.rs");
assert.ok(
  querySource.includes("link_omena_transform_bundle_modules_with_semantic_reachability("),
  "omena-query transform execution must request closed-world bundles through the bundler linker",
);
assert.ok(
  !querySource.includes(constructorNeedle),
  "omena-query must not construct ClosedWorldBundleV0 directly",
);
assert.ok(
  !querySource.includes(linkedModuleNeedle),
  "omena-query must not assemble linked closed-world modules directly",
);

const executorSource = read("rust/crates/omena-transform-passes/src/runtime/executor.rs");
assert.ok(
  executorSource.includes("closed_world_bundle: Option<&'a ClosedWorldBundleV0>"),
  "transform execution must receive closed-world authority as an explicit bundle witness",
);
assert.ok(
  !sourceBeforeTestModule(executorSource).includes(constructorNeedle),
  "transform execution runtime must not construct ClosedWorldBundleV0 directly",
);

console.log(
  JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-bundler.closed-world-authority",
      constructorNeedle,
      productBypassCount: occurrences.length,
    },
    null,
    2,
  ),
);

interface SourceOccurrence {
  readonly relativePath: string;
  readonly line: number;
  readonly lineText: string;
  readonly source: string;
}

function allowedConstructorOccurrence(occurrence: SourceOccurrence): boolean {
  const pathName = occurrence.relativePath;

  if (pathName === "rust/crates/omena-bundler/src/lib.rs") {
    return true;
  }

  if (pathName === "rust/crates/omena-parser/src/closed_world.rs") {
    return true;
  }

  if (
    pathName.startsWith("rust/crates/omena-transform-passes/src/tests") ||
    pathName === "rust/crates/omena-transform-passes/src/tests.rs"
  ) {
    return true;
  }

  if (pathName === "rust/crates/omena-transform-cst/src/lib.rs") {
    return isAfterAnchor(occurrence.source, occurrence.lineText, "#[cfg(test)]");
  }

  if (pathName === "rust/crates/omena-transform-passes/src/runtime/executor.rs") {
    return isAfterAnchor(occurrence.source, occurrence.lineText, "#[cfg(test)]");
  }

  if (pathName === "rust/crates/omena-transform-passes/src/runtime/structural_shadow.rs") {
    return true;
  }

  return false;
}

function rustSourceFiles(root: string): string[] {
  const entries = readdirSync(root).sort();
  const files: string[] = [];
  for (const entry of entries) {
    const absolutePath = path.join(root, entry);
    const stats = statSync(absolutePath);
    if (stats.isDirectory()) {
      files.push(...rustSourceFiles(absolutePath));
      continue;
    }
    if (stats.isFile() && absolutePath.endsWith(".rs")) {
      files.push(absolutePath);
    }
  }
  return files;
}

function occurrencesInFile(filePath: string, needle: string): SourceOccurrence[] {
  const source = read(path.relative(repoRoot, filePath));
  const relativePath = path.relative(repoRoot, filePath);
  return source.split("\n").flatMap((lineText, lineIndex) =>
    lineText.includes(needle)
      ? [
          {
            relativePath,
            line: lineIndex + 1,
            lineText,
            source,
          },
        ]
      : [],
  );
}

function sourceBeforeTestModule(source: string): string {
  const index = source.indexOf("#[cfg(test)]");
  return index < 0 ? source : source.slice(0, index);
}

function isAfterAnchor(source: string, lineText: string, anchor: string): boolean {
  const anchorIndex = source.indexOf(anchor);
  const lineIndex = source.indexOf(lineText);
  return anchorIndex >= 0 && lineIndex > anchorIndex;
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function formatOccurrences(occurrencesToFormat: readonly SourceOccurrence[]): string {
  if (occurrencesToFormat.length === 0) {
    return "<none>";
  }
  return occurrencesToFormat
    .map(
      (occurrence) =>
        `${occurrence.relativePath}:${occurrence.line}: ${occurrence.lineText.trim()}`,
    )
    .join("\n");
}
