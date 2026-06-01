import type { ClassRefOrigin, Range } from "@omena/shared";
import type { AbstractClassValue } from "../abstract-value/class-value-domain";
import type { EdgeCertainty } from "./certainty";
import type { EdgeReason } from "./provenance";

export interface SemanticReferenceSite {
  readonly refId: string;
  readonly selectorId: string;
  readonly filePath: string;
  readonly uri: string;
  readonly range: Range;
  readonly origin: ClassRefOrigin;
  readonly scssModulePath: string;
  readonly selectorFilePath: string;
  readonly canonicalName: string;
  readonly className: string;
  readonly selectorCertainty: EdgeCertainty;
  readonly reason: EdgeReason;
  readonly expansion: "direct" | "expanded";
  readonly abstractValue?: AbstractClassValue;
}

export interface ReferenceQueryOptions {
  readonly minimumSelectorCertainty?: EdgeCertainty;
}
