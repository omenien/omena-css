import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import {
  API,
  TypeFlags,
  type APIOptions,
  type LiteralType,
  type Type,
  type UnionType,
} from "@typescript/native-preview/unstable/async";
import ts from "../../engine-core-ts/src/ts-facade";
import type { ResolvedType } from "@omena/shared";
import {
  createTypeFactTableEntryV1,
  createTypeFactTableEntryV2,
  type TypeFactTableV1,
  type TypeFactTableV2,
} from "../../engine-core-ts/src/contracts";
import type { CollectTypeFactTableV1Options } from "./historical/type-fact-table-v1";
import { typeFactControlFlowGraphForSymbolExpression } from "./type-fact-control-flow-graph";
import { resolveTsgoBinaryPathForEnv } from "./tsgo-probe-type-resolver";

const UNRESOLVABLE: ResolvedType = { kind: "unresolvable", values: [] };

export interface TsgoTypeFactTarget {
  readonly filePath: string;
  readonly expressionId: string;
  readonly position: number;
}

export interface TsgoTypeFactWorkerInput {
  readonly workspaceRoot: string;
  readonly configPath: string;
  readonly targets: readonly TsgoTypeFactTarget[];
}

export interface TsgoTypeFactWorkerResultEntry {
  readonly filePath: string;
  readonly expressionId: string;
  readonly resolvedType: ResolvedType;
}

export type RunTsgoTypeFactWorker = (
  input: TsgoTypeFactWorkerInput,
) => Promise<readonly TsgoTypeFactWorkerResultEntry[]>;

export interface TsgoTypeFactResolvedTypesCache {
  get(key: string): Map<string, ResolvedType> | undefined;
  set(key: string, resolvedTypes: Map<string, ResolvedType>): void;
  clear(): void;
}

interface TsgoTypeFactResolvedTypesCacheEntry {
  readonly expiresAt: number;
  readonly resolvedTypes: Map<string, ResolvedType>;
}

export interface TsgoTypeFactApiOptions {
  readonly cwd: string;
  readonly tsserverPath?: string;
}

export interface CollectTsgoTypeFactsOptions extends CollectTypeFactTableV1Options {
  readonly findConfigFile?: (workspaceRoot: string) => string | null;
  readonly runWorker?: RunTsgoTypeFactWorker;
  readonly workerCache?: TsgoTypeFactResolvedTypesCache;
}

export function collectTypeFactTableV1WithTsgo(
  options: CollectTsgoTypeFactsOptions,
): Promise<TypeFactTableV1> {
  return collectTsgoResolvedTypes(options).then((resolvedTypes) =>
    buildTypeFactTableV1(options, resolvedTypes),
  );
}

export async function collectTypeFactTableV2WithTsgo(
  options: CollectTsgoTypeFactsOptions,
): Promise<TypeFactTableV2> {
  return buildTypeFactTableV2(options, await collectTsgoResolvedTypes(options));
}

async function collectTsgoResolvedTypes(
  options: CollectTsgoTypeFactsOptions,
): Promise<Map<string, ResolvedType>> {
  let resolvedTypes: Map<string, ResolvedType> | null;
  try {
    resolvedTypes = await collectTsgoResolvedTypesUnchecked(options);
  } catch (error) {
    if (!isRecoverableTsgoWorkerError(error)) {
      throw error;
    }
    resolvedTypes = new Map();
  }
  if (!resolvedTypes) {
    resolvedTypes = new Map();
  }

  return resolvedTypes ?? new Map();
}

function buildTypeFactTableV1(
  options: CollectTsgoTypeFactsOptions,
  resolvedTypes: Map<string, ResolvedType>,
): TypeFactTableV1 {
  return options.sourceEntries
    .flatMap(({ document, analysis }) =>
      analysis.sourceDocument.classExpressions.flatMap((expression) => {
        if (expression.kind !== "symbolRef") return [];
        return [
          createTypeFactTableEntryV1(
            document.filePath,
            expression.id,
            resolvedTypes.get(typeFactKey(document.filePath, expression.id)) ?? UNRESOLVABLE,
          ),
        ];
      }),
    )
    .toSorted(
      (a, b) =>
        a.filePath.localeCompare(b.filePath) || a.expressionId.localeCompare(b.expressionId),
    );
}

function buildTypeFactTableV2(
  options: CollectTsgoTypeFactsOptions,
  resolvedTypes: Map<string, ResolvedType>,
): TypeFactTableV2 {
  return options.sourceEntries
    .flatMap(({ document, analysis }) =>
      analysis.sourceDocument.classExpressions.flatMap((expression) => {
        if (expression.kind !== "symbolRef") return [];
        return [
          createTypeFactTableEntryV2(
            document.filePath,
            expression.id,
            resolvedTypes.get(typeFactKey(document.filePath, expression.id)) ?? UNRESOLVABLE,
            typeFactControlFlowGraphForSymbolExpression(analysis.sourceFile, expression),
          ),
        ];
      }),
    )
    .toSorted(
      (a, b) =>
        a.filePath.localeCompare(b.filePath) || a.expressionId.localeCompare(b.expressionId),
    );
}

async function collectTsgoResolvedTypesUnchecked(
  options: CollectTsgoTypeFactsOptions,
): Promise<Map<string, ResolvedType> | null> {
  const findConfigFile =
    options.findConfigFile ??
    ((workspaceRoot: string) => ts.findConfigFile(workspaceRoot, ts.sys.fileExists) ?? null);
  const configPath = findConfigFile(options.workspaceRoot);
  if (!configPath) {
    return null;
  }

  const targets = options.sourceEntries.flatMap(({ document, analysis }) =>
    analysis.sourceDocument.classExpressions.flatMap((expression) => {
      if (expression.kind !== "symbolRef") return [];
      return [
        {
          filePath: document.filePath,
          expressionId: expression.id,
          position: offsetAtPosition(
            document.content,
            expression.range.start.line,
            expression.range.start.character,
          ),
        } satisfies TsgoTypeFactTarget,
      ];
    }),
  );

  if (targets.length === 0) {
    return new Map();
  }

  const runWorker = options.runWorker ?? defaultRunTsgoTypeFactWorker;
  const workerCache = options.workerCache ?? (options.runWorker ? null : defaultResolvedTypesCache);
  const cacheKey = createTsgoResolvedTypesCacheKey(
    options.workspaceRoot,
    configPath,
    options.sourceEntries,
    targets,
  );
  const cachedResolvedTypes = workerCache?.get(cacheKey);
  if (cachedResolvedTypes) {
    return cachedResolvedTypes;
  }

  const resolved = await runWorker({
    workspaceRoot: options.workspaceRoot,
    configPath,
    targets,
  });
  const resolvedTypes = new Map(
    resolved.map((entry) => [typeFactKey(entry.filePath, entry.expressionId), entry.resolvedType]),
  );
  workerCache?.set(cacheKey, resolvedTypes);
  return resolvedTypes;
}

async function defaultRunTsgoTypeFactWorker(
  input: TsgoTypeFactWorkerInput,
): Promise<readonly TsgoTypeFactWorkerResultEntry[]> {
  const api = new API(buildTsgoTypeFactApiOptions(input.workspaceRoot));
  let snapshot: Awaited<ReturnType<API["updateSnapshot"]>> | undefined;
  try {
    snapshot = await api.updateSnapshot({ openProject: input.configPath });
    const projectByFile = new Map(
      await Promise.all(
        [...new Set(input.targets.map((target) => target.filePath))].map(async (filePath) => {
          const project = await snapshot?.getDefaultProjectForFile(filePath);
          if (!project) {
            throw new Error(`no project found for file ${filePath}`);
          }
          return [filePath, project] as const;
        }),
      ),
    );

    const results = await Promise.all(
      input.targets.map(async (target) => {
        const project = projectByFile.get(target.filePath);
        const type = project
          ? await project.checker.getTypeAtPosition(target.filePath, target.position)
          : undefined;
        return {
          filePath: target.filePath,
          expressionId: target.expressionId,
          resolvedType: await extractResolvedType(type),
        };
      }),
    );
    return results;
  } catch (error) {
    throw normalizeTsgoApiError(error);
  } finally {
    try {
      await snapshot?.dispose();
    } finally {
      await closeTsgoTypeFactApi(api);
    }
  }
}

export function buildTsgoTypeFactApiOptions(
  workspaceRoot: string,
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): APIOptions {
  const tsgoPath = resolveTsgoBinaryPathForEnv(env, fileExists);
  const options: TsgoTypeFactApiOptions = { cwd: workspaceRoot };
  if (env.OMENA_TSGO_PATH || fileExists(tsgoPath)) {
    return { ...options, tsserverPath: tsgoPath };
  }
  return options;
}

function normalizeTsgoApiError(error: unknown): Error {
  if (error instanceof Error) {
    if (isTsgoProjectMissError(error)) {
      return error;
    }
    return new Error(`tsgo type fact worker failed\nstderr: ${error.message}`);
  }
  return new Error(`tsgo type fact worker failed\nstderr: ${String(error)}`);
}

async function closeTsgoTypeFactApi(api: API): Promise<void> {
  await api.close();
}

function isTsgoProjectMissError(error: unknown): boolean {
  return error instanceof Error && /\bno project found for file\b/u.test(error.message);
}

function isRecoverableTsgoWorkerError(error: unknown): boolean {
  if (isTsgoProjectMissError(error)) {
    return true;
  }
  if (!(error instanceof Error)) {
    return false;
  }
  return /\b(tsgo type fact worker failed|No extension-owned tsgo binary|ENOENT|spawn)\b/u.test(
    error.message,
  );
}

function offsetAtPosition(text: string, line: number, character: number): number {
  let offset = 0;
  let currentLine = 0;

  while (currentLine < line && offset < text.length) {
    const newline = text.indexOf("\n", offset);
    if (newline < 0) {
      return text.length;
    }
    offset = newline + 1;
    currentLine += 1;
  }

  return offset + character;
}

function typeFactKey(filePath: string, expressionId: string): string {
  return `${filePath}::${expressionId}`;
}

export function createTsgoTypeFactResolvedTypesCache(
  maxEntries = 64,
  maxAgeMs = 1_000,
  now: () => number = Date.now,
): TsgoTypeFactResolvedTypesCache {
  const entries = new Map<string, TsgoTypeFactResolvedTypesCacheEntry>();

  return {
    get(key) {
      const entry = entries.get(key);
      if (!entry) return undefined;
      if (entry.expiresAt <= now()) {
        entries.delete(key);
        return undefined;
      }
      entries.delete(key);
      entries.set(key, entry);
      return cloneResolvedTypes(entry.resolvedTypes);
    },
    set(key, resolvedTypes) {
      entries.delete(key);
      entries.set(key, {
        expiresAt: now() + maxAgeMs,
        resolvedTypes: cloneResolvedTypes(resolvedTypes),
      });
      while (entries.size > maxEntries) {
        const oldestKey = entries.keys().next().value as string | undefined;
        if (oldestKey === undefined) break;
        entries.delete(oldestKey);
      }
    },
    clear() {
      entries.clear();
    },
  };
}

function createTsgoResolvedTypesCacheKey(
  workspaceRoot: string,
  configPath: string,
  sourceEntries: CollectTsgoTypeFactsOptions["sourceEntries"],
  targets: readonly TsgoTypeFactTarget[],
): string {
  const sourceSignature = sourceEntries
    .map(({ document, analysis }) => ({
      filePath: document.filePath,
      version: document.version,
      contentHash: analysis.contentHash,
    }))
    .toSorted((a, b) => a.filePath.localeCompare(b.filePath))
    .map(({ filePath, version, contentHash }) => `${filePath}:${version}:${contentHash}`)
    .join("|");
  const targetSignature = targets
    .map(({ filePath, expressionId, position }) => `${filePath}:${expressionId}:${position}`)
    .toSorted()
    .join("|");

  return JSON.stringify({
    workspaceRoot,
    configPath,
    configHash: readFileContentHash(configPath),
    sources: sourceSignature,
    targets: targetSignature,
    workerEnv: readTsgoTypeFactWorkerEnvSignature(process.env),
  });
}

function readFileContentHash(filePath: string): string {
  try {
    return createHash("sha256").update(readFileSync(filePath)).digest("hex");
  } catch {
    return "unreadable";
  }
}

function readTsgoTypeFactWorkerEnvSignature(env: NodeJS.ProcessEnv): string {
  return JSON.stringify({
    projectRoot: env.OMENA_PROJECT_ROOT ?? "",
    tsgoCheckers: env.OMENA_TSGO_CHECKERS ?? "",
    tsgoPath: env.OMENA_TSGO_PATH ?? "",
  });
}

function cloneResolvedTypes(resolvedTypes: Map<string, ResolvedType>): Map<string, ResolvedType> {
  return new Map(
    [...resolvedTypes.entries()].map(([key, resolvedType]) => [
      key,
      cloneResolvedType(resolvedType),
    ]),
  );
}

const defaultResolvedTypesCache = createTsgoTypeFactResolvedTypesCache();

function cloneResolvedType(resolvedType: ResolvedType): ResolvedType {
  if (resolvedType.kind === "union") {
    return { kind: "union", values: [...resolvedType.values] };
  }
  return UNRESOLVABLE;
}

async function extractResolvedType(type: Type | undefined): Promise<ResolvedType> {
  if (!type) {
    return UNRESOLVABLE;
  }
  if (isStringLiteralType(type)) {
    return { kind: "union", values: [(type as LiteralType).value as string] };
  }
  if ((type.flags & TypeFlags.Union) !== 0) {
    const members = await (type as UnionType).getTypes();
    const resolvedMembers = await Promise.all(members.map((member) => extractResolvedType(member)));
    const values: string[] = [];
    for (const resolved of resolvedMembers) {
      if (resolved.kind !== "union" || resolved.values.length !== 1) {
        return UNRESOLVABLE;
      }
      const value = resolved.values[0];
      if (value === undefined) {
        return UNRESOLVABLE;
      }
      values.push(value);
    }
    const deduped = [...new Set(values)];
    return deduped.length > 0 ? { kind: "union", values: deduped } : UNRESOLVABLE;
  }
  return UNRESOLVABLE;
}

function isStringLiteralType(type: Type): boolean {
  return (
    (type.flags & TypeFlags.StringLiteral) !== 0 && typeof (type as LiteralType).value === "string"
  );
}
