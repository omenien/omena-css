export interface OmenaStyleSourceInput {
  readonly stylePath: string;
  readonly styleSource: string;
}

export interface OmenaPackageManifestInput {
  readonly packageJsonPath: string;
  readonly packageJsonSource: string;
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
  readonly targetOptions?: Record<string, unknown>;
  readonly context?: Record<string, unknown>;
  readonly cwd?: string;
  readonly configFile?: false | string;
  readonly wasmFallback?: boolean;
  readonly devRuntime?: boolean;
  readonly engine?: unknown;
}

export interface OmenaBuildState {
  root: string;
  command: string;
  cache: Map<string, unknown>;
  generations: Map<string, number>;
  configPromise: Promise<Partial<OmenaBuildAdapterOptions>> | null;
  enginePromise: Promise<unknown> | null;
}

export interface OmenaBuildOutput {
  readonly code: string;
  readonly map: Record<string, unknown> | null;
  readonly summary: Record<string, unknown>;
}

export declare const DEFAULT_INCLUDE: RegExp;
export declare const MINIFY_PASS_IDS: readonly string[];
export declare const TREE_SHAKE_PASS_IDS: readonly string[];
export declare const BUNDLE_PASS_IDS: readonly string[];
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
  options: OmenaBuildAdapterOptions,
  state: OmenaBuildState,
): Promise<OmenaBuildOutput>;
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
export declare function summarizeCache(
  cache: Map<string, unknown>,
): readonly Record<string, unknown>[];
