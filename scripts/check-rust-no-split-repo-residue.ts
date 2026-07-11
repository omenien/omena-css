import { strict as assert } from "node:assert";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/no-split-repo-residue
 *
 * Model A (direct publish): the monorepo IS the crates.io publish source. There is
 * no generated standalone workspace, no per-crate split repo, and no remote
 * git-consumer fixture. The OLD split-repo machinery — the
 * `prepare-omena-css-workspace.mjs` generator, the `prepare-*-subtree.sh` push
 * scripts, the `*-git-consumer.sh` checks, and the `rust/external-consumers/`
 * standalone-fixture tree — was retired in §9.3b. This gate is the RESIDUE guard:
 * it fails if any of that machinery (or a package.json hook that drives it) creeps
 * back in, so a stray re-add cannot silently resurrect the split-repo publish path.
 *
 * It proves four structural facts about the working tree:
 *
 *   (1) No generator/subtree/git-consumer SCRIPT under scripts/ — i.e. no
 *       `prepare-omena-css-workspace.mjs`, no `*-subtree.sh`, no `*-git-consumer.sh`.
 *   (2) No `rust/external-consumers/` directory (the standalone-fixture tree).
 *   (3) No package.json script KEY that drives the retired path — a key whose name
 *       carries a split-repo marker (`git-consumer` / `split-publish` / `subtree` /
 *       `omena-css-workspace` / split-repo `cutover`), OR whose command invokes a
 *       retired script (the generator / a `*-subtree.sh` / a `*-git-consumer.sh`).
 *   (4) No `git-consumer` reference in any package.json command or surviving
 *       scripts/ file (the remote-consumer testing layer is gone for good).
 *
 * NOTE on `cutover`: the marker is scoped to the SPLIT-REPO sense. Legitimate,
 * surviving names such as `check:rust-z5-parser-product-cutover` (a benchmark) use
 * `cutover` in an unrelated sense and must NOT be flagged — so a `cutover` key is
 * only residue when it ALSO carries a split-repo marker / invokes a retired script.
 *
 * Self-tests guard each detection predicate. Everything is read from the fs +
 * package.json; this gate takes no arguments and depends on no external tooling.
 */

const selfFileName = path.basename(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const scriptsDir = path.join(repoRoot, "scripts");
const packageJsonPath = path.join(repoRoot, "package.json");
const externalConsumersDir = path.join(repoRoot, "rust", "external-consumers");

// The single retired generator filename (the standalone-workspace synthesizer).
const RETIRED_GENERATOR = "prepare-omena-css-workspace.mjs";

// Split-repo markers that may never re-appear in a package.json script KEY name.
// `cutover` is scoped to the split-repo sense (see the predicate below), so it is
// NOT listed here as a bare substring — it would false-flag the z5 benchmark.
const KEY_NAME_MARKERS = [
  "git-consumer",
  "split-publish",
  "subtree",
  "omena-css-workspace",
] as const;

/** A scripts/ filename is retired split-repo machinery. */
function isRetiredScriptName(fileName: string): boolean {
  return (
    fileName === RETIRED_GENERATOR ||
    fileName.endsWith("-subtree.sh") ||
    fileName.endsWith("-git-consumer.sh")
  );
}

/** A scripts/ command invokes a retired script (generator / subtree / git-consumer). */
function commandInvokesRetiredScript(command: string): boolean {
  return (
    command.includes(RETIRED_GENERATOR) ||
    /[\w./-]*-subtree\.sh\b/.test(command) ||
    /[\w./-]*-git-consumer\.sh\b/.test(command)
  );
}

/**
 * A package.json script (name + command) is split-repo residue when its KEY name
 * carries a split-repo marker (incl. split-repo `cutover`), OR its command invokes a
 * retired script. The `cutover` marker only counts in the split-repo sense: a key
 * named with `cutover` is residue ONLY if it also carries another marker or invokes a
 * retired script — so the standalone z5 benchmark (`...-cutover`) is left alone.
 */
function isResidueScript(name: string, command: string): boolean {
  const nameMarked = KEY_NAME_MARKERS.some((marker) => name.includes(marker));
  const splitRepoCutover =
    name.includes("cutover") && (nameMarked || commandInvokesRetiredScript(command));
  return nameMarked || splitRepoCutover || commandInvokesRetiredScript(command);
}

/** Any `git-consumer` reference (key, command, or file body) is forbidden residue. */
function referencesGitConsumer(text: string): boolean {
  return text.includes("git-consumer");
}

// (1) No retired generator/subtree/git-consumer script under scripts/.
const retiredScripts = existsSync(scriptsDir)
  ? readdirSync(scriptsDir).filter(isRetiredScriptName).toSorted()
  : [];
assert.equal(
  retiredScripts.length,
  0,
  `split-repo residue: retired generator/subtree/git-consumer script(s) still present under scripts/:\n  ${retiredScripts.join(
    "\n  ",
  )}\nUnder Model A the monorepo is the publish source — delete these (§9.3b).`,
);

// (2) No rust/external-consumers/ standalone-fixture tree.
assert.equal(
  existsSync(externalConsumersDir),
  false,
  "split-repo residue: rust/external-consumers/ still exists — the remote git-consumer fixture tree " +
    "was retired in §9.3b. Remove the directory.",
);

// (3) No package.json script key that drives the retired path.
const packageJsonRaw = readFileSync(packageJsonPath, "utf8");
const packageJson = JSON.parse(packageJsonRaw) as { readonly scripts?: Record<string, string> };
const scripts = packageJson.scripts ?? {};
const residueScriptKeys = Object.entries(scripts)
  .filter(([name, command]) => isResidueScript(name, command))
  .map(([name]) => name)
  .toSorted();
assert.equal(
  residueScriptKeys.length,
  0,
  `split-repo residue: package.json script key(s) still drive the retired split-repo path:\n  ${residueScriptKeys.join(
    "\n  ",
  )}\nRemove these script entries (§9.3b) — Model A publishes the workspace directly.`,
);

// (4) No git-consumer reference in any package.json command or surviving scripts/ file.
const gitConsumerReferences: string[] = [];
for (const [name, command] of Object.entries(scripts)) {
  if (referencesGitConsumer(name) || referencesGitConsumer(command)) {
    gitConsumerReferences.push(`package.json scripts.${name}`);
  }
}
if (existsSync(scriptsDir)) {
  for (const entry of readdirSync(scriptsDir, { withFileTypes: true }).toSorted((left, right) =>
    left.name.localeCompare(right.name),
  )) {
    if (!entry.isFile()) {
      continue;
    }
    const fileName = entry.name;
    // This gate's own body legitimately names the retired layer (docstring +
    // predicate + self-tests); it is the residue guard, not residue itself.
    if (fileName === selfFileName) {
      continue;
    }
    const filePath = path.join(scriptsDir, fileName);
    if (!existsSync(filePath)) {
      continue;
    }
    if (referencesGitConsumer(readFileSync(filePath, "utf8"))) {
      gitConsumerReferences.push(`scripts/${fileName}`);
    }
  }
}
assert.equal(
  gitConsumerReferences.length,
  0,
  `split-repo residue: surviving git-consumer reference(s) — the remote-consumer testing layer was retired:\n  ${gitConsumerReferences.join(
    "\n  ",
  )}\nRemove the reference(s).`,
);

// Self-test: the four detection predicates flag residue and clear surviving artifacts.
{
  // (1) retired script-name predicate.
  assert.equal(
    isRetiredScriptName("prepare-omena-css-workspace.mjs"),
    true,
    "self-test: the retired generator filename is residue",
  );
  assert.equal(
    isRetiredScriptName("prepare-omena-css-subtree.sh"),
    true,
    "self-test: a *-subtree.sh script is residue",
  );
  assert.equal(
    isRetiredScriptName("engine-style-parser-git-consumer.sh"),
    true,
    "self-test: a *-git-consumer.sh script is residue",
  );
  assert.equal(
    isRetiredScriptName("prepare-omena-lsp-server.mjs"),
    false,
    "self-test: a surviving prepare-*.mjs (non-generator) build script is not residue",
  );
  assert.equal(
    isRetiredScriptName("check-rust-publish-flags.ts"),
    false,
    "self-test: a surviving metadata gate is not residue",
  );

  // (3) residue-script predicate (name marker / split-repo cutover / command invocation).
  assert.equal(
    isResidueScript(
      "check:rust-omena-css-h2-workspace",
      "node ./scripts/prepare-omena-css-workspace.mjs --temp",
    ),
    true,
    "self-test: a key invoking the retired generator is residue",
  );
  assert.equal(
    isResidueScript("prepare:omena-css-subtree", "bash ./scripts/prepare-omena-css-subtree.sh"),
    true,
    "self-test: a key invoking a *-subtree.sh is residue",
  );
  assert.equal(
    isResidueScript(
      "check:rust-parser-git-consumer",
      "node --import tsx ./scripts/check-rust-parser-git-consumer.ts",
    ),
    true,
    "self-test: a git-consumer-named key is residue",
  );
  assert.equal(
    isResidueScript(
      "check:rust-z5-parser-product-cutover",
      "cargo run --release --manifest-path rust/Cargo.toml -p omena-benchmarks --bin z5_parser_product_cutover",
    ),
    false,
    "self-test: the surviving z5 benchmark 'cutover' key is NOT residue (cutover scoped to split-repo sense)",
  );
  assert.equal(
    isResidueScript(
      "check:rust-publish-flags",
      "node --import tsx ./scripts/check-rust-publish-flags.ts",
    ),
    false,
    "self-test: a surviving metadata gate key is not residue",
  );

  // (4) git-consumer reference predicate.
  assert.equal(
    referencesGitConsumer("check:rust-parser-git-consumer"),
    true,
    "self-test: a git-consumer string is a forbidden reference",
  );
  assert.equal(
    referencesGitConsumer("check:rust-publish-flags"),
    false,
    "self-test: a clean key carries no git-consumer reference",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.no-split-repo-residue",
      retiredScripts: 0,
      externalConsumersDir: false,
      residueScriptKeys: 0,
      gitConsumerReferences: 0,
    },
    null,
    2,
  )}\n`,
);
