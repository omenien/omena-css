import type { Plugin } from "vite";

export interface OmenaVitePluginOptions {
  readonly name?: string;
  readonly enforce?: "pre" | "post";
  readonly include?: RegExp | string | readonly (RegExp | string)[] | ((id: string) => boolean);
  readonly passes?: readonly string[];
  readonly minify?: boolean;
  readonly treeShake?: boolean;
  readonly bundle?: boolean;
  readonly closedStyleWorld?: boolean;
  readonly sourceMap?: boolean;
  readonly requireDiskSource?: boolean;
  readonly sources?: readonly string[];
  readonly packageManifests?: readonly string[];
  readonly targetQuery?: string;
  readonly targetOptions?: Record<string, unknown>;
  readonly context?: Record<string, unknown>;
  readonly cwd?: string;
  readonly configFile?: false | string;
  readonly wasmFallback?: boolean;
  readonly devRuntime?: boolean;
}

export declare const MINIFY_PASS_IDS: readonly string[];
export declare const TREE_SHAKE_PASS_IDS: readonly string[];
export declare const VIRTUAL_MODULE_ID: "virtual:omena-css/build-summary";
export type OmenaCssModuleExportDeltaDecision = "styleOnly" | "valueChanged" | "shapeChanged";
export declare function classifyCssModuleExportDelta(
  previousClassMap: Readonly<Record<string, string>> | null | undefined,
  nextClassMap: Readonly<Record<string, string>> | null | undefined,
): OmenaCssModuleExportDeltaDecision;
export declare function omenaCss(options?: OmenaVitePluginOptions): Plugin;
export default omenaCss;
