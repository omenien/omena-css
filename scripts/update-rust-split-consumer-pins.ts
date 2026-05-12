import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import process from "node:process";

type Fixture = {
  readonly label: string;
  readonly manifestPath: string;
  readonly dependencyNames: readonly string[];
};

const repoRoot = process.cwd();
const fixtures: readonly Fixture[] = [
  {
    label: "omena-abstract-value",
    manifestPath: path.join(
      repoRoot,
      "rust/external-consumers/omena-abstract-value-git-consumer/Cargo.toml",
    ),
    dependencyNames: ["omena-abstract-value"],
  },
  {
    label: "engine-input-producers",
    manifestPath: path.join(
      repoRoot,
      "rust/external-consumers/engine-input-producers-git-consumer/Cargo.toml",
    ),
    dependencyNames: ["engine-input-producers"],
  },
  {
    label: "engine-style-parser",
    manifestPath: path.join(
      repoRoot,
      "rust/external-consumers/engine-style-parser-git-consumer/Cargo.toml",
    ),
    dependencyNames: ["engine-style-parser"],
  },
  {
    label: "omena-semantic",
    manifestPath: path.join(
      repoRoot,
      "rust/external-consumers/omena-semantic-git-consumer/Cargo.toml",
    ),
    dependencyNames: [
      "engine-input-producers",
      "omena-semantic",
      "omena-engine-input-producers",
    ],
  },
  {
    label: "omena-bridge",
    manifestPath: path.join(
      repoRoot,
      "rust/external-consumers/omena-bridge-git-consumer/Cargo.toml",
    ),
    dependencyNames: ["engine-input-producers", "omena-bridge", "omena-engine-input-producers"],
  },
  {
    label: "omena-checker",
    manifestPath: path.join(
      repoRoot,
      "rust/external-consumers/omena-checker-git-consumer/Cargo.toml",
    ),
    dependencyNames: ["omena-checker"],
  },
  {
    label: "omena-resolver",
    manifestPath: path.join(
      repoRoot,
      "rust/external-consumers/omena-resolver-git-consumer/Cargo.toml",
    ),
    dependencyNames: ["engine-input-producers", "omena-resolver", "omena-engine-input-producers"],
  },
  {
    label: "omena-query",
    manifestPath: path.join(
      repoRoot,
      "rust/external-consumers/omena-query-git-consumer/Cargo.toml",
    ),
    dependencyNames: ["engine-input-producers", "omena-query", "omena-engine-input-producers"],
  },
] as const;

const checkOnly = process.argv.includes("--check");

function parseDependency(manifest: string, dependencyName: string) {
  const pattern = new RegExp(`^${escapeRegExp(dependencyName)} = \\{ ([^\\n]+) \\}$`, "m");
  const match = manifest.match(pattern);
  if (!match) {
    throw new Error(`missing pinned git dependency for ${dependencyName}`);
  }
  const body = match[1];
  const repoUrl = body.match(/git = "([^"]+)"/)?.[1];
  const rev = body.match(/rev = "([^"]+)"/)?.[1];
  if (!repoUrl || !rev) {
    throw new Error(`invalid pinned git dependency for ${dependencyName}`);
  }
  return {
    pattern,
    body,
    repoUrl,
    rev,
  };
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function resolveMainSha(repoUrl: string) {
  const output = execFileSync("git", ["ls-remote", repoUrl, "refs/heads/main"], {
    cwd: repoRoot,
    encoding: "utf8",
  }).trim();
  const [sha] = output.split(/\s+/, 1);
  if (!sha) {
    throw new Error(`unable to resolve main SHA for ${repoUrl}`);
  }
  return sha;
}

let changed = false;

for (const fixture of fixtures) {
  const manifest = readFileSync(fixture.manifestPath, "utf8");
  let nextManifest = manifest;
  let fixtureChanged = false;

  for (const dependencyName of fixture.dependencyNames) {
    const dependency = parseDependency(nextManifest, dependencyName);
    const mainSha = resolveMainSha(dependency.repoUrl);
    const shortSha = mainSha.slice(0, 7);
    const dependencyLabel =
      fixture.dependencyNames.length === 1 ? fixture.label : `${fixture.label}/${dependencyName}`;

    if (dependency.rev === shortSha) {
      process.stdout.write(`${dependencyLabel}: already pinned to ${shortSha}\n`);
      continue;
    }

    if (checkOnly) {
      process.stderr.write(
        `${dependencyLabel}: pinned ${dependency.rev} but remote main is ${shortSha}\n`,
      );
      process.exitCode = 1;
      continue;
    }

    nextManifest = nextManifest.replace(
      dependency.pattern,
      `${dependencyName} = { ${dependency.body.replace(/rev = "[^"]+"/, `rev = "${shortSha}"`)} }`,
    );
    fixtureChanged = true;
    process.stdout.write(`${dependencyLabel}: updated ${dependency.rev} -> ${shortSha}\n`);
  }

  if (!checkOnly && fixtureChanged) {
    writeFileSync(fixture.manifestPath, nextManifest);
    execFileSync("cargo", ["generate-lockfile", "--manifest-path", fixture.manifestPath], {
      cwd: repoRoot,
      stdio: "inherit",
    });
    changed = true;
  }
}

if (checkOnly && process.exitCode === 1) {
  process.stderr.write("rust split consumer pins are out of date\n");
} else if (!checkOnly && !changed) {
  process.stdout.write("rust split consumer pins already up to date\n");
}
