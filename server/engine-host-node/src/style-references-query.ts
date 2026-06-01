import type { Range } from "@omena/shared";
import {
  findAnimationNameRefAtCursor,
  findComposesTokenAtCursor,
  findCustomPropertyDeclAtCursor,
  findCustomPropertyRefAtCursor,
  findKeyframesAtCursor,
  findKeyframesByName,
  findSassModuleMemberRefAtCursor,
  findSassSymbolAtCursor,
  findSassSymbolDeclAtCursor,
  findSassSymbolDeclForSymbol,
  findSelectorAtCursor,
  findValueDeclAtCursor,
  findValueImportAtCursor,
  findValueRefAtCursor,
  listAnimationNameRefs,
  listCustomPropertyRefs,
  listSassModuleMemberRefsForMember,
  listSassSymbolsForDecl,
  listSassWildcardSymbolsForTarget,
  listValueRefs,
  resolveComposesTarget,
  resolveCustomPropertyDeclTarget,
  resolveSassModuleMemberRefTarget,
  resolveSassWildcardSymbolTarget,
  resolveValueImportTarget,
  resolveValueTarget,
} from "../../engine-core-ts/src/core/query";
import type {
  CustomPropertyDeclHIR,
  StyleDocumentHIR,
} from "../../engine-core-ts/src/core/hir/style-types";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import { pathToFileUrl } from "../../engine-core-ts/src/core/util/text-utils";
import {
  buildSelectorUsageLocationsFromRustPayload,
  resolveRustSelectorUsagePayloadForWorkspaceTargetAsync,
  resolveRustSelectorUsagePayloadForWorkspaceTarget,
  type SelectorUsagePayloadCache,
} from "./selector-usage-query-backend";
import {
  resolveSelectedQueryBackendKind,
  usesRustSelectorUsageBackend,
} from "./selected-query-backend";
import { resolveSelectorReferenceLocations } from "./selector-references-query";
import {
  resolveRustStyleSelectorReferenceSummaryForWorkspaceTargetAsync,
  resolveRustStyleSelectorReferenceSummaryForWorkspaceTarget,
  type StyleSelectorReferenceQueryOptions,
} from "./style-selector-reference-query";
import {
  resolveStyleDesignTokenRankingForReference,
  resolveStyleDesignTokenRankingForReferenceAsync,
  type StyleDesignTokenRankingDeps,
  type StyleDesignTokenRankingQueryOptions,
} from "./style-design-token-ranking-query";

export interface StyleReferenceLocation {
  readonly uri: string;
  readonly range: Range;
}

export interface StyleReferenceQueryOptions
  extends StyleSelectorReferenceQueryOptions, StyleDesignTokenRankingQueryOptions {
  readonly env?: NodeJS.ProcessEnv;
  readonly readRustSelectorUsagePayloadForWorkspaceTarget?: typeof resolveRustSelectorUsagePayloadForWorkspaceTarget;
  readonly readRustSelectorUsagePayloadForWorkspaceTargetAsync?: typeof resolveRustSelectorUsagePayloadForWorkspaceTargetAsync;
  readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
}

type StyleReferenceDeps = Pick<
  ProviderDeps,
  | "analysisCache"
  | "semanticReferenceIndex"
  | "settings"
  | "styleDependencyGraph"
  | "styleDocumentForPath"
  | "typeResolver"
  | "workspaceRoot"
  | "aliasResolver"
  | "readStyleFile"
> &
  StyleDesignTokenRankingDeps;

export function resolveStyleReferencesAtCursor(
  args: {
    readonly filePath: string;
    readonly line: number;
    readonly character: number;
    readonly includeDeclaration: boolean;
    readonly styleDocument: StyleDocumentHIR;
  },
  deps: StyleReferenceDeps,
  options: StyleReferenceQueryOptions = {},
): readonly StyleReferenceLocation[] {
  const selector = findSelectorAtCursor(args.styleDocument, args.line, args.character);
  const composesHit = selector
    ? null
    : findComposesTokenAtCursor(args.styleDocument, args.line, args.character);
  const target = selector
    ? {
        filePath: args.filePath,
        canonicalName: selector.canonicalName,
      }
    : (() => {
        const resolved = resolveComposesTarget(
          deps.styleDocumentForPath,
          args.styleDocument.filePath,
          composesHit,
        );
        if (!resolved) return null;
        return {
          filePath: resolved.filePath,
          canonicalName: resolved.selector.canonicalName,
        };
      })();
  if (target) {
    const graphReferences = resolveRustStyleSelectorReferenceSummaryForWorkspaceTarget(
      target,
      deps,
      options,
    );
    if (graphReferences?.hasAnyReferences) {
      return graphReferences.sites.map((site) => ({
        uri: pathToFileUrl(site.filePath),
        range: site.range,
      }));
    }

    const selectedQueryBackend = resolveSelectedQueryBackendKind(options.env);
    if (usesRustSelectorUsageBackend(selectedQueryBackend)) {
      const selectorUsagePayloadCache =
        options.selectorUsagePayloadCache ??
        (
          deps as {
            readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
          }
        ).selectorUsagePayloadCache;
      const payload = (
        options.readRustSelectorUsagePayloadForWorkspaceTarget ??
        resolveRustSelectorUsagePayloadForWorkspaceTarget
      )(
        {
          workspaceRoot: deps.workspaceRoot,
          classnameTransform: deps.settings.scss.classnameTransform,
          pathAlias: deps.settings.pathAlias,
        },
        deps,
        target.filePath,
        target.canonicalName,
        selectorUsagePayloadCache,
      );
      const rustLocations =
        payload &&
        buildSelectorUsageLocationsFromRustPayload(payload)?.map((site) => ({
          uri: pathToFileUrl(site.filePath),
          range: site.range,
        }));
      if (rustLocations) return rustLocations;
    }
    if (graphReferences) return [];
    return resolveSelectorReferenceLocations(deps, target);
  }

  const keyframes =
    findKeyframesAtCursor(args.styleDocument, args.line, args.character) ??
    (() => {
      const ref = findAnimationNameRefAtCursor(args.styleDocument, args.line, args.character);
      return ref ? findKeyframesByName(args.styleDocument, ref.name) : null;
    })();
  if (keyframes) {
    const refs = listAnimationNameRefs(
      args.styleDocument,
      keyframes.name,
    ).map<StyleReferenceLocation>((ref) => ({
      uri: pathToFileUrl(args.filePath),
      range: ref.range,
    }));
    if (args.includeDeclaration) {
      refs.unshift({
        uri: pathToFileUrl(args.filePath),
        range: keyframes.range,
      });
    }
    return refs;
  }

  const valueDecl = findValueDeclAtCursor(args.styleDocument, args.line, args.character);
  if (valueDecl) {
    const valueRefs = listValueRefs(args.styleDocument, valueDecl.name).map<StyleReferenceLocation>(
      (ref) => ({
        uri: pathToFileUrl(args.filePath),
        range: ref.range,
      }),
    );
    if (args.includeDeclaration) {
      valueRefs.unshift({
        uri: pathToFileUrl(args.filePath),
        range: valueDecl.range,
      });
    }
    return dedupeLocations(valueRefs);
  }

  const valueImport = findValueImportAtCursor(args.styleDocument, args.line, args.character);
  if (valueImport) {
    const valueRefs: StyleReferenceLocation[] = [
      {
        uri: pathToFileUrl(args.filePath),
        range: valueImport.range,
      },
      ...listValueRefs(args.styleDocument, valueImport.name).map((ref) => ({
        uri: pathToFileUrl(args.filePath),
        range: ref.range,
      })),
    ];
    if (args.includeDeclaration) {
      const importedTarget = resolveValueImportTarget(
        deps.styleDocumentForPath,
        args.styleDocument.filePath,
        valueImport,
      );
      if (importedTarget) {
        valueRefs.unshift({
          uri: pathToFileUrl(importedTarget.filePath),
          range: importedTarget.valueDecl.range,
        });
      }
    }
    return dedupeLocations(valueRefs);
  }

  const customPropertyDecl = findCustomPropertyDeclAtCursor(
    args.styleDocument,
    args.line,
    args.character,
  );
  if (customPropertyDecl) {
    return readCustomPropertyReferenceLocations(args, deps, customPropertyDecl.name, {
      filePath: args.filePath,
      decl: customPropertyDecl,
    });
  }

  const customPropertyRef = findCustomPropertyRefAtCursor(
    args.styleDocument,
    args.line,
    args.character,
  );
  if (customPropertyRef) {
    const ranking = resolveStyleDesignTokenRankingForReference(
      {
        filePath: args.filePath,
        styleDocument: args.styleDocument,
        customPropertyRef,
      },
      deps,
      options,
    );
    const targetDecl =
      customPropertyReferenceTargetFromRustRanking(ranking) ??
      resolveCustomPropertyDeclTarget(
        deps.styleDocumentForPath,
        args.filePath,
        args.styleDocument,
        customPropertyRef,
        deps.styleDependencyGraph,
        deps.aliasResolver,
        { readFile: deps.readStyleFile },
      );
    if (!targetDecl) return [];
    return readCustomPropertyReferenceLocations(args, deps, customPropertyRef.name, targetDecl);
  }

  const sassSymbolDecl = findSassSymbolDeclAtCursor(args.styleDocument, args.line, args.character);
  if (sassSymbolDecl) {
    const locations = listSassSymbolsForDecl(
      args.styleDocument,
      sassSymbolDecl,
    ).map<StyleReferenceLocation>((symbol) => ({
      uri: pathToFileUrl(args.filePath),
      range: symbol.range,
    }));
    locations.push(
      ...deps.styleDependencyGraph
        .getIncomingSassModuleMemberRefs(
          args.filePath,
          sassSymbolDecl.symbolKind,
          sassSymbolDecl.name,
        )
        .map<StyleReferenceLocation>((memberRef) => ({
          uri: pathToFileUrl(memberRef.filePath),
          range: memberRef.range,
        })),
    );
    if (args.includeDeclaration) {
      locations.unshift({
        uri: pathToFileUrl(args.filePath),
        range: sassSymbolDecl.range,
      });
    }
    return dedupeLocations(locations);
  }

  const sassSymbol = findSassSymbolAtCursor(args.styleDocument, args.line, args.character);
  if (sassSymbol) {
    const matchedSassSymbolDecl = findSassSymbolDeclForSymbol(args.styleDocument, sassSymbol);
    if (!matchedSassSymbolDecl) {
      const wildcardTarget = resolveSassWildcardSymbolTarget(
        deps.styleDocumentForPath,
        args.styleDocument.filePath,
        args.styleDocument,
        sassSymbol,
        deps.aliasResolver,
        { readFile: deps.readStyleFile },
      );
      if (!wildcardTarget) return [];
      const locations = listSassWildcardSymbolsForTarget(
        args.styleDocument,
        wildcardTarget,
      ).map<StyleReferenceLocation>((symbol) => ({
        uri: pathToFileUrl(args.filePath),
        range: symbol.range,
      }));
      if (args.includeDeclaration) {
        locations.unshift({
          uri: pathToFileUrl(wildcardTarget.filePath),
          range: wildcardTarget.decl.range,
        });
      }
      return dedupeLocations(locations);
    }
    const locations = listSassSymbolsForDecl(
      args.styleDocument,
      matchedSassSymbolDecl,
    ).map<StyleReferenceLocation>((symbol) => ({
      uri: pathToFileUrl(args.filePath),
      range: symbol.range,
    }));
    if (args.includeDeclaration) {
      locations.unshift({
        uri: pathToFileUrl(args.filePath),
        range: matchedSassSymbolDecl.range,
      });
    }
    return dedupeLocations(locations);
  }

  const sassModuleMemberRef = findSassModuleMemberRefAtCursor(
    args.styleDocument,
    args.line,
    args.character,
  );
  if (sassModuleMemberRef) {
    const moduleMemberTarget = resolveSassModuleMemberRefTarget(
      deps.styleDocumentForPath,
      args.styleDocument.filePath,
      args.styleDocument,
      sassModuleMemberRef,
      deps.aliasResolver,
      { readFile: deps.readStyleFile },
    );
    if (!moduleMemberTarget) return [];
    const locations = listSassModuleMemberRefsForMember(
      args.styleDocument,
      sassModuleMemberRef,
    ).map<StyleReferenceLocation>((memberRef) => ({
      uri: pathToFileUrl(args.filePath),
      range: memberRef.range,
    }));
    if (args.includeDeclaration) {
      locations.unshift({
        uri: pathToFileUrl(moduleMemberTarget.filePath),
        range: moduleMemberTarget.decl.range,
      });
    }
    return dedupeLocations(locations);
  }

  const valueRef = findValueRefAtCursor(args.styleDocument, args.line, args.character);
  if (!valueRef) return [];
  const valueTarget = resolveValueTarget(
    deps.styleDocumentForPath,
    args.styleDocument.filePath,
    args.styleDocument,
    valueRef.name,
  );
  if (!valueTarget) return [];
  const valueRefs: StyleReferenceLocation[] = [
    ...(valueTarget.bindingKind === "imported"
      ? [
          {
            uri: pathToFileUrl(args.filePath),
            range: valueTarget.valueImport!.range,
          },
        ]
      : []),
    ...listValueRefs(args.styleDocument, valueRef.name).map((ref) => ({
      uri: pathToFileUrl(args.filePath),
      range: ref.range,
    })),
  ];
  if (args.includeDeclaration) {
    valueRefs.unshift({
      uri: pathToFileUrl(valueTarget.filePath),
      range: valueTarget.valueDecl.range,
    });
  }
  return dedupeLocations(valueRefs);
}

export async function resolveStyleReferencesAtCursorAsync(
  args: {
    readonly filePath: string;
    readonly line: number;
    readonly character: number;
    readonly includeDeclaration: boolean;
    readonly styleDocument: StyleDocumentHIR;
  },
  deps: StyleReferenceDeps,
  options: StyleReferenceQueryOptions = {},
): Promise<readonly StyleReferenceLocation[]> {
  const selector = findSelectorAtCursor(args.styleDocument, args.line, args.character);
  const composesHit = selector
    ? null
    : findComposesTokenAtCursor(args.styleDocument, args.line, args.character);
  const target = selector
    ? {
        filePath: args.filePath,
        canonicalName: selector.canonicalName,
      }
    : (() => {
        const resolved = resolveComposesTarget(
          deps.styleDocumentForPath,
          args.styleDocument.filePath,
          composesHit,
        );
        if (!resolved) return null;
        return {
          filePath: resolved.filePath,
          canonicalName: resolved.selector.canonicalName,
        };
      })();
  if (!target) {
    const customPropertyRef = findCustomPropertyRefAtCursor(
      args.styleDocument,
      args.line,
      args.character,
    );
    if (customPropertyRef) {
      const ranking = await resolveStyleDesignTokenRankingForReferenceAsync(
        {
          filePath: args.filePath,
          styleDocument: args.styleDocument,
          customPropertyRef,
        },
        deps,
        options,
      );
      const rustTargetDecl = customPropertyReferenceTargetFromRustRanking(ranking);
      if (rustTargetDecl) {
        return readCustomPropertyReferenceLocations(
          args,
          deps,
          customPropertyRef.name,
          rustTargetDecl,
        );
      }
    }
    return resolveStyleReferencesAtCursor(args, deps, {
      ...options,
      readRustStyleSemanticGraphForWorkspaceTarget: () => null,
    });
  }

  const graphReferences = await resolveRustStyleSelectorReferenceSummaryForWorkspaceTargetAsync(
    target,
    deps,
    options,
  );
  if (graphReferences?.hasAnyReferences) {
    return graphReferences.sites.map((site) => ({
      uri: pathToFileUrl(site.filePath),
      range: site.range,
    }));
  }

  const selectedQueryBackend = resolveSelectedQueryBackendKind(options.env);
  if (usesRustSelectorUsageBackend(selectedQueryBackend)) {
    const selectorUsagePayloadCache =
      options.selectorUsagePayloadCache ??
      (
        deps as {
          readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
        }
      ).selectorUsagePayloadCache;
    const payload = await (
      options.readRustSelectorUsagePayloadForWorkspaceTargetAsync ??
      resolveRustSelectorUsagePayloadForWorkspaceTargetAsync
    )(
      {
        workspaceRoot: deps.workspaceRoot,
        classnameTransform: deps.settings.scss.classnameTransform,
        pathAlias: deps.settings.pathAlias,
      },
      deps,
      target.filePath,
      target.canonicalName,
      selectorUsagePayloadCache,
      options.runRustSelectedQueryBackendJsonAsync,
    );
    const rustLocations =
      payload &&
      buildSelectorUsageLocationsFromRustPayload(payload)?.map((site) => ({
        uri: pathToFileUrl(site.filePath),
        range: site.range,
      }));
    if (rustLocations) return rustLocations;
  }
  if (graphReferences) return [];
  return resolveSelectorReferenceLocations(deps, target);
}

function dedupeLocations(
  locations: readonly StyleReferenceLocation[],
): readonly StyleReferenceLocation[] {
  const unique = new Map<string, StyleReferenceLocation>();
  for (const location of locations) {
    const key = [
      location.uri,
      location.range.start.line,
      location.range.start.character,
      location.range.end.line,
      location.range.end.character,
    ].join(":");
    if (!unique.has(key)) unique.set(key, location);
  }
  return [...unique.values()];
}

function customPropertyReferenceTargetFromRustRanking(
  ranking: Awaited<ReturnType<typeof resolveStyleDesignTokenRankingForReferenceAsync>>,
): {
  readonly filePath: string;
  readonly decl: Pick<CustomPropertyDeclHIR, "range">;
} | null {
  if (!ranking?.winnerDeclarationFilePath || !ranking.winnerDeclarationRange) return null;
  return {
    filePath: ranking.winnerDeclarationFilePath,
    decl: { range: ranking.winnerDeclarationRange },
  };
}

function readCustomPropertyReferenceLocations(
  args: Pick<
    Parameters<typeof resolveStyleReferencesAtCursor>[0],
    "filePath" | "includeDeclaration" | "styleDocument"
  >,
  deps: Pick<ProviderDeps, "styleDependencyGraph">,
  name: string,
  targetDecl: {
    readonly filePath: string;
    readonly decl: Pick<CustomPropertyDeclHIR, "range">;
  },
): readonly StyleReferenceLocation[] {
  const locations: StyleReferenceLocation[] = [
    ...deps.styleDependencyGraph.getCustomPropertyRefs(name).map((ref) => ({
      uri: pathToFileUrl(ref.filePath),
      range: ref.range,
    })),
    ...listCustomPropertyRefs(args.styleDocument, name).map((ref) => ({
      uri: pathToFileUrl(args.filePath),
      range: ref.range,
    })),
  ];

  if (args.includeDeclaration) {
    locations.unshift({
      uri: pathToFileUrl(targetDecl.filePath),
      range: targetDecl.decl.range,
    });
  }

  return dedupeLocations(locations);
}
