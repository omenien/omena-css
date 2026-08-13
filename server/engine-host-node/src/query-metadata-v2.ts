import type { StringConstraintKindV2, ValueDomainKindV2 } from "../../engine-core-ts/src/contracts";
import type { Utf16CodeUnitLength } from "../../engine-core-ts/src/core/abstract-value/class-value-domain";
import type { SourceExpressionResolution } from "../../engine-core-ts/src/core/query/read-source-expression-resolution";

export function classifyValueDomainV2(
  abstractValue?: SourceExpressionResolution["abstractValue"],
): {
  readonly kind: ValueDomainKindV2;
  readonly constraintKind?: StringConstraintKindV2;
  readonly prefix?: string;
  readonly suffix?: string;
  readonly minLen?: Utf16CodeUnitLength;
  readonly maxLen?: Utf16CodeUnitLength;
  readonly charMust?: string;
  readonly charMay?: string;
  readonly mayIncludeOtherChars?: boolean;
} {
  if (!abstractValue) return { kind: "none" };

  switch (abstractValue.kind) {
    case "bottom":
      return { kind: "none" };
    case "exact":
      return { kind: "exact" };
    case "finiteSet":
      return { kind: "finiteSet" };
    case "prefix":
      return { kind: "constrained", constraintKind: "prefix", prefix: abstractValue.prefix };
    case "suffix":
      return { kind: "constrained", constraintKind: "suffix", suffix: abstractValue.suffix };
    case "prefixSuffix":
      return {
        kind: "constrained",
        constraintKind: "prefixSuffix",
        prefix: abstractValue.prefix,
        suffix: abstractValue.suffix,
        minLen: abstractValue.minLength,
      };
    case "charInclusion":
      return {
        kind: "constrained",
        constraintKind: "charInclusion",
        charMust: abstractValue.mustChars,
        charMay: abstractValue.mayChars,
        ...(abstractValue.mayIncludeOtherChars ? { mayIncludeOtherChars: true } : {}),
      };
    case "composite":
      return {
        kind: "constrained",
        constraintKind: "composite",
        ...(abstractValue.prefix ? { prefix: abstractValue.prefix } : {}),
        ...(abstractValue.suffix ? { suffix: abstractValue.suffix } : {}),
        ...(abstractValue.minLength !== undefined ? { minLen: abstractValue.minLength } : {}),
        charMust: abstractValue.mustChars,
        charMay: abstractValue.mayChars,
        ...(abstractValue.mayIncludeOtherChars ? { mayIncludeOtherChars: true } : {}),
      };
    case "top":
      return { kind: "top" };
    default:
      abstractValue satisfies never;
      return { kind: "none" };
  }
}
