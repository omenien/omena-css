import path from "node:path";
import { readFileSync } from "node:fs";
import type { AliasResolver } from "../../engine-core-ts/src/core/cx/alias-resolver";
import type { StyleDocumentHIR } from "../../engine-core-ts/src/core/hir/style-types";
import {
  resolveSassModuleForwardTarget,
  resolveSassModuleUseTarget,
} from "../../engine-core-ts/src/core/query/find-style-selector";

export interface SelectedQueryStyleSourceInput {
  readonly stylePath: string;
  readonly styleSource: string;
}

export interface SelectedQueryPackageManifestInput {
  readonly packageJsonPath: string;
  readonly packageJsonSource: string;
}

export interface SelectedQueryStyleResolutionInputs {
  readonly packageManifests: readonly SelectedQueryPackageManifestInput[];
  readonly tsconfigPathMappings: readonly {
    readonly basePath: string;
    readonly pattern: string;
    readonly targetPatterns: readonly string[];
  }[];
  readonly bundlerPathMappings: readonly {
    readonly pattern: string;
    readonly targetPath: string;
  }[];
}

export interface SelectedQuerySourceDocumentInput {
  readonly sourcePath: string;
  readonly sourceSource: string;
}

export interface SelectedQuerySourceCorpusEvidence {
  readonly documents: readonly SelectedQuerySourceDocumentInput[];
  /**
   * The exhaustive source-path enumeration produced by the workspace walker.
   * `null` means the caller collected only a bounded subset.
   */
  readonly completeSourcePathEnumeration: readonly string[] | null;
}

export interface SelectedQueryWorkspaceInputs {
  readonly styles: readonly SelectedQueryStyleSourceInput[];
  readonly packageManifests: readonly SelectedQueryPackageManifestInput[];
  readonly resolutionInputs: SelectedQueryStyleResolutionInputs;
  readonly sourceCorpusComplete: boolean;
}

interface SelectedQueryWorkspaceInputDeps {
  readonly aliasResolver: AliasResolver;
  readonly buildStyleDocument: (filePath: string, content: string) => StyleDocumentHIR;
  readonly readOpenDocumentText?: (filePath: string) => string | null;
  readonly readStyleFile: (filePath: string) => string | null;
  readonly readWorkspaceFile?: (filePath: string) => string | null;
  readonly styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null;
  readonly workspaceRoot: string;
}

interface SelectedQueryWorkspaceInputSeed extends SelectedQueryStyleSourceInput {
  readonly styleDocument?: StyleDocumentHIR;
}

/**
 * Build the closed request input used by source and style selected queries.
 *
 * The collector follows only parser-owned CSS Modules and Sass dependency
 * facts. It prefers open buffers, recursively adds reachable style sources,
 * and snapshots package/path-resolution inputs instead of asking the Rust
 * request process to rediscover workspace state.
 */
export function collectSelectedQueryWorkspaceInputs(
  seeds: readonly SelectedQueryWorkspaceInputSeed[],
  deps: SelectedQueryWorkspaceInputDeps,
  containingFilePath: string,
  sourceCorpus?: SelectedQuerySourceCorpusEvidence,
): SelectedQueryWorkspaceInputs {
  const stylesByPath = new Map<string, SelectedQueryStyleSourceInput>();
  const documentsByPath = new Map<string, StyleDocumentHIR>();
  const pending: string[] = [];

  for (const seed of seeds) {
    addStyle(seed.stylePath, seed.styleSource);
    if (seed.styleDocument) {
      documentsByPath.set(normalizePath(seed.stylePath), seed.styleDocument);
    }
  }

  while (pending.length > 0) {
    const stylePath = pending.shift()!;
    const styleDocument =
      documentsByPath.get(stylePath) ??
      deps.styleDocumentForPath(stylePath) ??
      buildStyleDocument(stylePath);
    if (!styleDocument) continue;

    for (const selector of styleDocument.selectors) {
      for (const reference of selector.composes) {
        if (!reference.from || reference.fromGlobal) continue;
        addStyleFromSpecifier(stylePath, reference.from);
      }
    }
    for (const valueImport of styleDocument.valueImports) {
      addStyleFromSpecifier(stylePath, valueImport.from);
    }
    for (const moduleUse of styleDocument.sassModuleUses) {
      const target = resolveSassModuleUseTarget(
        styleDocumentForPath,
        stylePath,
        moduleUse,
        deps.aliasResolver,
        { readFile: readCurrentFile },
      );
      if (target) addStyleFromDocument(target.styleDocument);
    }
    for (const moduleForward of styleDocument.sassModuleForwards) {
      const target = resolveSassModuleForwardTarget(
        styleDocumentForPath,
        stylePath,
        moduleForward,
        deps.aliasResolver,
        { readFile: readCurrentFile },
      );
      if (target) addStyleFromDocument(target.styleDocument);
    }
  }

  const styles = [...stylesByPath.values()];
  const packageManifests = collectPackageManifests(
    styles.map((style) => style.stylePath),
    deps.readWorkspaceFile ?? readWorkspaceFile,
    deps.workspaceRoot,
  );
  const pathInputs = deps.aliasResolver.styleResolutionPathInputs(containingFilePath);
  return {
    styles,
    packageManifests,
    sourceCorpusComplete: deriveSelectedQuerySourceCorpusComplete(sourceCorpus),
    resolutionInputs: {
      packageManifests,
      ...pathInputs,
    },
  };

  function addStyle(stylePath: string, styleSource: string): void {
    const normalized = normalizePath(stylePath);
    if (stylesByPath.has(normalized)) return;
    stylesByPath.set(normalized, { stylePath: normalized, styleSource });
    pending.push(normalized);
  }

  function addStyleFromDocument(styleDocument: StyleDocumentHIR): void {
    documentsByPath.set(normalizePath(styleDocument.filePath), styleDocument);
    const source = readCurrentFile(styleDocument.filePath);
    if (source !== null) addStyle(styleDocument.filePath, source);
  }

  function addStyleFromSpecifier(fromStylePath: string, specifier: string): void {
    const targetPath = resolveStyleSpecifier(fromStylePath, specifier);
    if (!targetPath) return;
    const source = readCurrentFile(targetPath);
    if (source !== null) addStyle(targetPath, source);
  }

  function resolveStyleSpecifier(fromStylePath: string, specifier: string): string | null {
    if (specifier.startsWith(".")) {
      return path.resolve(path.dirname(fromStylePath), specifier);
    }
    return deps.aliasResolver.resolve(
      specifier,
      (candidate) => readCurrentFile(candidate) !== null,
      fromStylePath,
    );
  }

  function styleDocumentForPath(filePath: string): StyleDocumentHIR | null {
    return (
      documentsByPath.get(normalizePath(filePath)) ??
      deps.styleDocumentForPath(filePath) ??
      buildStyleDocument(filePath)
    );
  }

  function buildStyleDocument(filePath: string): StyleDocumentHIR | null {
    const source = readCurrentFile(filePath);
    if (source === null) return null;
    const document = deps.buildStyleDocument(filePath, source);
    documentsByPath.set(normalizePath(filePath), document);
    return document;
  }

  function readCurrentFile(filePath: string): string | null {
    return deps.readOpenDocumentText?.(filePath) ?? deps.readStyleFile(filePath);
  }
}

export function deriveSelectedQuerySourceCorpusComplete(
  sourceCorpus?: SelectedQuerySourceCorpusEvidence,
): boolean {
  const expectedPaths = sourceCorpus?.completeSourcePathEnumeration;
  if (!expectedPaths) return false;

  const expected = expectedPaths.map(normalizePath);
  const collected = sourceCorpus.documents.map((document) => normalizePath(document.sourcePath));
  const expectedSet = new Set(expected);
  const collectedSet = new Set(collected);
  return (
    expected.length === expectedSet.size &&
    collected.length === collectedSet.size &&
    expected.length === collected.length &&
    expected.every((sourcePath) => collectedSet.has(sourcePath))
  );
}

function readWorkspaceFile(filePath: string): string | null {
  try {
    return readFileSync(filePath, "utf8");
  } catch {
    return null;
  }
}

function collectPackageManifests(
  stylePaths: readonly string[],
  readFile: (filePath: string) => string | null,
  workspaceRoot: string,
): readonly SelectedQueryPackageManifestInput[] {
  const manifests = new Map<string, SelectedQueryPackageManifestInput>();
  const normalizedWorkspaceRoot = normalizePath(workspaceRoot);
  for (const stylePath of stylePaths) {
    let current = path.dirname(stylePath);
    while (true) {
      const packageJsonPath = path.join(current, "package.json");
      if (!manifests.has(packageJsonPath)) {
        const packageJsonSource = readFile(packageJsonPath);
        if (packageJsonSource !== null && isPackageManifest(packageJsonSource)) {
          manifests.set(packageJsonPath, { packageJsonPath, packageJsonSource });
        }
      }
      if (
        normalizePath(current) === normalizedWorkspaceRoot &&
        isWithinOrEqual(stylePath, normalizedWorkspaceRoot)
      ) {
        break;
      }
      const parent = path.dirname(current);
      if (parent === current) break;
      current = parent;
    }
  }
  return [...manifests.values()];
}

function isPackageManifest(source: string): boolean {
  try {
    const value: unknown = JSON.parse(source);
    return typeof value === "object" && value !== null && !Array.isArray(value);
  } catch {
    return false;
  }
}

function isWithinOrEqual(candidate: string, root: string): boolean {
  const relative = path.relative(root, candidate);
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}

function normalizePath(filePath: string): string {
  return path.resolve(filePath);
}
