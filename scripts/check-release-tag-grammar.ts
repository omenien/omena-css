import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";
import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";

/**
 * release/tag-grammar
 *
 * Release tags are immutable source selectors, never event sources. The crate
 * train and CLI accept `release-v<x.y.z>` only through workflow inputs; the VS
 * Code extension owns `vscode-v<x.y.z>`. All workflow `on:` blocks are parsed so
 * tag-push publication or another indirect release event cannot return silently.
 */

type UnknownRecord = Record<string, unknown>;

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workflowsDir = path.join(repoRoot, ".github/workflows");
const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);
const args = new Set(process.argv.slice(2));
const frozenEvents = new Set(["create", "release", "workflow_run", "repository_dispatch"]);

const sources = new Map(
  evidenceScanSurface
    .readdirSync(workflowsDir)
    .filter((name) => /\.ya?ml$/u.test(name))
    .toSorted()
    .map((name) => [name, readFileSync(path.join(workflowsDir, name), "utf8")]),
);

if (args.has("--inject-release-trigger")) {
  sources.set(
    "release-cli.yml",
    sources.get("release-cli.yml")!.replace("on:\n", 'on:\n  push:\n    tags: ["release-v*"]\n'),
  );
}
if (args.has("--inject-frozen-event")) {
  sources.set(
    "release-cli.yml",
    sources.get("release-cli.yml")!.replace("on:\n", "on:\n  release:\n    types: [published]\n"),
  );
}
if (args.has("--inject-release-filter")) {
  sources.set(
    "release-cli.yml",
    sources.get("release-cli.yml")!.replace("on:\n", 'on:\n  push:\n    tags: ["release-*"]\n'),
  );
}
if (args.has("--inject-unfiltered-push")) {
  sources.set(
    "release-cli.yml",
    sources.get("release-cli.yml")!.replace("on:\n", "on:\n  push:\n"),
  );
}
if (args.has("--inject-bare-v-class-filter")) {
  sources.set(
    "release-cli.yml",
    sources.get("release-cli.yml")!.replace("on:\n", 'on:\n  push:\n    tags: ["v[0-9]*"]\n'),
  );
}

let parsedWorkflowCount = 0;
let pushTagFilterCount = 0;
for (const [name, source] of sources) {
  const document = asRecord(parseYaml(source));
  const events = eventRecord(document.on);
  parsedWorkflowCount += 1;
  for (const event of frozenEvents) {
    assert.ok(!events.has(event), `${name} must not use frozen release event ${event}`);
  }
  const push = events.get("push");
  if (push === undefined) continue;
  const pushRecord = asRecord(push);
  const tags = stringList(pushRecord, "tags");
  const tagsIgnore = stringList(pushRecord, "tags-ignore");
  const filters = [...(tags ?? []), ...(tagsIgnore ?? [])];
  pushTagFilterCount += filters.length;
  for (const sample of ["release-v0.0.0", "release-v999.999.999", "v0.0.0"]) {
    assert.equal(
      pushRunsForTag(pushRecord, tags, tagsIgnore, sample),
      false,
      `${name} push trigger must not run for release tag ${JSON.stringify(sample)}`,
    );
  }
}

const releaseCli = sources.get("release-cli.yml")!;
const crateTrain = sources.get("_publish-crate-train.yml")!;
const publishExt = sources.get("publish-extension.yml")!;
const releaseRegex = "^release-v[0-9]+\\.[0-9]+\\.[0-9]+$";

assert.equal(
  occurrences(crateTrain, releaseRegex),
  2,
  "crate train must validate the dispatch tag in both source guards",
);
assert.equal(
  occurrences(releaseCli, releaseRegex),
  1,
  "release CLI stage must validate its dispatch tag exactly once",
);
assert.ok(
  /TAG="vscode-v\$\{VERSION\}"/u.test(publishExt),
  "publish-extension.yml stable tag must be vscode-v${VERSION}",
);
assert.ok(
  /TAG="vscode-v\$\{VERSION\}-preview/u.test(publishExt),
  "publish-extension.yml preview tag must use the vscode-v prefix",
);
assert.ok(
  !/TAG="v\$\{VERSION\}"/u.test(publishExt),
  "publish-extension.yml must not create a bare v${VERSION} tag",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "release.tag-grammar",
      crateTrainTagPrefix: "release-v",
      vsixTagPrefix: "vscode-v",
      bareVTagForbidden: true,
      pushedReleaseTagsStartPublication: false,
      parsedWorkflowCount,
      pushTagFilterCount,
      frozenReleaseEventCount: 0,
      dispatchReleaseTagValidationCount: 3,
    },
    null,
    2,
  )}\n`,
);

function asRecord(value: unknown): UnknownRecord {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? (value as UnknownRecord)
    : {};
}

function eventRecord(value: unknown): Map<string, unknown> {
  if (typeof value === "string") return new Map([[value, null]]);
  if (Array.isArray(value)) return new Map(value.map((event) => [String(event), null]));
  return new Map(Object.entries(asRecord(value)));
}

function stringList(record: UnknownRecord, key: string): readonly string[] | undefined {
  if (!Object.hasOwn(record, key)) return undefined;
  const value = record[key];
  if (typeof value === "string") return [value];
  assert.ok(Array.isArray(value), `push.${key} must be a string or string array`);
  assert.ok(
    value.every((item) => typeof item === "string"),
    `push.${key} contains non-text`,
  );
  return value as string[];
}

function pushRunsForTag(
  push: UnknownRecord,
  tags: readonly string[] | undefined,
  tagsIgnore: readonly string[] | undefined,
  sample: string,
): boolean {
  if (tags) {
    let included = false;
    for (const pattern of tags) {
      if (pattern.startsWith("!")) {
        if (globMatches(pattern.slice(1), sample)) included = false;
      } else if (globMatches(pattern, sample)) {
        included = true;
      }
    }
    return included;
  }
  if (tagsIgnore) return !tagsIgnore.some((pattern) => globMatches(pattern, sample));
  if (Object.hasOwn(push, "branches") || Object.hasOwn(push, "branches-ignore")) return false;
  return true;
}

function globMatches(pattern: string, sample: string): boolean {
  let expression = "";
  for (let index = 0; index < pattern.length; index += 1) {
    const character = pattern[index]!;
    if (character === "*") {
      while (pattern[index + 1] === "*") index += 1;
      expression += ".*";
      continue;
    }
    if (character === "?") {
      expression += ".";
      continue;
    }
    if (character === "[") {
      const closing = pattern.indexOf("]", index + 1);
      assert.ok(closing > index + 1, `cannot classify push tag glob ${JSON.stringify(pattern)}`);
      const rawClass = pattern.slice(index + 1, closing);
      assert.match(rawClass, /^!?[A-Za-z0-9_-]+$/u, `unsupported tag glob class in ${pattern}`);
      expression += `[${rawClass.startsWith("!") ? `^${rawClass.slice(1)}` : rawClass}]`;
      index = closing;
      continue;
    }
    if (character === "\\") {
      const escaped = pattern[index + 1];
      assert(escaped, `dangling escape in push tag glob ${JSON.stringify(pattern)}`);
      expression += escapeRegex(escaped);
      index += 1;
      continue;
    }
    assert.ok(
      !"+{}()|".includes(character),
      `cannot classify push tag glob ${JSON.stringify(pattern)} containing ${character}`,
    );
    expression += escapeRegex(character);
  }
  return new RegExp(`^${expression}$`, "u").test(sample);
}

function escapeRegex(character: string): string {
  return ".^$|(){}+[]\\".includes(character) ? `\\${character}` : character;
}

function occurrences(source: string, needle: string): number {
  return source.split(needle).length - 1;
}
