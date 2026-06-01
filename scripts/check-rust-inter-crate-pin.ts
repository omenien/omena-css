import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/inter-crate-pin
 *
 * Direct-publish blocker guard. Under Model A (`cargo publish --workspace`), every
 * NON-dev inter-crate dependency must carry an EXACT version requirement
 * (`version = "=<workspace version>"`) alongside its `path`: cargo REFUSES to
 * publish a crate whose normal/build dependency is a bare `{ path = ... }` (no
 * versioned requirement to fall back on in the registry). The exact `=` pin also
 * enforces the single-version V0 lockstep — a train member may only resolve to the
 * one workspace version, never a semver-compatible range.
 *
 * The expected requirement is DERIVED from `[workspace.package].version`, so a
 * version bump only needs the pins re-stamped (the release tooling does that) and
 * this gate follows automatically. dev-dependencies are exempt (cargo strips their
 * path at publish and does not require a version), but pinning them too is allowed.
 *
 * A self-test guards the detection predicate itself.
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workspaceManifestPath = path.join(repoRoot, "rust/Cargo.toml");
const workspaceManifest = readFileSync(workspaceManifestPath, "utf8");

// Derive the expected exact requirement from [workspace.package].version.
const workspacePackageMatch = workspaceManifest.match(/\[workspace\.package\]([\s\S]*?)(?:\n\[|$)/);
assert.ok(workspacePackageMatch, "expected a [workspace.package] section in rust/Cargo.toml");
const versionMatch = workspacePackageMatch[1].match(/\bversion = "([^"]+)"/);
assert.ok(versionMatch, 'expected version = "..." under [workspace.package]');
const workspaceVersion = versionMatch[1];
const expectedReq = `=${workspaceVersion}`;

interface CargoDependency {
  readonly name: string;
  readonly req: string;
  readonly kind: string | null;
  readonly path?: string;
}
interface CargoPackage {
  readonly name: string;
  readonly dependencies: readonly CargoDependency[];
}

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  ),
) as { readonly packages: readonly CargoPackage[] };

const memberNames = new Set(metadata.packages.map((pkg) => pkg.name));

/**
 * Shared detection predicate (also exercised by the self-test): a dependency that
 * must carry the exact pin is a NON-dev inter-crate path-dep — it has a `path`, its
 * target is a workspace member, and its kind is not "dev".
 */
function requiresExactPin(
  members: ReadonlySet<string>,
  dep: { readonly name: string; readonly kind: string | null; readonly path?: string },
): boolean {
  return typeof dep.path === "string" && dep.kind !== "dev" && members.has(dep.name);
}

const violations: string[] = [];
let interCrateNonDevDeps = 0;
for (const pkg of metadata.packages) {
  for (const dep of pkg.dependencies) {
    if (!requiresExactPin(memberNames, dep)) {
      continue;
    }
    interCrateNonDevDeps += 1;
    if (dep.req !== expectedReq) {
      violations.push(`${pkg.name} -> ${dep.name}: req "${dep.req}" != "${expectedReq}"`);
    }
  }
}

assert.equal(
  violations.length,
  0,
  `inter-crate dependencies must be pinned EXACT to the workspace version (${expectedReq}) so ` +
    `\`cargo publish --workspace\` accepts them and the V0 lockstep holds:\n  ${violations.join("\n  ")}\n` +
    `Re-stamp the offending \`{ path = ... }\` deps with \`version = "${expectedReq}"\`.`,
);

// Self-test: the predicate flags a non-dev workspace path-dep, ignores a dev-dep,
// ignores a registry (non-path) dep, and ignores a path-dep to a non-member.
{
  const members = new Set(["probe-member"]);
  assert.equal(
    requiresExactPin(members, { name: "probe-member", kind: null, path: "/x" }),
    true,
    "self-test: predicate must flag a normal inter-crate path-dep",
  );
  assert.equal(
    requiresExactPin(members, { name: "probe-member", kind: "dev", path: "/x" }),
    false,
    "self-test: predicate must ignore a dev inter-crate path-dep",
  );
  assert.equal(
    requiresExactPin(members, { name: "probe-member", kind: null }),
    false,
    "self-test: predicate must ignore a non-path (registry) dep",
  );
  assert.equal(
    requiresExactPin(members, { name: "outsider", kind: null, path: "/x" }),
    false,
    "self-test: predicate must ignore a path-dep to a non-member",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.inter-crate-pin",
      workspaceVersion,
      expectedReq,
      interCrateNonDevDeps,
      pinViolations: 0,
    },
    null,
    2,
  )}\n`,
);
