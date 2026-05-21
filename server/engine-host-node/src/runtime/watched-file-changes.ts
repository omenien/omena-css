import type { StyleDocumentHIR } from "../../../engine-core-ts/src/core/hir/style-types";
import { findLangForPath } from "../../../engine-core-ts/src/core/scss/lang-registry";
import { styleDocumentSemanticFingerprint } from "../../../engine-core-ts/src/core/scss/scss-index";
import { fileUrlToPath, pathToFileUrl } from "../../../engine-core-ts/src/core/util/text-utils";
import type { RuntimeDependencySnapshot } from "./dependency-snapshot";
import type {
  RuntimeFileChangeType,
  RuntimeFileEvent,
  WatchedFileChangeInput,
} from "./invalidation-planner";

export interface WatchedFileDeps {
  readonly workspaceRoot: string;
  readonly peekStyleDocument: (path: string) => StyleDocumentHIR | null;
  readonly buildStyleDocument: (path: string, content: string) => StyleDocumentHIR;
  readonly readStyleFile: (path: string) => string | null;
}

export interface WatchedFileChangeCollectionContext {
  readonly documents: RuntimeOpenDocumentLookup;
  getDepsForFilePath(filePath: string): WatchedFileDeps | null;
}

export interface RuntimeOpenDocumentLookup {
  get(uri: string): { readonly getText: () => string } | undefined;
}

export function collectWatchedFileChangeInputs(
  events: readonly RuntimeFileEvent[],
  ctx: WatchedFileChangeCollectionContext,
  snapshot: RuntimeDependencySnapshot,
): readonly WatchedFileChangeInput[] {
  const changes: WatchedFileChangeInput[] = [];

  for (const event of events) {
    const filePath = fileUrlToPath(event.uri);
    const deps = ctx.getDepsForFilePath(filePath);
    if (!deps) continue;

    if (findLangForPath(filePath)) {
      const semanticsChanged = hasStyleSemanticChange(filePath, event.type, deps, ctx.documents);
      changes.push({
        kind: "style",
        workspaceRoot: deps.workspaceRoot,
        filePath,
        changeType: event.type,
        semanticsChanged,
        dependentSourceUris: semanticsChanged
          ? snapshot.findStyleDependentSourceUris(deps.workspaceRoot, filePath)
          : [],
      });
      continue;
    }

    changes.push({
      kind: "source",
      workspaceRoot: deps.workspaceRoot,
      filePath,
      projectConfigChange: isProjectConfigPath(filePath),
      dependentSourceUris: snapshot.findSourceDependencyUris(deps.workspaceRoot, filePath),
    });
  }

  return changes;
}

function isProjectConfigPath(filePath: string): boolean {
  const base = filePath.split(/[\\/]/u).pop();
  return (
    base !== undefined &&
    (/^tsconfig.*\.json$/u.test(base) ||
      /^jsconfig.*\.json$/u.test(base) ||
      /^vite\.config\.[cm]?[jt]s$/u.test(base) ||
      /^webpack\.config\.[cm]?[jt]s$/u.test(base))
  );
}

function hasStyleSemanticChange(
  filePath: string,
  changeType: RuntimeFileChangeType,
  deps: WatchedFileDeps,
  documents: RuntimeOpenDocumentLookup,
): boolean {
  if (changeType === "deleted") return true;
  const previous = deps.peekStyleDocument(filePath);
  if (!previous) return true;
  const nextContent = readCurrentStyleContent(filePath, deps, documents);
  if (nextContent === null) return true;
  const next = deps.buildStyleDocument(filePath, nextContent);
  return styleDocumentSemanticFingerprint(previous) !== styleDocumentSemanticFingerprint(next);
}

function readCurrentStyleContent(
  filePath: string,
  deps: WatchedFileDeps,
  documents: RuntimeOpenDocumentLookup,
): string | null {
  const openDocument = documents.get(pathToFileUrl(filePath));
  if (openDocument) {
    return openDocument.getText();
  }
  return deps.readStyleFile(filePath);
}
