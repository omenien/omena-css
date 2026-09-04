import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  FS_CAPABILITY_BIRTH_SET,
  enumerateRustPublicItems,
  readClippyDisallowedMethods,
  type RustPublicItem,
} from "./lib/rust-write-authority";

interface PartitionRow {
  readonly path: string;
  readonly reason: string;
}

interface PartitionManifest {
  readonly schemaVersion: "0";
  readonly product: "rust.fs-capability-partition";
  readonly rustcRelease: string;
  readonly notBanned: readonly PartitionRow[];
  readonly excludedUnstable: readonly string[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifestPath = path.join(repoRoot, "rust/omena-fs-capability-partition.json");
const sysroot = execFileSync("rustc", ["--print", "sysroot"], {
  cwd: repoRoot,
  encoding: "utf8",
}).trim();
const rustcRelease = execFileSync("rustc", ["--version"], {
  cwd: repoRoot,
  encoding: "utf8",
}).trim();
const stdSourceRoot = path.join(sysroot, "lib/rustlib/src/rust/library/std/src");

function reasonFor(itemPath: string): string {
  const name = itemPath.slice(itemPath.lastIndexOf("::") + 2);
  if (/^(?:Metadata|FileType|Permissions|FileTimes|TryLockError)$/u.test(name)) return "metadata";
  if (/^(?:ReadDir|DirEntry)$/u.test(name)) return "iteration";
  if (
    /^(?:read|read_to_string|read_dir|read_link|canonicalize|metadata|symlink_metadata|exists|open)$/u.test(
      name,
    )
  ) {
    return "observation";
  }
  if (
    /^(?:is_|len$|modified$|accessed$|created$|file_type$|permissions$|path$|file_name$|ino$|dev$|mode$|uid$|gid$|size$|blocks$|blksize$|flags$|attributes$|volume_serial_number$|number_of_links$|file_index$|st_)/u.test(
      name,
    )
  ) {
    return "observation";
  }
  if (/^(?:File|OpenOptions|DirBuilder)$/u.test(name)) return "handle";
  if (/^(?:sync_all|sync_data|lock|lock_shared|try_lock|try_lock_shared|unlock)$/u.test(name)) {
    return "coordination";
  }
  if (
    /^(?:new|options|custom_flags|mode|read|write|append|truncate|create|create_new|recursive)$/u.test(
      name,
    )
  ) {
    return "configuration";
  }
  return "observation";
}

function sourceFiles(): Array<{ absolute: string; publicPath: string }> {
  const platformFiles = execFileSync(
    "find",
    [path.join(stdSourceRoot, "os"), "-mindepth", "2", "-maxdepth", "2", "-name", "fs.rs"],
    { cwd: repoRoot, encoding: "utf8" },
  )
    .trim()
    .split("\n")
    .filter(Boolean)
    .toSorted()
    .map((absolute) => {
      const platform = path.basename(path.dirname(absolute));
      return { absolute, publicPath: `std::os::${platform}::fs` };
    });
  return [{ absolute: path.join(stdSourceRoot, "fs.rs"), publicPath: "std::fs" }, ...platformFiles];
}

function deriveManifest(): PartitionManifest {
  const allItems = sourceFiles().flatMap(({ absolute, publicPath }) =>
    enumerateRustPublicItems(absolute, publicPath),
  );
  const stable = allItems.filter(({ stability }) => stability === "stable");
  const unstable = allItems.filter(({ stability }) => stability === "unstable");
  const configured = readClippyDisallowedMethods(repoRoot);
  const banned = new Set(configured.map(({ path: itemPath }) => itemPath));
  const notBanned = stable
    .filter(({ path: itemPath }) => !banned.has(itemPath))
    .map(({ path: itemPath }) => ({ path: itemPath, reason: reasonFor(itemPath) }));
  return {
    schemaVersion: "0",
    product: "rust.fs-capability-partition",
    rustcRelease,
    notBanned,
    excludedUnstable: unstable.map(({ path: itemPath }) => itemPath).toSorted(),
  };
}

const derived = deriveManifest();
if (process.argv.includes("--print-derived")) {
  process.stdout.write(`${JSON.stringify(derived, null, 2)}\n`);
  process.exit(0);
}
const committed = JSON.parse(readFileSync(manifestPath, "utf8")) as PartitionManifest;
assert.ok(
  committed.notBanned.every(({ reason }) => /^[a-z]+$/u.test(reason)),
  "partition reasons must be one word",
);

const banned = new Set(readClippyDisallowedMethods(repoRoot).map(({ path: itemPath }) => itemPath));
const notBanned = new Set(committed.notBanned.map(({ path: itemPath }) => itemPath));
const stableItems: RustPublicItem[] = sourceFiles()
  .flatMap(({ absolute, publicPath }) => enumerateRustPublicItems(absolute, publicPath))
  .filter(({ stability }) => stability === "stable");
const errors: string[] = [];
for (const itemPath of FS_CAPABILITY_BIRTH_SET.filter((itemPath) => !banned.has(itemPath))) {
  errors.push(`banned item demoted ${itemPath}`);
}
for (const { path: itemPath, allowInvalid } of readClippyDisallowedMethods(repoRoot)) {
  if (itemPath.startsWith("std::os::windows::") && !allowInvalid) {
    errors.push(`non-host disallowed path lacks allow-invalid ${itemPath}`);
  }
  if (!stableItems.some(({ path: candidate }) => candidate === itemPath)) {
    errors.push(`configured banned item is not stable rust-src API ${itemPath}`);
  }
}
for (const item of stableItems) {
  const classifications = Number(banned.has(item.path)) + Number(notBanned.has(item.path));
  if (classifications === 0) errors.push(`unclassified std::fs item ${item.path}`);
  if (classifications === 2) errors.push(`double-classified std::fs item ${item.path}`);
}
assert.deepEqual(errors, [], errors.join("\n"));
assert.deepEqual(committed, derived, "rust filesystem capability partition is stale");

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.fs-capability-partition",
      rustcRelease,
      stableItemCount: stableItems.length,
      bannedItemCount: banned.size,
      notBannedItemCount: notBanned.size,
      excludedUnstableItemCount: committed.excludedUnstable.length,
      unclassifiedItemCount: 0,
      doubleClassifiedItemCount: 0,
    },
    null,
    2,
  )}\n`,
);
