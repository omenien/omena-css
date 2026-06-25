import type ts from "../../ts-facade";
import type { Range } from "@omena/shared";
import type { ClassValueUniverseLookupResultV0 } from "../abstract-value/class-value-universe";
import type { SourceBinderResult } from "./scope-types";

/**
 * V0 class-value universe substrate for built-in CSS Modules and recipe binders.
 *
 * This is a staged, compatibility-preserving contract used by the current
 * binder/query product path. It is not a public plugin ABI, mechanism-complete
 * class intelligence claim, or v1.0 API-finality boundary.
 */
export interface ClassValueUniverseProviderV0 {
  readonly pluginId: string;
  readonly version: "0";
  readonly stability: "builtIn";
  lookup(args: ClassValueUniverseLookupArgsV0): readonly ClassValueUniverseEntryV0[];
}

export interface ClassValueUniverseLookupArgsV0 {
  readonly sourceFile: ts.SourceFile;
  readonly filePath: string;
  readonly sourceBinder: SourceBinderResult;
}

export interface ClassValueUniverseEntryV0 {
  readonly id: string;
  readonly pluginId: string;
  readonly domain: string;
  readonly ownerName: string;
  readonly range?: Range;
  readonly universe: ClassValueUniverseLookupResultV0;
}
