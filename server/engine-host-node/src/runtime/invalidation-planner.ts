export type RuntimeFileChangeType = "created" | "changed" | "deleted";

export interface RuntimeFileEvent {
  readonly uri: string;
  readonly type: RuntimeFileChangeType;
}

export interface OpenDocumentSnapshot {
  readonly uri: string;
  readonly filePath: string;
  readonly isStyle: boolean;
  readonly workspaceRoot: string | null;
}

export interface SettingsReloadWorkspaceChange {
  readonly workspaceRoot: string;
  readonly aliasChanged: boolean;
  readonly modeChanged: boolean;
  readonly settingsKeyChanged: boolean;
  readonly affectedSettingsDependencyUris: readonly string[];
}

export interface SettingsReloadPlan {
  readonly resourceChanged: boolean;
  readonly aliasRebuildRoots: readonly string[];
  readonly affectedStyleRoots: readonly string[];
  readonly affectedSourceUris: readonly string[];
}

export type WatchedFileChangeInput = WatchedStyleChangeInput | WatchedSourceChangeInput;

export interface WatchedStyleChangeInput {
  readonly kind: "style";
  readonly workspaceRoot: string;
  readonly filePath: string;
  readonly changeType: RuntimeFileChangeType;
  readonly semanticsChanged: boolean;
  readonly dependentSourceUris: readonly string[];
}

export interface WatchedSourceChangeInput {
  readonly kind: "source";
  readonly workspaceRoot: string;
  readonly filePath: string;
  readonly changeType: RuntimeFileChangeType;
  readonly projectConfigChange: boolean;
  readonly dependentSourceUris: readonly string[];
}

export interface WatchedFilesPlan {
  readonly aliasRebuildRoots: readonly string[];
  readonly affectedWorkspaceRoots: readonly string[];
  readonly typeResolverInvalidationRoots: readonly string[];
  readonly affectedSourceUris: readonly string[];
  readonly stylePathsToInvalidate: readonly string[];
  readonly stylePathsToPush: readonly string[];
}

export function planSettingsReload(
  changes: readonly SettingsReloadWorkspaceChange[],
  openDocuments: readonly OpenDocumentSnapshot[],
): SettingsReloadPlan {
  const aliasRebuildRoots = new Set<string>();
  const affectedSourceUris = new Set<string>();
  const affectedStyleRoots = new Set<string>();
  let resourceChanged = false;

  for (const change of changes) {
    if (!change.aliasChanged && !change.modeChanged) continue;

    resourceChanged = true;

    if (change.aliasChanged) {
      aliasRebuildRoots.add(change.workspaceRoot);
    }

    if (change.modeChanged) {
      affectedStyleRoots.add(change.workspaceRoot);
      addAll(affectedSourceUris, change.affectedSettingsDependencyUris);
    }

    if (change.aliasChanged || change.settingsKeyChanged) {
      for (const doc of openDocuments) {
        if (doc.isStyle) continue;
        if (doc.workspaceRoot !== change.workspaceRoot) continue;
        affectedSourceUris.add(doc.uri);
      }
    }
  }

  return {
    resourceChanged,
    aliasRebuildRoots: sorted(aliasRebuildRoots),
    affectedStyleRoots: sorted(affectedStyleRoots),
    affectedSourceUris: sorted(affectedSourceUris),
  };
}

export function planWatchedFileInvalidation(
  changes: readonly WatchedFileChangeInput[],
  openDocuments: readonly OpenDocumentSnapshot[],
): WatchedFilesPlan {
  const affectedWorkspaceRoots = new Set<string>();
  const typeResolverInvalidationRoots = new Set<string>();
  const affectedSourceUris = new Set<string>();
  const stylePathsToInvalidate = new Set<string>();
  const stylePathsToPush = new Set<string>();

  let hasStyleChange = false;
  let hasSourceChange = false;
  let hasProjectConfigChange = false;

  for (const change of changes) {
    affectedWorkspaceRoots.add(change.workspaceRoot);

    if (change.kind === "style") {
      hasStyleChange = true;
      if (!change.semanticsChanged) continue;
      stylePathsToInvalidate.add(change.filePath);
      if (change.changeType !== "deleted") {
        stylePathsToPush.add(change.filePath);
      }
      addAll(affectedSourceUris, change.dependentSourceUris);
      continue;
    }

    hasSourceChange = true;
    if (change.projectConfigChange) {
      hasProjectConfigChange = true;
      typeResolverInvalidationRoots.add(change.workspaceRoot);
      continue;
    }

    if (change.dependentSourceUris.length > 0) {
      typeResolverInvalidationRoots.add(change.workspaceRoot);
    }
    addAll(affectedSourceUris, change.dependentSourceUris);
  }

  if (hasProjectConfigChange) {
    for (const root of affectedWorkspaceRoots) {
      for (const doc of openDocuments) {
        if (doc.isStyle) continue;
        if (doc.workspaceRoot !== root) continue;
        affectedSourceUris.add(doc.uri);
      }
    }
  }

  const shouldSchedule = hasStyleChange || hasSourceChange;
  return {
    aliasRebuildRoots: hasProjectConfigChange ? sorted(affectedWorkspaceRoots) : [],
    affectedWorkspaceRoots: shouldSchedule ? sorted(affectedWorkspaceRoots) : [],
    typeResolverInvalidationRoots: sorted(typeResolverInvalidationRoots),
    affectedSourceUris: sorted(affectedSourceUris),
    stylePathsToInvalidate: sorted(stylePathsToInvalidate),
    stylePathsToPush: sorted(stylePathsToPush),
  };
}

function addAll(target: Set<string>, values: readonly string[]): void {
  for (const value of values) target.add(value);
}

function sorted(values: Iterable<string>): readonly string[] {
  return [...values].toSorted();
}
