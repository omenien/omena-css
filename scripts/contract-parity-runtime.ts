import path from "node:path";
import { pathToFileURL } from "node:url";
import {
  collectSourceDocuments,
  createWorkspaceAnalysisHost,
  createWorkspaceStyleHost,
  resolveWorkspaceCheckFiles,
} from "../server/engine-host-node/src/checker-host/workspace-check-support";
import { runWorkspaceCheckCommand } from "../server/engine-host-node/src/checker-host";
import { buildCheckerEngineParitySnapshotV1 } from "../server/engine-host-node/src/historical/engine-parity-v1";
import { buildCheckerEngineParitySnapshotV2 } from "../server/engine-host-node/src/engine-parity-v2";
import type { ContractParityEntry } from "./contract-parity-corpus-v1";

async function prepareContractParityContext(entry: ContractParityEntry) {
  const { sourceFiles, styleFiles } = await resolveWorkspaceCheckFiles({
    workspaceRoot: entry.workspace.workspaceRoot,
    ...(entry.workspace.sourceFilePaths
      ? { sourceFilePaths: entry.workspace.sourceFilePaths }
      : {}),
    ...(entry.workspace.styleFilePaths ? { styleFilePaths: entry.workspace.styleFilePaths } : {}),
  });

  const styleHost = createWorkspaceStyleHost({
    styleFiles,
    classnameTransform: entry.workspace.classnameTransform ?? "asIs",
  });
  styleHost.preloadStyleDocuments();
  const analysisHost = createWorkspaceAnalysisHost({
    workspaceRoot: entry.workspace.workspaceRoot,
    classnameTransform: entry.workspace.classnameTransform ?? "asIs",
    pathAlias: entry.workspace.pathAlias ?? {},
    styleDocumentForPath: styleHost.styleDocumentForPath,
  });
  const sourceDocuments = collectSourceDocuments(sourceFiles, analysisHost.analysisCache);
  const command = await runWorkspaceCheckCommand({
    workspace: entry.workspace,
    filters: entry.filters,
  });

  return {
    workspaceRoot: entry.workspace.workspaceRoot,
    classnameTransform: entry.workspace.classnameTransform ?? "asIs",
    pathAlias: entry.workspace.pathAlias ?? {},
    sourceDocuments,
    styleFiles,
    analysisCache: analysisHost.analysisCache,
    styleDocumentForPath: styleHost.styleDocumentForPath,
    typeResolver: analysisHost.typeResolver,
    semanticReferenceIndex: analysisHost.semanticReferenceIndex,
    styleDependencyGraph: styleHost.styleDependencyGraph,
    checkerReport: command.checkerReport,
  };
}

export async function buildContractParitySnapshotV1(entry: ContractParityEntry) {
  const context = await prepareContractParityContext(entry);
  return await buildCheckerEngineParitySnapshotV1({
    workspaceRoot: context.workspaceRoot,
    classnameTransform: context.classnameTransform,
    pathAlias: context.pathAlias,
    sourceDocuments: context.sourceDocuments,
    styleFiles: context.styleFiles,
    analysisCache: context.analysisCache,
    styleDocumentForPath: context.styleDocumentForPath,
    typeResolver: context.typeResolver,
    semanticReferenceIndex: context.semanticReferenceIndex,
    styleDependencyGraph: context.styleDependencyGraph,
    checkerReport: context.checkerReport,
  });
}

export async function buildContractParitySnapshotV2(entry: ContractParityEntry) {
  const context = await prepareContractParityContext(entry);
  return await buildCheckerEngineParitySnapshotV2({
    workspaceRoot: context.workspaceRoot,
    classnameTransform: context.classnameTransform,
    pathAlias: context.pathAlias,
    sourceDocuments: context.sourceDocuments,
    styleFiles: context.styleFiles,
    analysisCache: context.analysisCache,
    styleDocumentForPath: context.styleDocumentForPath,
    typeResolver: context.typeResolver,
    semanticReferenceIndex: context.semanticReferenceIndex,
    styleDependencyGraph: context.styleDependencyGraph,
    checkerReport: context.checkerReport,
  });
}

export async function buildContractParitySnapshot(entry: ContractParityEntry) {
  return buildContractParitySnapshotV2(entry);
}

export function normalizeContractParitySnapshot<T>(value: T, workspaceRoot: string): T {
  return normalizeValue(value, path.resolve(workspaceRoot)) as T;
}

export function stableJsonStringify(value: unknown): string {
  return `${JSON.stringify(sortObjectKeys(value), null, 2)}\n`;
}

function normalizeValue(value: unknown, workspaceRoot: string): unknown {
  if (typeof value === "string") {
    return normalizePathString(value, workspaceRoot);
  }
  if (Array.isArray(value)) {
    return value.map((entry) => normalizeValue(entry, workspaceRoot));
  }
  if (!value || typeof value !== "object") {
    return value;
  }

  return Object.fromEntries(
    Object.entries(value).map(([key, nested]) => [key, normalizeValue(nested, workspaceRoot)]),
  );
}

function normalizePathString(value: string, workspaceRoot: string): string {
  const withPortableFileUrls = replaceWorkspaceReference(
    value,
    pathToFileURL(workspaceRoot).href,
    "/",
  );
  if (!hasWorkspaceReference(withPortableFileUrls, workspaceRoot, path.sep)) {
    return withPortableFileUrls;
  }

  const withPortableSeparators =
    path.sep === "/" ? withPortableFileUrls : withPortableFileUrls.replaceAll(path.sep, "/");
  return replaceWorkspaceReference(withPortableSeparators, toPosix(workspaceRoot), "/");
}

function hasWorkspaceReference(value: string, workspaceRoot: string, separator: string): boolean {
  let cursor = 0;

  for (;;) {
    const occurrence = value.indexOf(workspaceRoot, cursor);
    if (occurrence === -1) return false;

    const afterWorkspaceRoot = occurrence + workspaceRoot.length;
    if (afterWorkspaceRoot === value.length || value.startsWith(separator, afterWorkspaceRoot)) {
      return true;
    }
    cursor = afterWorkspaceRoot;
  }
}

function replaceWorkspaceReference(
  value: string,
  workspaceRoot: string,
  separator: string,
): string {
  let cursor = 0;
  let normalized = "";

  for (;;) {
    const occurrence = value.indexOf(workspaceRoot, cursor);
    if (occurrence === -1) return normalized + value.slice(cursor);

    const afterWorkspaceRoot = occurrence + workspaceRoot.length;
    normalized += value.slice(cursor, occurrence);

    if (value.startsWith(separator, afterWorkspaceRoot)) {
      normalized += "<workspace>/";
      cursor = afterWorkspaceRoot + separator.length;
      continue;
    }
    if (afterWorkspaceRoot === value.length) return normalized + "<workspace>";

    normalized += workspaceRoot;
    cursor = afterWorkspaceRoot;
  }
}

function sortObjectKeys(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(sortObjectKeys);
  }
  if (!value || typeof value !== "object") {
    return value;
  }

  return Object.fromEntries(
    Object.entries(value)
      .toSorted(([a], [b]) => a.localeCompare(b))
      .map(([key, nested]) => [key, sortObjectKeys(nested)]),
  );
}

function toPosix(value: string): string {
  return value.split(path.sep).join("/");
}
