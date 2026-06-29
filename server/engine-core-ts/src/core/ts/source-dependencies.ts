import path from "node:path";
import type { AliasResolver } from "../cx/alias-resolver";

const SOURCE_FILE_EXTENSIONS = [
  ".ts",
  ".tsx",
  ".js",
  ".jsx",
  ".mts",
  ".cts",
  ".mjs",
  ".cjs",
  ".d.ts",
] as const;

export function collectSourceDependencyPaths(
  filePath: string,
  moduleSpecifiers: readonly string[],
  aliasResolver?: AliasResolver,
): readonly string[] {
  const dependencyPaths = new Set<string>([path.normalize(filePath)]);

  for (const specifier of moduleSpecifiers) {
    for (const candidate of resolveSourceDependencyCandidates(filePath, specifier, aliasResolver)) {
      dependencyPaths.add(candidate);
    }
  }

  return [...dependencyPaths].toSorted();
}

function resolveSourceDependencyCandidates(
  containingFilePath: string,
  specifier: string,
  aliasResolver?: AliasResolver,
): readonly string[] {
  if (specifier.startsWith(".")) {
    const resolvedBase = path.normalize(path.resolve(path.dirname(containingFilePath), specifier));
    return expandSourceCandidates(resolvedBase);
  }

  const aliasedBase = aliasResolver?.resolve(specifier, undefined, containingFilePath);
  if (!aliasedBase) return [];
  return expandSourceCandidates(path.normalize(aliasedBase));
}

function expandSourceCandidates(resolvedBase: string): readonly string[] {
  if (SOURCE_FILE_EXTENSIONS.some((ext) => resolvedBase.endsWith(ext))) {
    return [resolvedBase];
  }

  if (path.extname(resolvedBase) !== "") {
    return [];
  }

  const candidates: string[] = [];
  for (const extension of SOURCE_FILE_EXTENSIONS) {
    candidates.push(`${resolvedBase}${extension}`);
    candidates.push(path.join(resolvedBase, `index${extension}`));
  }
  return candidates;
}
