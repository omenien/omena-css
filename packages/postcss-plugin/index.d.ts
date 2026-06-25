import type { PluginCreator } from "postcss";

export interface OmenaPostcssPluginOptions {
  readonly name?: string;
  readonly from?: string;
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
  readonly engine?: unknown;
}

export declare const MINIFY_PASS_IDS: readonly string[];
export declare const TREE_SHAKE_PASS_IDS: readonly string[];
export declare const omenaPostcss: PluginCreator<OmenaPostcssPluginOptions>;
export default omenaPostcss;
