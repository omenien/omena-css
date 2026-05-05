import type { Range } from "@css-module-explainer/shared";
import type { StyleDocumentHIR } from "../../engine-core-ts/src/core/hir/style-types";
import {
  resolveStyleDesignTokenRankingForReference,
  resolveStyleDesignTokenRankingForReferenceAsync,
  type StyleDesignTokenRankingDeps,
  type StyleDesignTokenRankingQueryOptions,
} from "./style-design-token-ranking-query";
import {
  findAnimationNameRefAtCursor,
  findCanonicalSelector,
  findComposesTokenAtCursor,
  findCustomPropertyRefAtCursor,
  findKeyframesByName,
  findSassModuleMemberRefAtCursor,
  findSassModuleUseAtCursor,
  findSassSymbolAtCursor,
  findSassSymbolDeclForSymbol,
  findValueImportAtCursor,
  findValueRefAtCursor,
  resolveComposesTarget,
  resolveCustomPropertyDeclTarget,
  resolveSassModuleMemberRefTarget,
  resolveSassModuleUseTarget,
  resolveSassWildcardSymbolTarget,
  resolveValueImportTarget,
  resolveValueTarget,
} from "../../engine-core-ts/src/core/query";
import type { CursorParams, ProviderDeps } from "../../engine-core-ts/src/provider-deps";

export interface StyleDefinitionTarget {
  readonly originRange: Range;
  readonly targetFilePath: string;
  readonly targetRange: Range;
  readonly targetSelectionRange: Range;
}

export interface StyleDefinitionQueryOptions extends StyleDesignTokenRankingQueryOptions {}

type StyleDefinitionDeps = Pick<
  ProviderDeps,
  "styleDocumentForPath" | "aliasResolver" | "styleDependencyGraph" | "readStyleFile"
> &
  StyleDesignTokenRankingDeps;

export function resolveStyleDefinitionTargets(
  params: Pick<CursorParams, "filePath" | "line" | "character">,
  deps: StyleDefinitionDeps,
  options: StyleDefinitionQueryOptions = {},
): readonly StyleDefinitionTarget[] {
  const styleDocument = deps.styleDocumentForPath(params.filePath);
  if (!styleDocument) return [];

  const composesHit = findComposesTokenAtCursor(styleDocument, params.line, params.character);
  const composesTarget = resolveComposesTarget(
    deps.styleDocumentForPath,
    styleDocument.filePath,
    composesHit,
  );
  if (composesHit && composesTarget) {
    return [
      toStyleDefinitionTarget(
        composesHit.token.range,
        composesTarget.filePath,
        findCanonicalSelector(composesTarget.styleDocument, composesTarget.selector),
      ),
    ];
  }

  const animationRef = findAnimationNameRefAtCursor(styleDocument, params.line, params.character);
  if (animationRef) {
    const keyframes = findKeyframesByName(styleDocument, animationRef.name);
    return keyframes
      ? [toStyleDefinitionTarget(animationRef.range, styleDocument.filePath, keyframes)]
      : [];
  }

  const valueImport = findValueImportAtCursor(styleDocument, params.line, params.character);
  if (valueImport) {
    const valueTarget = resolveValueImportTarget(
      deps.styleDocumentForPath,
      styleDocument.filePath,
      valueImport,
    );
    return valueTarget
      ? [toStyleDefinitionTarget(valueImport.range, valueTarget.filePath, valueTarget.valueDecl)]
      : [];
  }

  const customPropertyRef = findCustomPropertyRefAtCursor(
    styleDocument,
    params.line,
    params.character,
  );
  if (customPropertyRef) {
    const rustTarget = resolveCustomPropertyDefinitionTargetFromRustRanking(
      params.filePath,
      styleDocument,
      customPropertyRef,
      deps,
      options,
    );
    if (rustTarget) return [rustTarget];

    const target = resolveCustomPropertyDeclTarget(
      deps.styleDocumentForPath,
      params.filePath,
      styleDocument,
      customPropertyRef,
      deps.styleDependencyGraph,
      deps.aliasResolver,
      { readFile: deps.readStyleFile },
    );
    return target
      ? [toStyleDefinitionTarget(customPropertyRef.range, target.filePath, target.decl)]
      : [];
  }

  const sassModuleUse = findSassModuleUseAtCursor(styleDocument, params.line, params.character);
  if (sassModuleUse) {
    const target = resolveSassModuleUseTarget(
      deps.styleDocumentForPath,
      styleDocument.filePath,
      sassModuleUse,
      deps.aliasResolver,
      { readFile: deps.readStyleFile },
    );
    return target
      ? [
          {
            originRange: sassModuleUse.range,
            targetFilePath: target.filePath,
            targetRange: fileStartRange(),
            targetSelectionRange: fileStartRange(),
          },
        ]
      : [];
  }

  const sassModuleMemberRef = findSassModuleMemberRefAtCursor(
    styleDocument,
    params.line,
    params.character,
  );
  if (sassModuleMemberRef) {
    const target = resolveSassModuleMemberRefTarget(
      deps.styleDocumentForPath,
      styleDocument.filePath,
      styleDocument,
      sassModuleMemberRef,
      deps.aliasResolver,
      { readFile: deps.readStyleFile },
    );
    return target
      ? [toStyleDefinitionTarget(sassModuleMemberRef.range, target.filePath, target.decl)]
      : [];
  }

  const sassSymbol = findSassSymbolAtCursor(styleDocument, params.line, params.character);
  if (sassSymbol) {
    const target = findSassSymbolDeclForSymbol(styleDocument, sassSymbol);
    if (target) {
      return [toStyleDefinitionTarget(sassSymbol.range, styleDocument.filePath, target)];
    }
    const wildcardTarget = resolveSassWildcardSymbolTarget(
      deps.styleDocumentForPath,
      styleDocument.filePath,
      styleDocument,
      sassSymbol,
      deps.aliasResolver,
      { readFile: deps.readStyleFile },
    );
    return wildcardTarget
      ? [toStyleDefinitionTarget(sassSymbol.range, wildcardTarget.filePath, wildcardTarget.decl)]
      : [];
  }

  const valueRef = findValueRefAtCursor(styleDocument, params.line, params.character);
  if (!valueRef) return [];
  const valueTarget = resolveValueTarget(
    deps.styleDocumentForPath,
    styleDocument.filePath,
    styleDocument,
    valueRef.name,
  );
  return valueTarget
    ? [toStyleDefinitionTarget(valueRef.range, valueTarget.filePath, valueTarget.valueDecl)]
    : [];
}

export async function resolveStyleDefinitionTargetsAsync(
  params: Pick<CursorParams, "filePath" | "line" | "character">,
  deps: StyleDefinitionDeps,
  options: StyleDefinitionQueryOptions = {},
): Promise<readonly StyleDefinitionTarget[]> {
  const styleDocument = deps.styleDocumentForPath(params.filePath);
  if (!styleDocument) return [];

  const customPropertyRef = findCustomPropertyRefAtCursor(
    styleDocument,
    params.line,
    params.character,
  );
  if (!customPropertyRef) {
    return resolveStyleDefinitionTargets(params, deps, options);
  }

  const rustTarget = await resolveCustomPropertyDefinitionTargetFromRustRankingAsync(
    params.filePath,
    styleDocument,
    customPropertyRef,
    deps,
    options,
  );
  if (rustTarget) return [rustTarget];

  return resolveStyleDefinitionTargets(params, deps, {
    ...options,
    readRustStyleSemanticGraphForWorkspaceTarget: () => null,
  });
}

function resolveCustomPropertyDefinitionTargetFromRustRanking(
  filePath: string,
  styleDocument: StyleDocumentHIR,
  customPropertyRef: NonNullable<ReturnType<typeof findCustomPropertyRefAtCursor>>,
  deps: StyleDefinitionDeps,
  options: StyleDefinitionQueryOptions,
): StyleDefinitionTarget | null {
  const ranking = resolveStyleDesignTokenRankingForReference(
    { filePath, styleDocument, customPropertyRef },
    deps,
    options,
  );
  if (!ranking?.winnerDeclarationFilePath || !ranking.winnerDeclarationRange) return null;

  return {
    originRange: customPropertyRef.range,
    targetFilePath: ranking.winnerDeclarationFilePath,
    targetRange: ranking.winnerDeclarationRange,
    targetSelectionRange: ranking.winnerDeclarationRange,
  };
}

async function resolveCustomPropertyDefinitionTargetFromRustRankingAsync(
  filePath: string,
  styleDocument: StyleDocumentHIR,
  customPropertyRef: NonNullable<ReturnType<typeof findCustomPropertyRefAtCursor>>,
  deps: StyleDefinitionDeps,
  options: StyleDefinitionQueryOptions,
): Promise<StyleDefinitionTarget | null> {
  const ranking = await resolveStyleDesignTokenRankingForReferenceAsync(
    { filePath, styleDocument, customPropertyRef },
    deps,
    options,
  );
  if (!ranking?.winnerDeclarationFilePath || !ranking.winnerDeclarationRange) return null;

  return {
    originRange: customPropertyRef.range,
    targetFilePath: ranking.winnerDeclarationFilePath,
    targetRange: ranking.winnerDeclarationRange,
    targetSelectionRange: ranking.winnerDeclarationRange,
  };
}

function toStyleDefinitionTarget(
  originRange: Range,
  targetFilePath: string,
  target: { readonly ruleRange: Range; readonly range: Range },
): StyleDefinitionTarget {
  return {
    originRange,
    targetFilePath,
    targetRange: target.ruleRange,
    targetSelectionRange: target.range,
  };
}

function fileStartRange(): Range {
  return {
    start: { line: 0, character: 0 },
    end: { line: 0, character: 0 },
  };
}
