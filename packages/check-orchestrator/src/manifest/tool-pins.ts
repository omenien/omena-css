import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import type { CheckDiagnostic } from "./types";

interface PackageJsonLike {
  readonly dependencies?: Record<string, string>;
  readonly devDependencies?: Record<string, string>;
  readonly peerDependencies?: Record<string, string>;
}

interface ToolPinLocation {
  readonly packagePath: string;
  readonly dependencyBucket: "dependencies" | "devDependencies" | "peerDependencies";
  readonly packageName: string;
  readonly required?: boolean;
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
];

export function findToolPinCoherenceDiagnostics(rootDir: string): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [];
  const pinsByPackageName = new Map<
    string,
    Array<{ location: ToolPinLocation; version: string }>
  >();
  if (!hasOxcToolchainSurface(rootDir)) {
    return [];
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
