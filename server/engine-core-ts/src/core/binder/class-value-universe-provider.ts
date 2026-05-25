import type ts from "typescript";
import type { Range } from "@css-module-explainer/shared";
import type { ClassValueUniverseLookupResultV0 } from "../abstract-value/class-value-universe";
import type { SourceBinderResult } from "./scope-types";

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
