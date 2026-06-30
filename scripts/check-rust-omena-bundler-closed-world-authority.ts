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
for (const functionName of [
  "fn run_tree_shake_class_structural",
  "fn run_tree_shake_keyframes_structural",
  "fn run_tree_shake_value_structural",
  "fn run_tree_shake_custom_property_structural",
] as const) {
  const functionBody = functionBodyFromSource(executorSource, functionName);
  assert.ok(
    functionBody.includes("input.closed_world_bundle()"),
    `${functionName} must require an explicit closed-world bundle witness`,
  );
  assert.ok(
    functionBody.includes("bundle.reachability()"),
    `${functionName} must read reachable symbols from the closed-world bundle`,
  );
  assert.ok(
    !functionBody.includes("input.context.reachable") &&
      !functionBody.includes("context.reachable"),
    `${functionName} must not read reachable symbols from caller context`,
  );
}

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

function functionBodyFromSource(source: string, functionName: string): string {
  const anchor = source.indexOf(functionName);
  assert.ok(anchor >= 0, `${functionName} must exist`);
  const afterSignature = anchor + functionName.length;
  const nextFunction = source.slice(afterSignature).search(/\nfn\s/u);
  assert.ok(nextFunction >= 0, `${functionName} must be delimited by the next function`);
  return source.slice(anchor, afterSignature + nextFunction);
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
