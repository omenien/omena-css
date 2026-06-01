import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/publish-flags
 *
 * Model A (direct publish): the monorepo IS the crates.io publish source — there
 * is no generated standalone workspace. `[workspace.package] publish = true`, so
 * every member is publishable BY DEFAULT and the crates.io face is the workspace
 * itself. This gate pins the exact NON-published set so the publish surface stays
 * honest under `cargo publish --workspace`:
 *
 *   publish == false  IFF  (role == "I")  OR  (crate is a known npm-only product)
 *
 *   - [I] internal crates (diff-test / benchmarks / engine-shadow-runner /
 *     engine-style-parser) are never published anywhere.
 *   - npm-only products (omena-napi) ship to npm via @napi-rs, not crates.io, so
 *     they are publish = false on the cargo side even though their role is [P].
 *   - EVERY other member (R1 / R2 / U / S / the crates.io products cli, wasm,
 *     lsp-server) MUST be publishable — a stray publish = false would silently
 *     strand it (and any train dependent that pins it) out of the registry.
 *
 * A self-test guards the predicate. This gate no longer reads the generator —
 * the publishable set is derived purely from `cargo metadata` + the role manifest.
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

// Products that publish to npm (@napi-rs / wasm-pack) but NOT to crates.io. These
// are the only non-[I] members allowed to be publish = false on the cargo side.
const NPM_ONLY_CRATES = new Set(["omena-napi"]);

interface CargoPackage {
  readonly name: string;
  readonly publish: readonly string[] | null;
  readonly metadata?: { readonly omena?: { readonly role?: string } };
}

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  ),
) as { readonly packages: readonly CargoPackage[] };

/** A member must NOT publish to crates.io iff it is [I] or a known npm-only product. */
function mustNotPublishToCrates(pkg: {
  readonly name: string;
  readonly metadata?: { readonly omena?: { readonly role?: string } };
}): boolean {
  return pkg.metadata?.omena?.role === "I" || NPM_ONLY_CRATES.has(pkg.name);
}

const violations: string[] = [];
let publishable = 0;
let nonPublished = 0;
for (const pkg of metadata.packages) {
  const isPublishFalse = Array.isArray(pkg.publish) && pkg.publish.length === 0;
  const expectedNonPublished = mustNotPublishToCrates(pkg);
  if (isPublishFalse) {
    nonPublished += 1;
  } else {
    publishable += 1;
  }
  if (isPublishFalse !== expectedNonPublished) {
    const role = pkg.metadata?.omena?.role ?? "(none)";
    violations.push(
      isPublishFalse
        ? `${pkg.name} (role ${role}) is publish=false but is neither [I] nor npm-only — it would be stranded off crates.io`
        : `${pkg.name} (role ${role}) is publishable but must be publish=false ([I] crate or npm-only product)`,
    );
  }
}

assert.equal(
  violations.length,
  0,
  `publish flags must satisfy: publish=false IFF ([I] role OR npm-only product). Violations:\n  ${violations.join(
    "\n  ",
  )}\nFix the crate's [package] publish flag (or its [package.metadata.omena].role), ` +
    `or update NPM_ONLY_CRATES if a new npm-only product was added.`,
);

// Self-test: the predicate flags [I] and npm-only, and clears everything else.
{
  assert.equal(
    mustNotPublishToCrates({ name: "probe", metadata: { omena: { role: "I" } } }),
    true,
    "self-test: an [I] crate must not publish to crates.io",
  );
  assert.equal(
    mustNotPublishToCrates({ name: "omena-napi", metadata: { omena: { role: "P" } } }),
    true,
    "self-test: a known npm-only product must not publish to crates.io",
  );
  assert.equal(
    mustNotPublishToCrates({ name: "omena-parser", metadata: { omena: { role: "R1" } } }),
    false,
    "self-test: an R1 crate must be publishable",
  );
  assert.equal(
    mustNotPublishToCrates({ name: "omena-cli", metadata: { omena: { role: "P" } } }),
    false,
    "self-test: a crates.io product must be publishable",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.publish-flags",
      members: metadata.packages.length,
      publishable,
      nonPublished,
      npmOnly: [...NPM_ONLY_CRATES],
      flagViolations: 0,
    },
    null,
    2,
  )}\n`,
);
