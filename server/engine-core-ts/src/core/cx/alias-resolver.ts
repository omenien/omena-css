import * as path from "node:path";
import ts from "typescript";
import {
  loadWorkspaceBundlerPathAliases,
  type BundlerPathAliases,
} from "./bundler-alias-extractor";

/**
 * Workspace-scoped path resolver for CSS Module imports.
 *
 * Merges two alias sources:
 *   - `omena.pathAlias`
 *   - literal `resolve.alias` entries from Vite/Webpack config files
 *   - `compilerOptions.paths` from the workspace tsconfig/jsconfig
 *
 * `${workspaceFolder}` in settings targets is substituted at
 * construction time; relative settings targets are resolved
 * against `workspaceRoot`. tsconfig/jsconfig targets are resolved
 * against `compilerOptions.baseUrl` when present, otherwise the
 * config file directory (`pathsBasePath` in the parsed TS config).
 *
 * Matching uses longest-pattern order instead of relying on
 * insertion order. Explicit settings aliases win ties over
 * tsconfig paths so user-authored extension config can override
 * project config when needed.
 *
 * The resolver returns an absolute path even when the file does
 * not exist. When multiple tsconfig targets exist for one pattern,
 * an optional `fileExists` callback lets callers prefer the first
 * candidate that exists on disk; otherwise the first target wins.
 */

export interface TsconfigPathAliases {
  readonly basePath: string;
  readonly paths: Readonly<Record<string, readonly string[]>>;
}

type AliasEntry =
  | {
      readonly kind: "prefix";
      readonly source: "settings" | "bundler" | "tsconfig";
      readonly pattern: string;
      readonly target: string;
    }
  | {
      readonly kind: "wildcard";
      readonly source: "tsconfig";
      readonly pattern: string;
      readonly prefix: string;
      readonly suffix: string;
      readonly targets: readonly string[];
    };

interface TsconfigPathAliasSystem extends Pick<
  typeof ts.sys,
  "fileExists" | "readFile" | "readDirectory" | "useCaseSensitiveFileNames"
> {
  readonly onUnRecoverableConfigFileDiagnostic?: (diagnostic: ts.Diagnostic) => void;
}

function compareAliasEntries(a: AliasEntry, b: AliasEntry): number {
  if (a.pattern.length !== b.pattern.length) {
    return b.pattern.length - a.pattern.length;
  }
  if (a.source !== b.source) {
    return aliasSourcePriority(a.source) - aliasSourcePriority(b.source);
  }
  return a.pattern.localeCompare(b.pattern);
}

function aliasSourcePriority(source: AliasEntry["source"]): number {
  switch (source) {
    case "settings":
      return 0;
    case "bundler":
      return 1;
    case "tsconfig":
      return 2;
  }
}

function findWorkspaceConfigPath(
  workspaceRoot: string,
  sys: Pick<typeof ts.sys, "fileExists">,
): string | null {
  const tsconfig = path.join(workspaceRoot, "tsconfig.json");
  if (sys.fileExists(tsconfig)) return tsconfig;
  const jsconfig = path.join(workspaceRoot, "jsconfig.json");
  if (sys.fileExists(jsconfig)) return jsconfig;
  return null;
}

export function loadWorkspaceTsconfigPathAliases(
  workspaceRoot: string,
  system: TsconfigPathAliasSystem = {
    fileExists: ts.sys.fileExists,
    readFile: ts.sys.readFile,
    readDirectory: ts.sys.readDirectory,
    useCaseSensitiveFileNames: ts.sys.useCaseSensitiveFileNames,
    onUnRecoverableConfigFileDiagnostic: () => {},
  },
): TsconfigPathAliases | null {
  try {
    const configPath = findWorkspaceConfigPath(workspaceRoot, system);
    if (!configPath) return null;
    return loadTsconfigPathAliasesFromConfig(configPath, system);
  } catch {
    return null;
  }
}

function loadTsconfigPathAliasesFromConfig(
  configPath: string,
  system: TsconfigPathAliasSystem,
): TsconfigPathAliases | null {
  const parsed = ts.getParsedCommandLineOfConfigFile(configPath, undefined, {
    ...ts.sys,
    ...system,
    onUnRecoverableConfigFileDiagnostic: system.onUnRecoverableConfigFileDiagnostic ?? (() => {}),
  });
  if (!parsed || !parsed.options.paths) return null;
  const baseUrl = typeof parsed.options.baseUrl === "string" ? parsed.options.baseUrl : undefined;
  const pathsBasePath =
    typeof parsed.options.pathsBasePath === "string" ? parsed.options.pathsBasePath : undefined;

  return {
    basePath: baseUrl ?? pathsBasePath ?? path.dirname(configPath),
    paths: parsed.options.paths,
  };
}

function findNearestConfigPath(
  filePath: string,
  workspaceRoot: string,
  sys: Pick<typeof ts.sys, "fileExists">,
): string | null {
  const root = path.resolve(workspaceRoot);
  let current = path.dirname(path.resolve(filePath));

  while (isWithinOrEqual(current, root)) {
    const tsconfig = path.join(current, "tsconfig.json");
    if (sys.fileExists(tsconfig)) return tsconfig;
    const jsconfig = path.join(current, "jsconfig.json");
    if (sys.fileExists(jsconfig)) return jsconfig;
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }

  return null;
}

function isWithinOrEqual(candidate: string, root: string): boolean {
  const relative = path.relative(root, candidate);
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}
/**
 * Mutable holder for the workspace-scoped `AliasResolver`. The
 * composition root creates one at startup; `rebuildAliasResolver`
 * swaps the value on settings change. Consumers (the analysis
 * cache's `aliasResolver` getter) call `get()` at analyze time
 * so they always see the latest resolver without rewiring.
 */
export class AliasResolverHolder {
  private current: AliasResolver;
  private settingsPathAlias: Readonly<Record<string, string>>;

  constructor(
    private readonly workspaceRoot: string,
    initial: Readonly<Record<string, string>>,
  ) {
    this.settingsPathAlias = initial;
    this.current = this.build();
  }

  get(): AliasResolver {
    return this.current;
  }

  rebuild(pathAlias: Readonly<Record<string, string>> = this.settingsPathAlias): void {
    this.settingsPathAlias = pathAlias;
    this.current = this.build();
  }

  private build(): AliasResolver {
    return new AliasResolver(
      this.workspaceRoot,
      this.settingsPathAlias,
      loadWorkspaceTsconfigPathAliases(this.workspaceRoot),
      undefined,
      loadWorkspaceBundlerPathAliases(this.workspaceRoot),
    );
  }
}

export class AliasResolver {
  private readonly settingsEntries: readonly AliasEntry[];
  private readonly bundlerEntries: readonly AliasEntry[];
  private readonly defaultTsconfigEntries: readonly AliasEntry[];
  private readonly tsconfigEntriesByConfigPath = new Map<string, readonly AliasEntry[]>();

  constructor(
    private readonly workspaceRoot: string,
    pathAlias: Readonly<Record<string, string>>,
    tsconfigPaths: TsconfigPathAliases | null = null,
    private readonly system: TsconfigPathAliasSystem = {
      fileExists: ts.sys.fileExists,
      readFile: ts.sys.readFile,
      readDirectory: ts.sys.readDirectory,
      useCaseSensitiveFileNames: ts.sys.useCaseSensitiveFileNames,
      onUnRecoverableConfigFileDiagnostic: () => {},
    },
    bundlerAliases: BundlerPathAliases | null = null,
  ) {
    this.settingsEntries = Object.entries(pathAlias)
      .map<AliasEntry>(([pattern, target]) => ({
        kind: "prefix",
        source: "settings",
        pattern,
        target: this.substituteWorkspace(target),
      }))
      .toSorted(compareAliasEntries);
    this.bundlerEntries = bundlerAliases
      ? Object.entries(bundlerAliases.aliases)
          .map<AliasEntry>(([pattern, target]) => ({
            kind: "prefix",
            source: "bundler",
            pattern,
            target: this.resolveBundlerTarget(target),
          }))
          .toSorted(compareAliasEntries)
      : [];
    this.defaultTsconfigEntries = tsconfigPaths ? this.buildTsconfigEntries(tsconfigPaths) : [];
  }

  /**
   * Try to resolve a non-relative import specifier to an absolute
   * path. Returns `null` if no alias matches. Callers still fall
   * back to relative resolution before accepting the specifier as
   * unresolved.
   */
  resolve(
    specifier: string,
    fileExists?: (candidate: string) => boolean,
    containingFilePath?: string,
  ): string | null {
    const resolvedFromSettings = this.resolveFromEntries(
      this.settingsEntries,
      specifier,
      fileExists,
    );
    if (resolvedFromSettings) return resolvedFromSettings;
    const resolvedFromBundler = this.resolveFromEntries(this.bundlerEntries, specifier, fileExists);
    if (resolvedFromBundler) return resolvedFromBundler;
    return this.resolveFromEntries(
      this.resolveTsconfigEntries(containingFilePath),
      specifier,
      fileExists,
    );
  }

  private resolveTsconfigEntries(containingFilePath: string | undefined): readonly AliasEntry[] {
    if (!containingFilePath) return this.defaultTsconfigEntries;
    const configPath = findNearestConfigPath(containingFilePath, this.workspaceRoot, this.system);
    if (!configPath) return this.defaultTsconfigEntries;
    const cached = this.tsconfigEntriesByConfigPath.get(configPath);
    if (cached) return cached;
    const tsconfigPaths = loadTsconfigPathAliasesFromConfig(configPath, this.system);
    const entries = tsconfigPaths ? this.buildTsconfigEntries(tsconfigPaths) : [];
    this.tsconfigEntriesByConfigPath.set(configPath, entries);
    return entries;
  }

  private resolveFromEntries(
    entries: readonly AliasEntry[],
    specifier: string,
    fileExists?: (candidate: string) => boolean,
  ): string | null {
    for (const entry of entries) {
      if (entry.kind === "prefix") {
        if (specifier === entry.pattern) return entry.target;
        const normalised = entry.pattern.endsWith("/") ? entry.pattern : entry.pattern + "/";
        if (specifier.startsWith(normalised)) {
          return path.resolve(entry.target, specifier.slice(normalised.length));
        }
        continue;
      }

      if (!specifier.startsWith(entry.prefix) || !specifier.endsWith(entry.suffix)) {
        continue;
      }
      const matched = specifier.slice(entry.prefix.length, specifier.length - entry.suffix.length);
      const candidates = entry.targets.map((targetPattern) =>
        this.resolveTsconfigTarget(targetPattern, matched),
      );
      if (!fileExists) return candidates[0] ?? null;
      const existing = candidates.find((candidate) => fileExists(candidate));
      return existing ?? candidates[0] ?? null;
    }
    return null;
  }

  private buildTsconfigEntries(tsconfigPaths: TsconfigPathAliases): readonly AliasEntry[] {
    const entries: AliasEntry[] = [];
    for (const [pattern, targets] of Object.entries(tsconfigPaths.paths)) {
      if (targets.length === 0) continue;
      const wildcardIndex = pattern.indexOf("*");
      if (wildcardIndex === -1) {
        for (const target of targets) {
          entries.push({
            kind: "prefix",
            source: "tsconfig",
            pattern,
            target: this.resolveTsconfigBaseTarget(tsconfigPaths.basePath, target),
          });
        }
        continue;
      }
      entries.push({
        kind: "wildcard",
        source: "tsconfig",
        pattern,
        prefix: pattern.slice(0, wildcardIndex),
        suffix: pattern.slice(wildcardIndex + 1),
        targets: targets.map((target) =>
          this.resolveTsconfigBaseTarget(tsconfigPaths.basePath, target),
        ),
      });
    }
    return entries;
  }

  private resolveTsconfigBaseTarget(basePath: string, target: string): string {
    return path.isAbsolute(target) ? target : path.resolve(basePath, target);
  }

  private resolveTsconfigTarget(targetPattern: string, matched: string): string {
    return targetPattern.includes("*") ? targetPattern.replaceAll("*", matched) : targetPattern;
  }

  private substituteWorkspace(target: string): string {
    // `${workspaceFolder}` is substituted at construction time
    // because workspaceRoot is immutable for the server's lifetime.
    const replaced = target.replace("${workspaceFolder}", this.workspaceRoot);
    return path.isAbsolute(replaced) ? replaced : path.resolve(this.workspaceRoot, replaced);
  }

  private resolveBundlerTarget(target: string): string {
    return path.isAbsolute(target) ? target : path.resolve(this.workspaceRoot, target);
  }
}

export { loadWorkspaceBundlerPathAliases };
export type { BundlerPathAliases };
