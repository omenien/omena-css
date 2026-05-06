import path from "node:path";
import { fileUrlToPath } from "../../../engine-core-ts/src/core/util/text-utils";
import type { ProviderDeps } from "../provider-deps";
import type { RustSelectedQueryBackendJsonRunnerAsync } from "../selected-query-backend";
import type { SelectorUsagePayloadCache } from "../selector-usage-query-backend";
import type {
  StyleSemanticGraphBatchOutputCache,
  StyleSemanticGraphCache,
} from "../style-semantic-graph-query-backend";

export interface WorkspaceFolderInfo {
  readonly uri: string;
  readonly rootPath: string;
  readonly name: string;
}

export interface WorkspaceProviderDeps extends ProviderDeps {
  readonly workspaceFolderUri: string;
  readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
  readonly styleSemanticGraphBatchOutputCache?: StyleSemanticGraphBatchOutputCache;
  readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
  readonly runRustSelectedQueryBackendJsonAsync?: RustSelectedQueryBackendJsonRunnerAsync;
  clearStyleSemanticGraphCache?(): void;
}

function isWithinRoot(rootPath: string, candidatePath: string): boolean {
  const rel = path.relative(rootPath, candidatePath);
  return rel === "" || (!rel.startsWith("..") && !path.isAbsolute(rel));
}

function compareRoots(a: WorkspaceFolderInfo, b: WorkspaceFolderInfo): number {
  if (a.rootPath.length !== b.rootPath.length) {
    return b.rootPath.length - a.rootPath.length;
  }
  return a.rootPath.localeCompare(b.rootPath);
}

export function pickOwningWorkspaceFolder(
  folders: readonly WorkspaceFolderInfo[],
  filePath: string,
): WorkspaceFolderInfo | null {
  for (const folder of folders.toSorted(compareRoots)) {
    if (isWithinRoot(folder.rootPath, filePath)) {
      return folder;
    }
  }
  return null;
}

export class WorkspaceRegistry {
  private readonly folders = new Map<string, WorkspaceFolderInfo>();
  private readonly bundles = new Map<string, WorkspaceProviderDeps>();

  register(folder: WorkspaceFolderInfo, deps: WorkspaceProviderDeps): void {
    this.folders.set(folder.uri, folder);
    this.bundles.set(folder.uri, deps);
  }

  unregister(folderUri: string): WorkspaceProviderDeps | null {
    this.folders.delete(folderUri);
    const deps = this.bundles.get(folderUri) ?? null;
    this.bundles.delete(folderUri);
    return deps;
  }

  getFolder(uri: string): WorkspaceFolderInfo | null {
    return this.folders.get(uri) ?? null;
  }

  getFolders(): readonly WorkspaceFolderInfo[] {
    return Array.from(this.folders.values()).toSorted(compareRoots);
  }

  getDeps(documentUri: string): WorkspaceProviderDeps | null {
    try {
      return this.getDepsForFilePath(fileUrlToPath(documentUri));
    } catch {
      return null;
    }
  }

  getDepsForFilePath(filePath: string): WorkspaceProviderDeps | null {
    const folder = this.resolveFolderForPath(filePath);
    return folder ? (this.bundles.get(folder.uri) ?? null) : null;
  }

  resolveFolder(documentUri: string): WorkspaceFolderInfo | null {
    try {
      return this.resolveFolderForPath(fileUrlToPath(documentUri));
    } catch {
      return null;
    }
  }

  resolveFolderForPath(filePath: string): WorkspaceFolderInfo | null {
    return pickOwningWorkspaceFolder(this.getFolders(), filePath);
  }

  allDeps(): readonly WorkspaceProviderDeps[] {
    return Array.from(this.bundles.values());
  }
}
