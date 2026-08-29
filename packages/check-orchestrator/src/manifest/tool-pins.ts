import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { parse } from "yaml";
import type { CheckDiagnostic } from "./types";

interface PackageJsonLike {
  readonly dependencies?: Record<string, string>;
  readonly devDependencies?: Record<string, string>;
  readonly peerDependencies?: Record<string, string>;
  readonly engines?: Record<string, string>;
}

interface ToolPinLocation {
  readonly packagePath: string;
  readonly dependencyBucket: "dependencies" | "devDependencies" | "peerDependencies";
  readonly packageName: string;
  readonly required?: boolean;
}

interface DependabotUpdatePolicy {
  readonly "package-ecosystem"?: string;
  readonly directory?: string;
  readonly "exclude-paths"?: readonly string[];
  readonly ignore?: readonly { readonly "dependency-name"?: string }[];
  readonly groups?: Readonly<Record<string, { readonly "exclude-patterns"?: readonly string[] }>>;
}

const EXACT_VERSION = /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/;

const TOOL_PIN_LOCATIONS: readonly ToolPinLocation[] = [
  {
    packagePath: "package.json",
    dependencyBucket: "devDependencies",
    packageName: "oxlint",
    required: true,
  },
  {
    packagePath: "package.json",
    dependencyBucket: "devDependencies",
    packageName: "oxfmt",
    required: true,
  },
  {
    packagePath: "packages/oxlint-plugin/package.json",
    dependencyBucket: "peerDependencies",
    packageName: "oxlint",
    required: true,
  },
  {
    packagePath: "examples/package.json",
    dependencyBucket: "devDependencies",
    packageName: "vite-plus",
    required: true,
  },
];

const RUST_OXC_PACKAGES = [
  "oxc_allocator",
  "oxc_ast",
  "oxc_ast_visit",
  "oxc_parser",
  "oxc_semantic",
  "oxc_span",
] as const;

export function findToolPinCoherenceDiagnostics(rootDir: string): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [
    ...findVscodeCompatibilityDiagnostics(rootDir),
    ...findRustOxcCoherenceDiagnostics(rootDir),
    ...findRustApiToolInstallerDiagnostics(rootDir),
    ...findDependabotAuthorityDiagnostics(rootDir),
  ];
  const pinsByPackageName = new Map<
    string,
    Array<{ location: ToolPinLocation; version: string }>
  >();
  if (!hasOxcToolchainSurface(rootDir)) {
    return diagnostics;
  }

  for (const location of TOOL_PIN_LOCATIONS) {
    const packageJson = readPackageJson(rootDir, location.packagePath);
    if (!packageJson) {
      diagnostics.push({
        severity: "error",
        code: "tool-pin-package-missing",
        message: `${location.packagePath} is missing; cannot validate ${location.packageName} pin coherence.`,
      });
      continue;
    }

    const version = packageJson[location.dependencyBucket]?.[location.packageName];
    if (!version) {
      if (location.required) {
        diagnostics.push({
          severity: "error",
          code: "tool-pin-missing",
          message: `${location.packagePath} must declare ${location.packageName} in ${location.dependencyBucket}.`,
        });
      }
      continue;
    }

    if (!EXACT_VERSION.test(version)) {
      diagnostics.push({
        severity: "error",
        code: "tool-pin-not-exact",
        message: `${location.packagePath} ${location.dependencyBucket}.${location.packageName} must be exact-pinned, got "${version}".`,
      });
    }

    const pins = pinsByPackageName.get(location.packageName) ?? [];
    pins.push({ location, version });
    pinsByPackageName.set(location.packageName, pins);
  }

  for (const [packageName, pins] of pinsByPackageName) {
    const versions = new Set(pins.map((pin) => pin.version));
    if (versions.size <= 1) continue;

    diagnostics.push({
      severity: "error",
      code: "tool-pin-version-skew",
      message: `${packageName} must use one exact version across package manifests: ${pins
        .map((pin) => `${pin.location.packagePath}=${pin.version}`)
        .join(", ")}.`,
    });
  }

  return diagnostics;
}

function findRustApiToolInstallerDiagnostics(rootDir: string): readonly CheckDiagnostic[] {
  const relativePath = ".github/actions/install-rust-api-tools/action.yml";
  const absolutePath = path.join(rootDir, relativePath);
  if (!existsSync(absolutePath)) return [];
  const source = readFileSync(absolutePath, "utf8");
  if (/^\s*- uses: taiki-e\/install-action@[0-9a-f]{40}\s+#\s+v\d+\.\d+\.\d+\s*$/mu.test(source)) {
    return [];
  }
  return [
    {
      severity: "error",
      code: "rust-api-tool-installer-unpinned",
      message: `${relativePath} must pin taiki-e/install-action to a 40-character commit SHA.`,
    },
  ];
}

function findDependabotAuthorityDiagnostics(rootDir: string): readonly CheckDiagnostic[] {
  const relativePath = ".github/dependabot.yml";
  const absolutePath = path.join(rootDir, relativePath);
  if (!existsSync(absolutePath)) return [];
  const config = parse(readFileSync(absolutePath, "utf8")) as {
    readonly updates?: readonly DependabotUpdatePolicy[];
  };
  const updates = config.updates ?? [];
  const diagnostics: CheckDiagnostic[] = [];
  const githubActions = updates.find(
    (update) => update["package-ecosystem"] === "github-actions" && update.directory === "/",
  );
  const cargo = updates.find(
    (update) => update["package-ecosystem"] === "cargo" && update.directory === "/rust",
  );
  const npm = updates.find(
    (update) => update["package-ecosystem"] === "npm" && update.directory === "/",
  );
  if (!githubActions?.["exclude-paths"]?.includes(".github/workflows/ci.yml")) {
    diagnostics.push({
      severity: "error",
      code: "dependabot-generated-workflow-authority-leak",
      message:
        `${relativePath} must exclude generated .github/workflows/ci.yml from action updates; ` +
        "update its registry or local composites instead.",
    });
  }
  const cargoIgnored = new Set(
    cargo?.ignore?.flatMap((entry) =>
      entry["dependency-name"] ? [entry["dependency-name"]] : [],
    ) ?? [],
  );
  for (const dependencyName of ["oxc_*", "oxc-*"]) {
    if (cargoIgnored.has(dependencyName)) continue;
    diagnostics.push({
      severity: "error",
      code: "dependabot-rust-oxc-authority-missing",
      message: `${relativePath} must keep ${dependencyName} under manual lockstep authority.`,
    });
  }

  const npmManual = [
    "@types/node",
    "@types/vscode",
    "@typescript/native-preview",
    "sass",
    "@omena/napi",
    "@omena/wasm",
    "oxlint",
    "oxfmt",
    "vite-plus",
    "@tanstack/react-router",
    "@tanstack/react-start",
    "@tanstack/start-static-server-functions",
  ] as const;
  const npmIgnored = new Set(
    npm?.ignore?.flatMap((entry) => (entry["dependency-name"] ? [entry["dependency-name"]] : [])) ??
      [],
  );
  const npmFallbackExcluded = new Set(npm?.groups?.["npm-minor-patch"]?.["exclude-patterns"] ?? []);
  for (const dependencyName of npmManual) {
    if (!npmIgnored.has(dependencyName) || !npmFallbackExcluded.has(dependencyName)) {
      diagnostics.push({
        severity: "error",
        code: "dependabot-manual-authority-leak",
        message: `${relativePath} must both ignore and fallback-exclude ${dependencyName}.`,
      });
    }
  }

  const workspacePath = path.join(rootDir, "pnpm-workspace.yaml");
  if (existsSync(workspacePath)) {
    const workspace = parse(readFileSync(workspacePath, "utf8")) as {
      readonly overrides?: Readonly<Record<string, string>>;
    };
    const nodeTypesOverride = workspace.overrides?.["@types/node"];
    if (!nodeTypesOverride) {
      diagnostics.push({
        severity: "error",
        code: "tool-pin-missing",
        message:
          "pnpm-workspace.yaml must globally override @types/node so cooldown-filtered peer auto-installs cannot select an immature version.",
      });
    } else if (!EXACT_VERSION.test(nodeTypesOverride)) {
      diagnostics.push({
        severity: "error",
        code: "tool-pin-not-exact",
        message: `pnpm-workspace.yaml overrides.@types/node must be exact-pinned, got "${nodeTypesOverride}".`,
      });
    }
  }
  return diagnostics;
}

function findRustOxcCoherenceDiagnostics(rootDir: string): readonly CheckDiagnostic[] {
  const manifestPath = "rust/crates/omena-bridge/Cargo.toml";
  const absolutePath = path.join(rootDir, manifestPath);
  if (!existsSync(absolutePath)) return [];
  const source = readFileSync(absolutePath, "utf8");
  const pins: Array<{ packageName: string; version: string }> = [];
  const diagnostics: CheckDiagnostic[] = [];

  for (const packageName of RUST_OXC_PACKAGES) {
    const match = new RegExp(`^${packageName}\\s*=\\s*"([^"]+)"\\s*$`, "mu").exec(source);
    if (!match) {
      diagnostics.push({
        severity: "error",
        code: "rust-oxc-pin-missing",
        message: `${manifestPath} must exact-pin ${packageName}.`,
      });
      continue;
    }
    const version = match[1]!;
    pins.push({ packageName, version });
    if (!EXACT_VERSION.test(version)) {
      diagnostics.push({
        severity: "error",
        code: "rust-oxc-pin-not-exact",
        message: `${manifestPath} ${packageName} must be exact-pinned, got "${version}".`,
      });
    }
  }

  if (new Set(pins.map((pin) => pin.version)).size > 1) {
    diagnostics.push({
      severity: "error",
      code: "rust-oxc-pin-version-skew",
      message: `Rust OXC crates must use one exact version: ${pins
        .map((pin) => `${pin.packageName}=${pin.version}`)
        .join(", ")}.`,
    });
  }
  return diagnostics;
}

function findVscodeCompatibilityDiagnostics(rootDir: string): readonly CheckDiagnostic[] {
  const packageJson = readPackageJson(rootDir, "package.json");
  const typesRange = packageJson?.devDependencies?.["@types/vscode"];
  const engineRange = packageJson?.engines?.vscode;
  if (!typesRange || !engineRange) return [];

  const typesVersion = firstSemanticVersion(typesRange);
  const engineMinimum = firstSemanticVersion(engineRange);
  if (!typesVersion || !engineMinimum) {
    return [
      {
        severity: "error",
        code: "vscode-compat-version-unparseable",
        message: `Cannot compare @types/vscode "${typesRange}" with engines.vscode "${engineRange}".`,
      },
    ];
  }
  if (compareSemanticVersions(typesVersion, engineMinimum) <= 0) return [];

  return [
    {
      severity: "error",
      code: "vscode-types-engine-skew",
      message: `package.json devDependencies.@types/vscode (${typesRange}) exceeds the engines.vscode minimum (${engineRange}); align the types with the minimum supported editor or deliberately raise the engine floor.`,
    },
  ];
}

function firstSemanticVersion(range: string): readonly [number, number, number] | null {
  const match = range.match(/(\d+)\.(\d+)\.(\d+)/);
  if (!match) return null;
  return [Number(match[1]), Number(match[2]), Number(match[3])];
}

function compareSemanticVersions(
  left: readonly [number, number, number],
  right: readonly [number, number, number],
): number {
  for (let index = 0; index < left.length; index++) {
    const difference = left[index]! - right[index]!;
    if (difference !== 0) return difference;
  }
  return 0;
}

function readPackageJson(rootDir: string, packagePath: string): PackageJsonLike | null {
  const absolutePath = path.join(rootDir, packagePath);
  if (!existsSync(absolutePath)) return null;
  return JSON.parse(readFileSync(absolutePath, "utf8")) as PackageJsonLike;
}

function hasOxcToolchainSurface(rootDir: string): boolean {
  const rootPackage = readPackageJson(rootDir, "package.json");
  if (
    rootPackage?.devDependencies?.oxlint ||
    rootPackage?.devDependencies?.oxfmt ||
    rootPackage?.dependencies?.oxlint ||
    rootPackage?.dependencies?.oxfmt
  ) {
    return true;
  }
  return existsSync(path.join(rootDir, "packages/oxlint-plugin/package.json"));
}
