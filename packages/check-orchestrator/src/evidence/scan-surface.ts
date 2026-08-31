import {
  execFileSync as nodeExecFileSync,
  spawnSync as nodeSpawnSync,
  type ExecFileSyncOptionsWithStringEncoding,
  type SpawnSyncOptionsWithStringEncoding,
  type SpawnSyncReturns,
} from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readdirSync, realpathSync, statSync, type Dirent } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { SCAN_SURFACE_EXCLUDE_PREDICATES } from "./predicates/index.ts";

export const SCAN_SURFACE_MODES = ["index", "workingTree"] as const;

export type ScanSurfaceMode = (typeof SCAN_SURFACE_MODES)[number];

export const NAMED_SCAN_PREDICATE_IDS = [
  "git-metadata",
  "node-modules",
  "personal-docs",
  "rust-build-output",
  "test-only-rust",
] as const;

export type NamedScanPredicateId = (typeof NAMED_SCAN_PREDICATE_IDS)[number];

export interface ScanSurfaceSpec {
  readonly scannerPath: string;
  readonly mode: ScanSurfaceMode;
  readonly pathspecs: readonly string[];
  readonly includeUntracked: boolean;
  readonly excludes: readonly NamedScanPredicateId[];
}

export const UNMIGRATED_SCAN_REASONS = [
  "external-checkout",
  "outside-repoRoot-packaged-binary-or-node_modules-tree",
  "non-repo-temp-tree",
] as const;

export type UnmigratedScanReason = (typeof UNMIGRATED_SCAN_REASONS)[number];

export interface MigratedScanSurfaceDeclaration {
  readonly disposition: "MIGRATED";
  readonly spec: ScanSurfaceSpec;
  readonly narrowingReason?: string;
  readonly renamedFrom?: string;
}

export interface UnmigratedScanSurfaceDeclaration {
  readonly disposition: "UNMIGRATED";
  readonly scannerPath: string;
  readonly reason: UnmigratedScanReason;
  readonly inRepoSpec?: ScanSurfaceSpec;
}

export interface FalsePositiveScanSurfaceDeclaration {
  readonly disposition: "FALSE-POSITIVE";
  readonly scannerPath: string;
  readonly rationale: string;
}

export interface RetiredScanSurfaceDeclaration {
  readonly disposition: "RETIRED";
  readonly scannerPath: string;
  readonly reason: string;
}

export type ScanSurfaceDeclaration =
  | MigratedScanSurfaceDeclaration
  | UnmigratedScanSurfaceDeclaration
  | FalsePositiveScanSurfaceDeclaration
  | RetiredScanSurfaceDeclaration;

export interface ResolvedScanSurface {
  readonly spec: ScanSurfaceSpec;
  readonly paths: readonly string[];
  readonly trackedPaths: readonly string[];
  readonly untrackedPaths: readonly string[];
  readdirSync(
    directory: string,
    options?:
      | BufferEncoding
      | null
      | {
          readonly encoding?: BufferEncoding | null;
          readonly withFileTypes?: false;
          readonly recursive?: boolean;
        },
  ): string[];
  readdirSync(
    directory: string,
    options: {
      readonly encoding?: BufferEncoding | null;
      readonly withFileTypes: true;
      readonly recursive?: boolean;
    },
  ): Dirent<string>[];
  gitOutput(args: readonly string[]): string;
  execFileSync(
    command: "git",
    args: readonly string[],
    options: ExecFileSyncOptionsWithStringEncoding,
  ): string;
  spawnSync(
    command: "git",
    args: readonly string[],
    options: SpawnSyncOptionsWithStringEncoding,
  ): SpawnSyncReturns<string>;
  globSync(patterns?: string | readonly string[], options?: SurfaceGlobOptions): string[];
}

export interface SurfaceGlobOptions {
  readonly absolute?: boolean;
  readonly cwd?: string;
  readonly onlyFiles?: boolean;
}

export interface ResolveScanSurfaceOptions {
  readonly repoRoot: string;
  readonly excludePredicates?: Partial<
    Readonly<Record<NamedScanPredicateId, (candidate: string) => boolean>>
  >;
}

const gitLineSnapshotCache = new Map<string, readonly string[]>();
const workingTreeSnapshotCache = new Map<string, readonly string[]>();

export function defineScanSurface<const T extends ScanSurfaceSpec>(spec: T): T {
  validateScanSurfaceSpec(spec);
  return spec;
}

export function migratedScanSurface(
  spec: ScanSurfaceSpec,
  metadata: {
    readonly narrowingReason?: string;
    readonly renamedFrom?: string;
  } = {},
): MigratedScanSurfaceDeclaration {
  return { disposition: "MIGRATED", spec: defineScanSurface(spec), ...metadata };
}

export function unmigratedScanSurface(
  declaration: Omit<UnmigratedScanSurfaceDeclaration, "disposition">,
): UnmigratedScanSurfaceDeclaration {
  if (!UNMIGRATED_SCAN_REASONS.includes(declaration.reason)) {
    throw new Error(`unsupported unmigrated scan reason: ${declaration.reason}`);
  }
  if (declaration.inRepoSpec) {
    validateScanSurfaceSpec(declaration.inRepoSpec);
    if (declaration.inRepoSpec.scannerPath !== declaration.scannerPath) {
      throw new Error(`unmigrated in-repo surface path mismatch: ${declaration.scannerPath}`);
    }
  }
  return { disposition: "UNMIGRATED", ...declaration };
}

export function assertUnmigratedScanRoot(
  reason: UnmigratedScanReason,
  repoRoot: string,
  scanRoot: string,
): string {
  if (!UNMIGRATED_SCAN_REASONS.includes(reason)) {
    throw new Error(`unsupported unmigrated scan reason: ${reason}`);
  }
  const canonicalRepoRoot = realpathSync.native(repoRoot);
  const canonicalScanRoot = realpathSync.native(scanRoot);
  if (isPathWithin(canonicalRepoRoot, canonicalScanRoot)) {
    throw new Error(`unmigrated ${reason} root must be outside repoRoot: ${canonicalScanRoot}`);
  }
  if (reason === "non-repo-temp-tree") {
    const canonicalTempRoot = realpathSync.native(tmpdir());
    if (!isPathWithin(canonicalTempRoot, canonicalScanRoot)) {
      throw new Error(`unmigrated temp root is outside the system temp tree: ${canonicalScanRoot}`);
    }
  } else if (reason === "external-checkout") {
    if (!existsSync(path.join(canonicalScanRoot, ".git"))) {
      throw new Error(`unmigrated external checkout root lacks .git: ${canonicalScanRoot}`);
    }
  } else if (!canonicalScanRoot.split(path.sep).includes("node_modules")) {
    throw new Error(
      `unmigrated packaged tree root is not inside node_modules: ${canonicalScanRoot}`,
    );
  }
  return canonicalScanRoot;
}

function isPathWithin(parent: string, candidate: string): boolean {
  const relative = path.relative(parent, candidate);
  return relative === "" || (!relative.startsWith(`..${path.sep}`) && relative !== "..");
}

export function falsePositiveScanSurface(
  declaration: Omit<FalsePositiveScanSurfaceDeclaration, "disposition">,
): FalsePositiveScanSurfaceDeclaration {
  if (!declaration.rationale) {
    throw new Error(`false-positive scanner ${declaration.scannerPath} needs a rationale`);
  }
  return { disposition: "FALSE-POSITIVE", ...declaration };
}

export function retiredScanSurface(
  declaration: Omit<RetiredScanSurfaceDeclaration, "disposition">,
): RetiredScanSurfaceDeclaration {
  if (!declaration.reason) {
    throw new Error(`retired scanner ${declaration.scannerPath} needs a reason`);
  }
  return { disposition: "RETIRED", ...declaration };
}

export function validateScanSurfaceSpec(spec: ScanSurfaceSpec): void {
  if (normalizeRepoPath(spec.scannerPath) !== spec.scannerPath || !spec.scannerPath) {
    throw new Error(`scan surface scannerPath must be normalized: ${spec.scannerPath}`);
  }
  if (!SCAN_SURFACE_MODES.includes(spec.mode)) {
    throw new Error(`unsupported scan surface mode: ${spec.mode}`);
  }
  if (spec.pathspecs.length === 0 || spec.pathspecs.some((entry) => entry.length === 0)) {
    throw new Error(`scan surface ${spec.scannerPath} must declare at least one pathspec`);
  }
  if (spec.mode === "workingTree") {
    if (spec.includeUntracked) {
      throw new Error(
        `workingTree scan surface ${spec.scannerPath} cannot set includeUntracked=true`,
      );
    }
    const magicPathspec = spec.pathspecs.find((entry) => entry.startsWith(":("));
    if (magicPathspec) {
      throw new Error(
        `workingTree scan surface ${spec.scannerPath} cannot use git magic pathspec ${magicPathspec}`,
      );
    }
  }
  for (const predicateId of spec.excludes) {
    if (!NAMED_SCAN_PREDICATE_IDS.includes(predicateId)) {
      throw new Error(
        `scan surface ${spec.scannerPath} uses unknown exclude predicate ${predicateId}`,
      );
    }
  }
}

export function resolveScanSurface(
  spec: ScanSurfaceSpec,
  options: ResolveScanSurfaceOptions,
): ResolvedScanSurface {
  validateScanSurfaceSpec(spec);
  const repoRoot = realpathSync.native(path.resolve(options.repoRoot));
  const excludePredicates = options.excludePredicates ?? {};
  const resolved =
    spec.mode === "index"
      ? resolveIndexSurface(repoRoot, spec)
      : resolveWorkingTreeSurface(repoRoot, spec, excludePredicates);
  const trackedPaths = applyExcludes(resolved.trackedPaths, spec.excludes, excludePredicates);
  const untrackedPaths = applyExcludes(resolved.untrackedPaths, spec.excludes, excludePredicates);
  return {
    spec,
    paths: [...new Set([...trackedPaths, ...untrackedPaths])].toSorted(),
    trackedPaths,
    untrackedPaths,
    ...createSurfaceReaders(repoRoot, spec, trackedPaths, untrackedPaths),
  };
}

export function clearScanSurfaceResolverSnapshot(): void {
  gitLineSnapshotCache.clear();
  workingTreeSnapshotCache.clear();
}

export function scanSurfaceSpecDigest(spec: ScanSurfaceSpec): string {
  return createHash("sha256")
    .update(`${stableSpecJson(spec)}\n`)
    .digest("hex");
}

export function scanSurfaceMatchesPath(spec: ScanSurfaceSpec, candidatePath: string): boolean {
  validateScanSurfaceSpec(spec);
  const candidate = normalizeRepoPath(candidatePath);
  if (spec.excludes.some((predicateId) => excludesPath(predicateId, candidate))) {
    return false;
  }
  return spec.pathspecs.some((pathspec) => {
    const explicitGlob = pathspec.startsWith(":(glob)");
    const normalized = normalizeRepoPath(explicitGlob ? pathspec.slice(7) : pathspec);
    if (spec.mode === "index" && !explicitGlob && !normalized.includes("/")) {
      return compileWorkingTreePathspec(normalized)(path.posix.basename(candidate));
    }
    return compileWorkingTreePathspec(normalized)(candidate);
  });
}

export function stableSpecJson(spec: ScanSurfaceSpec): string {
  return JSON.stringify({
    scannerPath: spec.scannerPath,
    mode: spec.mode,
    pathspecs: [...spec.pathspecs],
    includeUntracked: spec.includeUntracked,
    excludes: [...spec.excludes],
  });
}

function resolveIndexSurface(
  repoRoot: string,
  spec: ScanSurfaceSpec,
): { readonly trackedPaths: readonly string[]; readonly untrackedPaths: readonly string[] } {
  const trackedPaths = runGitLines(repoRoot, ["ls-files", "-z", "--", ...spec.pathspecs]);
  const untrackedPaths = spec.includeUntracked
    ? runGitLines(repoRoot, [
        "ls-files",
        "-z",
        "--others",
        "--exclude-standard",
        "--",
        ...spec.pathspecs,
      ])
    : [];
  return { trackedPaths, untrackedPaths };
}

function resolveWorkingTreeSurface(
  repoRoot: string,
  spec: ScanSurfaceSpec,
  excludePredicates: Partial<
    Readonly<Record<NamedScanPredicateId, (candidate: string) => boolean>>
  >,
): { readonly trackedPaths: readonly string[]; readonly untrackedPaths: readonly string[] } {
  const matchers = spec.pathspecs.map(compileWorkingTreePathspec);
  const usesPredicateOverrides = Object.keys(excludePredicates).length > 0;
  const snapshotKey = `${repoRoot}\0${spec.excludes.join("\0")}`;
  let snapshot = usesPredicateOverrides ? undefined : workingTreeSnapshotCache.get(snapshotKey);
  if (!snapshot) {
    const paths: string[] = [];
    const directories = [repoRoot];
    while (directories.length > 0) {
      const directory = directories.pop();
      if (!directory) continue;
      for (const entry of readdirSync(directory, { withFileTypes: true })) {
        const absolutePath = path.join(directory, entry.name);
        const relativePath = normalizeRepoPath(path.relative(repoRoot, absolutePath));
        if (relativePath === ".git" || relativePath.startsWith(".git/")) continue;
        if (entry.isDirectory()) {
          if (
            spec.excludes.some((predicateId) =>
              excludesPath(predicateId, relativePath, excludePredicates),
            )
          ) {
            continue;
          }
          directories.push(absolutePath);
        } else if (entry.isFile()) {
          paths.push(relativePath);
        }
      }
    }
    snapshot = paths.toSorted();
    if (!usesPredicateOverrides) workingTreeSnapshotCache.set(snapshotKey, snapshot);
  }
  return {
    trackedPaths: snapshot.filter((candidate) => matchers.some((matches) => matches(candidate))),
    untrackedPaths: [],
  };
}

function runGitLines(repoRoot: string, args: readonly string[]): readonly string[] {
  const cacheKey = `${repoRoot}\0${args.join("\0")}`;
  const cached = gitLineSnapshotCache.get(cacheKey);
  if (cached) return cached;
  const result = nodeSpawnSync("git", args, {
    cwd: repoRoot,
    encoding: "utf8",
    shell: false,
    maxBuffer: 64 * 1024 * 1024,
  });
  if (result.error) {
    throw new Error(`failed to start git for scan surface: ${result.error.message}`);
  }
  if ((result.status ?? 1) !== 0) {
    throw new Error(
      String(result.stderr).trim() || `git ${args.join(" ")} failed for scan surface`,
    );
  }
  const lines = String(result.stdout).split("\0").map(normalizeRepoPath).filter(Boolean).toSorted();
  gitLineSnapshotCache.set(cacheKey, lines);
  return lines;
}

function createSurfaceReaders(
  repoRoot: string,
  spec: ScanSurfaceSpec,
  trackedPaths: readonly string[],
  untrackedPaths: readonly string[],
): Pick<
  ResolvedScanSurface,
  "readdirSync" | "gitOutput" | "execFileSync" | "spawnSync" | "globSync"
> {
  const paths = [...new Set([...trackedPaths, ...untrackedPaths])].toSorted();
  const allowedPathSet = new Set(paths);
  const allowedDirectorySet = new Set<string>();
  for (const candidate of paths) {
    let directory = path.posix.dirname(candidate);
    while (directory !== ".") {
      allowedDirectorySet.add(directory);
      const parent = path.posix.dirname(directory);
      if (parent === directory) break;
      directory = parent;
    }
  }

  function surfaceReaddirSync(
    directory: string,
    options?:
      | BufferEncoding
      | null
      | {
          readonly encoding?: BufferEncoding | null;
          readonly withFileTypes?: boolean;
          readonly recursive?: boolean;
        },
  ): string[] | Dirent<string>[] {
    const absoluteDirectory = realpathSync.native(path.resolve(directory));
    const relativeDirectory = normalizeRepoPath(path.relative(repoRoot, absoluteDirectory));
    if (
      relativeDirectory === ".." ||
      relativeDirectory.startsWith("../") ||
      path.isAbsolute(relativeDirectory)
    ) {
      throw new Error(
        `scan surface ${spec.scannerPath} attempted to enumerate outside repoRoot: ${directory}`,
      );
    }
    if (!statSync(absoluteDirectory).isDirectory()) {
      throw new Error(`scan surface directory is not a directory: ${directory}`);
    }
    const normalizedDirectory = relativeDirectory === "" ? "." : relativeDirectory;
    const prefix = normalizedDirectory === "." ? "" : `${normalizedDirectory}/`;
    const recursive = typeof options === "object" && options !== null && options.recursive === true;
    const withFileTypes =
      typeof options === "object" && options !== null && options.withFileTypes === true;
    const names = new Map<string, { readonly path: string; readonly directory: boolean }>();
    for (const candidate of [...allowedPathSet, ...allowedDirectorySet]) {
      if (!candidate.startsWith(prefix)) continue;
      const remainder = candidate.slice(prefix.length);
      if (!remainder) continue;
      const name = recursive ? remainder : remainder.split("/")[0];
      if (!name) continue;
      const childPath = prefix ? `${prefix}${name}` : name;
      names.set(name, {
        path: childPath,
        directory: allowedDirectorySet.has(childPath),
      });
    }
    const entries = [...names.entries()].toSorted(([left], [right]) => compareText(left, right));
    if (!withFileTypes) return entries.map(([name]) => name);
    return entries.map(
      ([name, entry]) =>
        new SurfaceDirent(name, absoluteDirectory, entry.directory, allowedPathSet.has(entry.path)),
    );
  }

  function gitOutput(args: readonly string[]): string {
    if (args[0] !== "ls-files") {
      throw new Error(
        `scan surface ${spec.scannerPath} only routes git ls-files, got ${args.join(" ")}`,
      );
    }
    const result = nodeSpawnSync("git", args, {
      cwd: repoRoot,
      encoding: "utf8",
      shell: false,
      maxBuffer: 64 * 1024 * 1024,
    });
    if (result.error) throw new Error(`failed to start git: ${result.error.message}`);
    if ((result.status ?? 1) !== 0) {
      throw new Error(String(result.stderr).trim() || `git ${args.join(" ")} failed`);
    }
    const output = String(result.stdout);
    const delimiter = args.includes("-z") ? "\0" : /\r?\n/u;
    const observed = output.split(delimiter).map(normalizeRepoPath).filter(Boolean);
    const outside = observed.find((candidate) => !allowedPathSet.has(candidate));
    if (outside) {
      throw new Error(
        `scan surface ${spec.scannerPath} read undeclared path through git ls-files: ${outside}`,
      );
    }
    return output;
  }

  function execFileSync(
    command: "git",
    args: readonly string[],
    options: ExecFileSyncOptionsWithStringEncoding,
  ): string {
    requireGitLsFiles(command, args);
    const output = nodeExecFileSync(command, args, { ...options, cwd: repoRoot });
    verifyGitOutput(args, output);
    return output;
  }

  function spawnSync(
    command: "git",
    args: readonly string[],
    options: SpawnSyncOptionsWithStringEncoding,
  ): SpawnSyncReturns<string> {
    requireGitLsFiles(command, args);
    const result = nodeSpawnSync(command, args, { ...options, cwd: repoRoot });
    if ((result.status ?? 1) === 0) verifyGitOutput(args, result.stdout);
    return result;
  }

  function requireGitLsFiles(command: string, args: readonly string[]): void {
    if (command !== "git" || args[0] !== "ls-files") {
      throw new Error(
        `scan surface ${spec.scannerPath} only routes git ls-files, got ${command} ${args.join(" ")}`,
      );
    }
  }

  function verifyGitOutput(args: readonly string[], output: string): void {
    const delimiter = args.includes("-z") ? "\0" : /\r?\n/u;
    const outside = output
      .split(delimiter)
      .map(normalizeRepoPath)
      .filter(Boolean)
      .find((candidate) => !allowedPathSet.has(candidate));
    if (outside) {
      throw new Error(
        `scan surface ${spec.scannerPath} read undeclared path through git ls-files: ${outside}`,
      );
    }
  }

  function globSync(
    patterns: string | readonly string[] = spec.pathspecs,
    options: SurfaceGlobOptions = {},
  ): string[] {
    const cwd = path.resolve(options.cwd ?? repoRoot);
    const relativeCwd = normalizeRepoPath(path.relative(repoRoot, cwd));
    if (relativeCwd === ".." || relativeCwd.startsWith("../") || path.isAbsolute(relativeCwd)) {
      throw new Error(
        `scan surface ${spec.scannerPath} attempted to glob outside repoRoot: ${cwd}`,
      );
    }
    const matchers = (typeof patterns === "string" ? [patterns] : patterns).map(
      compileWorkingTreePathspec,
    );
    return paths
      .filter((candidate) => {
        const relative = relativeCwd ? path.posix.relative(relativeCwd, candidate) : candidate;
        return !relative.startsWith("../") && matchers.some((matches) => matches(relative));
      })
      .map((candidate) => (options.absolute ? path.join(repoRoot, candidate) : candidate));
  }

  return {
    readdirSync: surfaceReaddirSync as ResolvedScanSurface["readdirSync"],
    gitOutput,
    execFileSync,
    spawnSync,
    globSync,
  };
}

class SurfaceDirent implements Dirent<string> {
  readonly name: string;
  readonly parentPath: string;
  readonly path: string;
  private readonly directory: boolean;
  private readonly file: boolean;

  constructor(name: string, parentPath: string, directory: boolean, file: boolean) {
    this.name = name;
    this.parentPath = parentPath;
    this.path = parentPath;
    this.directory = directory;
    this.file = file;
  }

  isFile(): boolean {
    return this.file;
  }

  isDirectory(): boolean {
    return this.directory;
  }

  isBlockDevice(): boolean {
    return false;
  }

  isCharacterDevice(): boolean {
    return false;
  }

  isSymbolicLink(): boolean {
    return false;
  }

  isFIFO(): boolean {
    return false;
  }

  isSocket(): boolean {
    return false;
  }
}

function compileWorkingTreePathspec(pathspec: string): (candidate: string) => boolean {
  const normalized = normalizeRepoPath(pathspec);
  if (!/[?*]/u.test(normalized)) {
    const prefix = normalized.replace(/\/$/u, "");
    return (candidate) => candidate === prefix || candidate.startsWith(`${prefix}/`);
  }

  let source = "^";
  for (let index = 0; index < normalized.length; index += 1) {
    const character = normalized[index];
    const next = normalized[index + 1];
    if (character === "*" && next === "*") {
      if (normalized[index + 2] === "/") {
        source += "(?:.*/)?";
        index += 2;
      } else {
        source += ".*";
        index += 1;
      }
    } else if (character === "*") {
      source += "[^/]*";
    } else if (character === "?") {
      source += "[^/]";
    } else {
      source += escapeRegExp(character ?? "");
    }
  }
  const matcher = new RegExp(`${source}$`, "u");
  return (candidate) => matcher.test(candidate);
}

function applyExcludes(
  paths: readonly string[],
  predicateIds: readonly NamedScanPredicateId[],
  predicateOverrides: Partial<
    Readonly<Record<NamedScanPredicateId, (candidate: string) => boolean>>
  > = {},
): readonly string[] {
  return paths.filter(
    (candidate) =>
      !predicateIds.some((predicateId) => excludesPath(predicateId, candidate, predicateOverrides)),
  );
}

function excludesPath(
  predicateId: NamedScanPredicateId,
  candidate: string,
  predicateOverrides: Partial<
    Readonly<Record<NamedScanPredicateId, (candidate: string) => boolean>>
  > = {},
): boolean {
  return (predicateOverrides[predicateId] ?? SCAN_SURFACE_EXCLUDE_PREDICATES[predicateId])(
    candidate,
  );
}

function normalizeRepoPath(value: string): string {
  return value.trim().replaceAll("\\", "/").replace(/^\.\//u, "");
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
