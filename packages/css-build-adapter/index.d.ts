export interface OmenaStyleSourceInput {
  readonly stylePath: string;
  readonly styleSource: string;
}

export interface OmenaPackageManifestInput {
  readonly packageJsonPath: string;
  readonly packageJsonSource: string;
}

export interface OmenaTargetTransformOptionsV0 {
  readonly allowLogicalToPhysical?: boolean;
  readonly allowScopeFlatten?: boolean;
  readonly allowLayerFlatten?: boolean;
  readonly enableSupportsStaticEval?: boolean;
  readonly enableMediaStaticEval?: boolean;
  readonly enableContainerStaticEval?: boolean;
  readonly dropDarkModeMediaQueries?: boolean;
}

export interface OmenaTransformExecutionContextV0 {
  readonly closedStyleWorld?: boolean;
  readonly [field: string]: unknown;
}

export interface OmenaBuildAdapterOptions {
  readonly include?: RegExp | string | readonly (RegExp | string)[] | ((id: string) => boolean);
  readonly passes?: readonly string[];
  readonly minify?: boolean;
  readonly treeShake?: boolean;
  readonly bundle?: boolean;
  readonly closedStyleWorld?: boolean;
  readonly sourceMap?: boolean;
  readonly sources?: readonly string[];
  readonly packageManifests?: readonly string[];
  readonly targetQuery?: string;
  readonly targetOptions?: OmenaTargetTransformOptionsV0;
  readonly context?: OmenaTransformExecutionContextV0;
  readonly cwd?: string;
  readonly configFile?: false | string;
  readonly wasmFallback?: boolean;
  readonly devRuntime?: boolean;
  readonly engine?: unknown;
}

export interface OmenaBuildAdapterBundleOptions extends OmenaBuildAdapterOptions {
  readonly bundle: true;
  readonly targetQuery?: undefined;
}

export interface OmenaBuildState {
  root: string;
  command: string;
  cache: Map<string, unknown>;
  generations: Map<string, number>;
  configPromise: Promise<Partial<OmenaBuildAdapterOptions>> | null;
  enginePromise: Promise<unknown> | null;
}

export interface OmenaSourceMapV3V0 {
  readonly version: 3;
  readonly file?: string;
  readonly sources: readonly string[];
  readonly sourcesContent?: readonly string[];
  readonly names: readonly string[];
  readonly mappings: string;
  readonly x_omenaSchemaVersion?: string;
  readonly x_omenaProduct?: string;
  readonly x_omenaSegmentCount?: number;
  readonly x_omenaPassIds?: readonly string[];
}

export interface OmenaTransformPassExecutionOutcomeV0 {
  readonly passId: string;
  readonly status: "applied" | "noChange" | "plannedOnly";
  readonly inputByteLen: number;
  readonly outputByteLen: number;
  readonly mutationCount: number;
  readonly provenancePreserved: boolean;
  readonly detail: string;
}

export interface OmenaTransformExecutionSummaryV0 {
  readonly schemaVersion: "0";
  readonly product: string;
  readonly requestedPassIds: readonly string[];
  readonly orderedPassIds: readonly string[];
  readonly executedPassIds: readonly string[];
  readonly plannedOnlyPassIds: readonly string[];
  readonly outputCss: string;
  readonly outcomes: readonly OmenaTransformPassExecutionOutcomeV0[];
}

export interface OmenaTransformBundleSourceSummaryV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-transform-bundle.source";
  readonly sourcePath: string;
  readonly dialect: string;
  readonly requiredPassIds: readonly string[];
  readonly plannedPassIds: readonly string[];
  readonly importInlineRequired: boolean;
  readonly moduleEvaluationRequired: boolean;
  readonly cssModulesResolutionRequired: boolean;
  readonly classHashingRequired: boolean;
  readonly valueResolutionRequired: boolean;
  readonly codeSplittingRequired: boolean;
}

export interface OmenaBundleCodeSplitWorkspacePlanOutputV0 {
  readonly sourcePath: string;
  readonly isEntry: boolean;
  readonly splitBoundary: string;
  readonly reachableFromEntries: readonly string[];
}

export interface OmenaBundleArtifactV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-query.bundle-artifact";
  readonly stylePath: string;
  readonly outputCss: string;
  readonly bundle: OmenaTransformBundleSourceSummaryV0;
  readonly sourceMapV3: OmenaSourceMapV3V0;
  readonly codeSplitOutputs: readonly OmenaBundleCodeSplitWorkspacePlanOutputV0[];
  readonly assetRewrites: readonly unknown[];
  readonly perPassProvenance: readonly OmenaTransformPassExecutionOutcomeV0[];
  readonly execution: OmenaTransformExecutionSummaryV0;
  readonly readySurfaces: readonly string[];
}

export interface OmenaConsumerBuildSummaryV0 {
  readonly schemaVersion?: "0";
  readonly product?: string;
  readonly execution: OmenaTransformExecutionSummaryV0;
  readonly sourceMapV3?: OmenaSourceMapV3V0;
  readonly readySurfaces?: readonly string[];
}

export interface OmenaBuildOutput {
  readonly code: string;
  readonly map: OmenaSourceMapV3V0 | null;
  readonly summary: OmenaConsumerBuildSummaryV0;
}

export interface OmenaBundleBuildOutput extends OmenaBuildOutput {
  readonly map: OmenaSourceMapV3V0 | null;
  readonly summary: OmenaBundleArtifactV0;
}

export declare const DEFAULT_INCLUDE: RegExp;
export declare const MINIFY_PASS_IDS: readonly string[];
export declare const TREE_SHAKE_PASS_IDS: readonly string[];
export declare function createOmenaBuildState(
  options?: OmenaBuildAdapterOptions,
  overrides?: { readonly command?: string },
): OmenaBuildState;
export declare function resolveEffectiveOptions(
  options: OmenaBuildAdapterOptions,
  state: OmenaBuildState,
): Promise<OmenaBuildAdapterOptions>;
export declare function rebuildAndCache(
  filePath: string,
  source: string,
  options: OmenaBuildAdapterBundleOptions,
  state: OmenaBuildState,
): Promise<OmenaBundleBuildOutput>;
export declare function rebuildAndCache(
  filePath: string,
  source: string,
  options: OmenaBuildAdapterOptions,
  state: OmenaBuildState,
): Promise<OmenaBuildOutput>;
export declare function runOmenaBuild(
  filePath: string,
  source: string,
  options: OmenaBuildAdapterBundleOptions,
  state: OmenaBuildState,
): Promise<OmenaBundleBuildOutput>;
export declare function runOmenaBuild(
  filePath: string,
  source: string,
  options: OmenaBuildAdapterOptions,
  state: OmenaBuildState,
): Promise<OmenaBuildOutput>;
export declare function normalizeFilePath(filePath: string): string;
export declare function matchesInclude(
  id: string,
  include: OmenaBuildAdapterOptions["include"],
): boolean;
export declare function extractCssModuleClassMap(css: string): Record<string, string>;
export declare function summarizeCache(cache: Map<string, unknown>): readonly {
  readonly filePath: string;
  readonly updatedAt: number;
  readonly outputBytes: number;
  readonly sourceMapSources: readonly string[];
  readonly readySurfaces: readonly string[];
}[];
