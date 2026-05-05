import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { get } from "node:https";
import { tmpdir } from "node:os";
import path from "node:path";

type PublishReadyRepo = {
  readonly kind: "publish-ready";
  readonly label: string;
  readonly repo: string;
  readonly packageName: string;
  readonly expectedVersion: string;
  readonly libName?: string;
  readonly expectedRegistryDependencies?: readonly RegistryDependency[];
};

type RegistryDependency = {
  readonly dependencyName: string;
  readonly packageName: string;
  readonly version: string;
};

type SplitRepo = PublishReadyRepo;

const splitRepos: readonly SplitRepo[] = [
  {
    kind: "publish-ready",
    label: "input-producers",
    repo: "omenien/omena-engine-input-producers",
    packageName: "omena-engine-input-producers",
    expectedVersion: "0.1.4",
    libName: "engine_input_producers",
    expectedRegistryDependencies: [
      {
        dependencyName: "omena-abstract-value",
        packageName: "omena-abstract-value",
        version: "0.1.8",
      },
    ],
  },
  {
    kind: "publish-ready",
    label: "style-parser",
    repo: "omenien/omena-engine-style-parser",
    packageName: "omena-engine-style-parser",
    expectedVersion: "0.1.6",
    libName: "engine_style_parser",
  },
  {
    kind: "publish-ready",
    label: "semantic",
    repo: "omenien/omena-semantic",
    packageName: "omena-semantic",
    expectedVersion: "0.1.10",
    libName: "omena_semantic",
    expectedRegistryDependencies: [
      {
        dependencyName: "engine-input-producers",
        packageName: "omena-engine-input-producers",
        version: "0.1.4",
      },
      {
        dependencyName: "engine-style-parser",
        packageName: "omena-engine-style-parser",
        version: "0.1.6",
      },
    ],
  },
  {
    kind: "publish-ready",
    label: "abstract-value",
    repo: "omenien/omena-abstract-value",
    packageName: "omena-abstract-value",
    expectedVersion: "0.1.8",
    libName: "omena_abstract_value",
    expectedRegistryDependencies: [
      {
        dependencyName: "omena-incremental",
        packageName: "omena-incremental",
        version: "0.1.2",
      },
    ],
  },
  {
    kind: "publish-ready",
    label: "incremental",
    repo: "omenien/omena-incremental",
    packageName: "omena-incremental",
    expectedVersion: "0.1.2",
    libName: "omena_incremental",
  },
  {
    kind: "publish-ready",
    label: "resolver",
    repo: "omenien/omena-resolver",
    packageName: "omena-resolver",
    expectedVersion: "0.1.2",
    libName: "omena_resolver",
    expectedRegistryDependencies: [
      {
        dependencyName: "engine-input-producers",
        packageName: "omena-engine-input-producers",
        version: "0.1.4",
      },
    ],
  },
  {
    kind: "publish-ready",
    label: "bridge",
    repo: "omenien/omena-bridge",
    packageName: "omena-bridge",
    expectedVersion: "0.1.10",
    libName: "omena_bridge",
    expectedRegistryDependencies: [
      {
        dependencyName: "engine-input-producers",
        packageName: "omena-engine-input-producers",
        version: "0.1.4",
      },
      {
        dependencyName: "engine-style-parser",
        packageName: "omena-engine-style-parser",
        version: "0.1.6",
      },
      {
        dependencyName: "omena-semantic",
        packageName: "omena-semantic",
        version: "0.1.10",
      },
    ],
  },
  {
    kind: "publish-ready",
    label: "query",
    repo: "omenien/omena-query",
    packageName: "omena-query",
    expectedVersion: "0.1.13",
    libName: "omena_query",
    expectedRegistryDependencies: [
      {
        dependencyName: "engine-input-producers",
        packageName: "omena-engine-input-producers",
        version: "0.1.4",
      },
      {
        dependencyName: "engine-style-parser",
        packageName: "omena-engine-style-parser",
        version: "0.1.6",
      },
      {
        dependencyName: "omena-abstract-value",
        packageName: "omena-abstract-value",
        version: "0.1.8",
      },
      {
        dependencyName: "omena-bridge",
        packageName: "omena-bridge",
        version: "0.1.10",
      },
      {
        dependencyName: "omena-resolver",
        packageName: "omena-resolver",
        version: "0.1.2",
      },
      {
        dependencyName: "omena-incremental",
        packageName: "omena-incremental",
        version: "0.1.2",
      },
    ],
  },
  {
    kind: "publish-ready",
    label: "tsgo-client",
    repo: "omenien/omena-tsgo-client",
    packageName: "omena-tsgo-client",
    expectedVersion: "0.1.2",
    libName: "omena_tsgo_client",
  },
  {
    kind: "publish-ready",
    label: "lsp-server",
    repo: "omenien/omena-lsp-server",
    packageName: "omena-lsp-server",
    expectedVersion: "0.1.5",
    libName: "omena_lsp_server",
    expectedRegistryDependencies: [
      {
        dependencyName: "omena-incremental",
        packageName: "omena-incremental",
        version: "0.1.2",
      },
      {
        dependencyName: "omena-query",
        packageName: "omena-query",
        version: "0.1.13",
      },
      {
        dependencyName: "omena-tsgo-client",
        packageName: "omena-tsgo-client",
        version: "0.1.2",
      },
    ],
  },
] as const;

void main().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});

async function main() {
  const tempRoot = mkdtempSync(path.join(tmpdir(), "omena-split-publish-readiness."));

  try {
    for (const repo of splitRepos) {
      const checkoutPath = path.join(tempRoot, repo.label);
      execFileSync(
        "git",
        ["clone", "--quiet", `https://github.com/${repo.repo}.git`, checkoutPath],
        {
          stdio: "inherit",
        },
      );
      const commit = execFileSync("git", ["rev-parse", "--short", "HEAD"], {
        cwd: checkoutPath,
        encoding: "utf8",
      }).trim();
      const manifest = readFileSync(path.join(checkoutPath, "Cargo.toml"), "utf8");
      process.stdout.write(`== rust-split-publish-readiness:${repo.label}@${commit} ==\n`);

      assertManifestBasics(manifest, repo.packageName, repo.expectedVersion, repo.repo);
      runCargo(checkoutPath, ["fmt", "--all", "--check"]);
      runCargo(checkoutPath, ["test"]);
      runCargo(checkoutPath, ["clippy", "--all-targets", "--all-features", "--", "-D", "warnings"]);

      assert.ok(
        !/^publish\s*=\s*false$/m.test(manifest),
        `${repo.packageName}: publish-ready packages must not set publish = false`,
      );
      if (repo.libName !== undefined) {
        assert.match(
          manifest,
          new RegExp(String.raw`^\[lib\]\s*\nname = "${repo.libName}"`, "m"),
          `${repo.packageName}: expected stable Rust lib import name ${repo.libName}`,
        );
      }
      for (const registryDependency of repo.expectedRegistryDependencies ?? []) {
        assertRegistryDependency(manifest, repo.packageName, registryDependency);
      }
      // oxlint-disable-next-line eslint/no-await-in-loop
      const crate = await cratesIoCrate(repo.packageName);
      assert.equal(crate.status, 200, `${repo.packageName}: expected published crate on crates.io`);
      assert.equal(
        crate.maxVersion,
        repo.expectedVersion,
        `${repo.packageName}: expected crates.io max version ${repo.expectedVersion}`,
      );
      runCargo(checkoutPath, ["publish", "--dry-run"]);
      process.stdout.write(
        `validated published split crate: package=${repo.packageName} version=${repo.expectedVersion} cratesIoStatus=${crate.status}\n\n`,
      );
    }
  } finally {
    rmSync(tempRoot, { force: true, recursive: true });
  }
}

function assertManifestBasics(
  manifest: string,
  packageName: string,
  expectedVersion: string,
  repo: string,
) {
  assert.match(manifest, new RegExp(String.raw`^name = "${packageName}"$`, "m"));
  assert.match(
    manifest,
    new RegExp(String.raw`^version = "${escapeRegExp(expectedVersion)}"$`, "m"),
  );
  assert.match(manifest, /^edition = "2024"$/m);
  assert.match(manifest, /^license = "MIT"$/m);
  assert.match(manifest, new RegExp(String.raw`^repository = "https://github.com/${repo}"$`, "m"));
  assert.match(manifest, /^readme = "README\.md"$/m);
  assert.match(manifest, /^keywords = \[/m);
  assert.match(manifest, /^categories = \[/m);
}

function assertRegistryDependency(
  manifest: string,
  packageName: string,
  dependency: RegistryDependency,
) {
  const escapedDependencyName = escapeRegExp(dependency.dependencyName);
  const escapedPackageName = escapeRegExp(dependency.packageName);
  const escapedVersion = escapeRegExp(dependency.version);
  const directDependency = new RegExp(
    String.raw`^${escapedDependencyName} = "${escapedVersion}"$`,
    "m",
  );
  const packageDependency = new RegExp(
    String.raw`^${escapedDependencyName} = \{ package = "${escapedPackageName}", version = "${escapedVersion}" \}$`,
    "m",
  );

  assert.ok(
    directDependency.test(manifest) || packageDependency.test(manifest),
    `${packageName}: expected registry dependency ${dependency.dependencyName} -> ${dependency.packageName}@${dependency.version}`,
  );
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function runCargo(cwd: string, args: readonly string[]) {
  execFileSync("cargo", args, {
    cwd,
    env: {
      ...process.env,
      RUSTUP_TOOLCHAIN: "stable",
    },
    stdio: "inherit",
  });
}

function cratesIoCrate(
  crateName: string,
): Promise<{ readonly status: number; readonly maxVersion: string }> {
  return new Promise((resolve, reject) => {
    const request = get(
      `https://crates.io/api/v1/crates/${crateName}`,
      {
        headers: {
          "User-Agent": "css-module-explainer split publish readiness check",
        },
      },
      (response) => {
        let body = "";
        response.setEncoding("utf8");
        response.on("data", (chunk: string) => {
          body += chunk;
        });
        response.on("end", () => {
          let maxVersion = "";
          if (body.length > 0) {
            const payload = JSON.parse(body) as {
              readonly crate?: { readonly max_version?: string };
            };
            maxVersion = payload.crate?.max_version ?? "";
          }
          resolve({ maxVersion, status: response.statusCode ?? 0 });
        });
      },
    );
    request.on("error", reject);
    request.end();
  });
}
