export type AbstractClassValue =
  | BottomClassValue
  | ExactClassValue
  | FiniteSetClassValue
  | PrefixClassValue
  | SuffixClassValue
  | PrefixSuffixClassValue
  | CharInclusionClassValue
  | CompositeClassValue
  | TopClassValue;

export interface BottomClassValue {
  readonly kind: "bottom";
}

export interface ExactClassValue {
  readonly kind: "exact";
  readonly value: string;
}

export interface FiniteSetClassValue {
  readonly kind: "finiteSet";
  readonly values: readonly string[];
}

export interface PrefixClassValue {
  readonly kind: "prefix";
  readonly prefix: string;
  readonly provenance?:
    | "concatUnknownRight"
    | "prefixJoinLcp"
    | "finiteSetWidening"
    | "finiteSetConcatPrefixLcp";
}

export interface SuffixClassValue {
  readonly kind: "suffix";
  readonly suffix: string;
  readonly provenance?: "concatUnknownLeft" | "suffixJoinLcs";
}

export interface PrefixSuffixClassValue {
  readonly kind: "prefixSuffix";
  readonly prefix: string;
  readonly suffix: string;
  readonly minLength: number;
  readonly provenance?:
    | "concatKnownEdges"
    | "prefixFiniteSetSharedSuffix"
    | "finiteSetConcatSuffixProduct"
    | "prefixSuffixJoin";
}

export interface CharInclusionClassValue {
  readonly kind: "charInclusion";
  readonly mustChars: string;
  readonly mayChars: string;
  readonly mayIncludeOtherChars?: true;
  readonly provenance?:
    | "finiteSetWideningChars"
    | "charInclusionJoin"
    | "charInclusionConcat"
    | "concatUnknownLeft"
    | "concatUnknownRight";
}

export interface CompositeClassValue {
  readonly kind: "composite";
  readonly prefix?: string;
  readonly suffix?: string;
  readonly minLength?: number;
  readonly mustChars: string;
  readonly mayChars: string;
  readonly mayIncludeOtherChars?: true;
  readonly provenance?: "finiteSetWideningComposite" | "compositeJoin" | "compositeConcat";
}

export interface TopClassValue {
  readonly kind: "top";
  readonly provenance?:
    | "unconstrainedInput"
    | "automatonStateLimit"
    | "flowIterationLimit"
    | "missingFlowPredecessor"
    | "joinUnrepresentable"
    | "concatenationUnrepresentable"
    | "reducedProductUnconstrained";
}

export const BOTTOM_CLASS_VALUE: BottomClassValue = { kind: "bottom" };
export const TOP_CLASS_VALUE: TopClassValue = {
  kind: "top",
  provenance: "unconstrainedInput",
};
export const MAX_FINITE_CLASS_VALUES = 8;

function topClassValueWithProvenance(
  provenance: NonNullable<TopClassValue["provenance"]>,
): TopClassValue {
  return { kind: "top", provenance };
}

export function exactClassValue(value: string): ExactClassValue {
  return { kind: "exact", value };
}

export function finiteSetClassValue(values: readonly string[]): AbstractClassValue {
  const normalized = normalizeValues(values);
  if (normalized.length === 0) return BOTTOM_CLASS_VALUE;
  if (normalized.length === 1) return exactClassValue(normalized[0]!);
  if (normalized.length > MAX_FINITE_CLASS_VALUES) {
    const prefix = meaningfulLongestCommonPrefix(normalized);
    const suffix = meaningfulLongestCommonSuffix(normalized);
    const chars = charInclusionFromFiniteValues(
      normalized,
      "finiteSetWideningChars",
    ) as CharInclusionClassValue;
    if (prefix.length > 0 || suffix.length > 0) {
      return compositeClassValue({
        ...(prefix.length > 0 ? { prefix } : {}),
        ...(suffix.length > 0 ? { suffix } : {}),
        minLength: Math.min(...normalized.map((value) => value.length)),
        mustChars: chars.mustChars,
        mayChars: chars.mayChars,
        ...(chars.mayIncludeOtherChars ? { mayIncludeOtherChars: true } : {}),
        provenance: "finiteSetWideningComposite",
      });
    }
    return chars;
  }
  return { kind: "finiteSet", values: normalized };
}

export function prefixClassValue(
  prefix: string,
  provenance?: PrefixClassValue["provenance"],
): PrefixClassValue {
  return provenance ? { kind: "prefix", prefix, provenance } : { kind: "prefix", prefix };
}

export function suffixClassValue(
  suffix: string,
  provenance?: SuffixClassValue["provenance"],
): SuffixClassValue {
  return provenance ? { kind: "suffix", suffix, provenance } : { kind: "suffix", suffix };
}

export function prefixSuffixClassValue(
  prefix: string,
  suffix: string,
  minLength = prefix.length + suffix.length,
  provenance?: PrefixSuffixClassValue["provenance"],
): AbstractClassValue {
  if (prefix.length === 0 && suffix.length === 0) return TOP_CLASS_VALUE;
  if (prefix.length === 0) return suffixClassValue(suffix);
  if (suffix.length === 0) return prefixClassValue(prefix);
  const normalizedMinLength = Math.max(minLength, prefix.length + suffix.length);
  return provenance
    ? { kind: "prefixSuffix", prefix, suffix, minLength: normalizedMinLength, provenance }
    : { kind: "prefixSuffix", prefix, suffix, minLength: normalizedMinLength };
}

export function charInclusionClassValue(
  mustChars: string,
  mayChars: string,
  provenance?: CharInclusionClassValue["provenance"],
  mayIncludeOtherChars = false,
): AbstractClassValue {
  const normalizedMustChars = normalizeCharSet(mustChars);
  const normalizedMayChars = normalizeCharSet(mayChars + normalizedMustChars);
  if (mayIncludeOtherChars && normalizedMustChars.length === 0) {
    return TOP_CLASS_VALUE;
  }
  if (!mayIncludeOtherChars && normalizedMayChars.length === 0) {
    return TOP_CLASS_VALUE;
  }
  const base = mayIncludeOtherChars
    ? {
        kind: "charInclusion" as const,
        mustChars: normalizedMustChars,
        mayChars: normalizedMayChars,
        mayIncludeOtherChars: true as const,
      }
    : {
        kind: "charInclusion" as const,
        mustChars: normalizedMustChars,
        mayChars: normalizedMayChars,
      };
  return provenance ? { ...base, provenance } : base;
}

export function compositeClassValue(input: {
  readonly prefix?: string;
  readonly suffix?: string;
  readonly minLength?: number;
  readonly mustChars: string;
  readonly mayChars: string;
  readonly mayIncludeOtherChars?: boolean;
  readonly provenance?: CompositeClassValue["provenance"];
}): AbstractClassValue {
  const normalizedPrefix = input.prefix ?? "";
  const normalizedSuffix = input.suffix ?? "";
  const edgeChars = charSetForString(normalizedPrefix + normalizedSuffix);
  const normalizedMustChars = normalizeCharSet(input.mustChars + edgeChars);
  const normalizedMayChars = normalizeCharSet(input.mayChars + normalizedMustChars);
  const mayIncludeOtherChars = Boolean(input.mayIncludeOtherChars);
  const hasCharInfo =
    normalizedMustChars.length > 0 || (!mayIncludeOtherChars && normalizedMayChars.length > 0);

  if (!hasCharInfo) {
    if (normalizedPrefix.length > 0 && normalizedSuffix.length > 0) {
      return prefixSuffixClassValue(normalizedPrefix, normalizedSuffix, input.minLength);
    }
    if (normalizedPrefix.length > 0) return prefixClassValue(normalizedPrefix);
    if (normalizedSuffix.length > 0) return suffixClassValue(normalizedSuffix);
    return TOP_CLASS_VALUE;
  }

  if (normalizedPrefix.length === 0 && normalizedSuffix.length === 0) {
    return charInclusionClassValue(
      normalizedMustChars,
      normalizedMayChars,
      undefined,
      mayIncludeOtherChars,
    );
  }

  const guaranteedDistinctCharCount = Array.from(normalizedMustChars).length;
  const minLength =
    normalizedPrefix.length > 0 || normalizedSuffix.length > 0
      ? Math.max(input.minLength ?? 0, normalizedPrefix.length + normalizedSuffix.length)
      : input.minLength;
  const tightenedMinLength =
    minLength !== undefined
      ? Math.max(minLength, guaranteedDistinctCharCount)
      : guaranteedDistinctCharCount > 0
        ? guaranteedDistinctCharCount
        : undefined;

  const base = {
    kind: "composite" as const,
    ...(normalizedPrefix.length > 0 ? { prefix: normalizedPrefix } : {}),
    ...(normalizedSuffix.length > 0 ? { suffix: normalizedSuffix } : {}),
    ...(tightenedMinLength !== undefined ? { minLength: tightenedMinLength } : {}),
    mustChars: normalizedMustChars,
    mayChars: normalizedMayChars,
    ...(mayIncludeOtherChars ? { mayIncludeOtherChars: true as const } : {}),
  };
  return input.provenance ? { ...base, provenance: input.provenance } : base;
}

export function concatenateClassValues(
  left: AbstractClassValue,
  right: AbstractClassValue,
): AbstractClassValue {
  if (left.kind === "bottom" || right.kind === "bottom") return BOTTOM_CLASS_VALUE;
  if (left.kind === "top") return left;
  if (right.kind === "top") return right;

  if (left.kind === "charInclusion") {
    switch (right.kind) {
      case "exact":
      case "finiteSet":
      case "charInclusion":
        return concatenateCharInclusions(left, toCharInclusion(right));
      case "prefix":
        return concatenateCharInclusions(
          left,
          charInclusionClassValue(
            "",
            charSetForString(right.prefix),
            "charInclusionConcat",
            true,
          ) as CharInclusionClassValue,
        );
      case "suffix":
        return compositeClassValue({
          suffix: right.suffix,
          minLength: right.suffix.length,
          mustChars: unionCharSets(left.mustChars, charSetForString(right.suffix)),
          mayChars: unionCharSets(left.mayChars, charSetForString(right.suffix)),
          ...(left.mayIncludeOtherChars ? { mayIncludeOtherChars: true } : {}),
          provenance: "compositeConcat",
        });
      case "prefixSuffix":
        return compositeClassValue({
          suffix: right.suffix,
          minLength: right.minLength,
          mustChars: unionCharSets(left.mustChars, charSetForString(right.prefix + right.suffix)),
          mayChars: unionCharSets(left.mayChars, charSetForString(right.prefix + right.suffix)),
          mayIncludeOtherChars: true,
          provenance: "compositeConcat",
        });
      default:
        return TOP_CLASS_VALUE;
    }
  }

  if (right.kind === "charInclusion") {
    switch (left.kind) {
      case "exact":
      case "finiteSet":
        return concatenateCharInclusions(toCharInclusion(left), right);
      case "prefix":
        return compositeClassValue({
          prefix: left.prefix,
          minLength: left.prefix.length,
          mustChars: unionCharSets(charSetForString(left.prefix), right.mustChars),
          mayChars: unionCharSets(charSetForString(left.prefix), right.mayChars),
          mayIncludeOtherChars: true,
          provenance: "compositeConcat",
        });
      case "suffix":
        return concatenateCharInclusions(
          charInclusionClassValue(
            "",
            charSetForString(left.suffix),
            "charInclusionConcat",
            true,
          ) as CharInclusionClassValue,
          right,
        );
      case "prefixSuffix":
        return compositeClassValue({
          prefix: left.prefix,
          minLength: left.minLength,
          mustChars: unionCharSets(charSetForString(left.prefix + left.suffix), right.mustChars),
          mayChars: unionCharSets(charSetForString(left.prefix + left.suffix), right.mayChars),
          mayIncludeOtherChars: true,
          provenance: "compositeConcat",
        });
      default:
        return TOP_CLASS_VALUE;
    }
  }

  if (left.kind === "composite" || right.kind === "composite") {
    return topClassValueWithProvenance("concatenationUnrepresentable");
  }

  if (left.kind === "prefix") {
    switch (right.kind) {
      case "exact":
        return prefixSuffixClassValue(
          left.prefix,
          right.value,
          left.prefix.length + right.value.length,
          "concatKnownEdges",
        );
      case "finiteSet": {
        const suffix = meaningfulLongestCommonSuffix(right.values);
        return suffix.length > 0
          ? prefixSuffixClassValue(
              left.prefix,
              suffix,
              left.prefix.length + suffix.length,
              "prefixFiniteSetSharedSuffix",
            )
          : left;
      }
      case "prefix":
        return left;
      case "suffix":
        return prefixSuffixClassValue(
          left.prefix,
          right.suffix,
          left.prefix.length + right.suffix.length,
          "concatKnownEdges",
        );
      case "prefixSuffix":
        return prefixSuffixClassValue(
          left.prefix,
          right.suffix,
          left.prefix.length + right.suffix.length,
          "concatKnownEdges",
        );
      default:
        right satisfies never;
        return TOP_CLASS_VALUE;
    }
  }

  if (left.kind === "exact") {
    switch (right.kind) {
      case "exact":
        return exactClassValue(left.value + right.value);
      case "finiteSet":
        return finiteSetClassValue(right.values.map((value) => left.value + value));
      case "prefix":
        return prefixClassValue(left.value + right.prefix);
      case "suffix":
        return prefixSuffixClassValue(
          left.value,
          right.suffix,
          left.value.length + right.suffix.length,
          "concatKnownEdges",
        );
      case "prefixSuffix":
        return prefixSuffixClassValue(
          left.value + right.prefix,
          right.suffix,
          left.value.length + right.minLength,
          "concatKnownEdges",
        );
      default:
        right satisfies never;
        return TOP_CLASS_VALUE;
    }
  }

  if (right.kind === "exact") {
    switch (left.kind) {
      case "finiteSet":
        return finiteSetClassValue(left.values.map((value) => value + right.value));
      case "suffix":
        return right.value.length > 0 ? suffixClassValue(right.value) : left;
      case "prefixSuffix":
        return prefixSuffixClassValue(
          left.prefix,
          left.suffix + right.value,
          left.minLength + right.value.length,
          "concatKnownEdges",
        );
      default:
        left satisfies never;
        return TOP_CLASS_VALUE;
    }
  }

  if (left.kind === "suffix") {
    switch (right.kind) {
      case "finiteSet":
        return suffixFromFiniteValues(right.values);
      case "prefix":
        return topClassValueWithProvenance("concatenationUnrepresentable");
      case "suffix":
        return right;
      case "prefixSuffix":
        return suffixClassValue(right.suffix);
      default:
        return TOP_CLASS_VALUE;
    }
  }

  if (left.kind === "finiteSet" && right.kind === "prefix") {
    const prefix = meaningfulLongestCommonPrefix(left.values.map((value) => value + right.prefix));
    return prefix.length > 0
      ? prefixClassValue(prefix, "finiteSetConcatPrefixLcp")
      : topClassValueWithProvenance("concatenationUnrepresentable");
  }

  if (left.kind === "finiteSet" && right.kind === "suffix") {
    const prefix = meaningfulLongestCommonPrefix(left.values);
    return prefix.length > 0
      ? prefixSuffixClassValue(
          prefix,
          right.suffix,
          prefix.length + right.suffix.length,
          "finiteSetConcatSuffixProduct",
        )
      : right;
  }

  if (left.kind === "prefixSuffix") {
    switch (right.kind) {
      case "finiteSet": {
        const suffix = meaningfulLongestCommonSuffix(right.values);
        return suffix.length > 0
          ? prefixSuffixClassValue(
              left.prefix,
              left.suffix + suffix,
              left.minLength + suffix.length,
              "concatKnownEdges",
            )
          : prefixClassValue(left.prefix);
      }
      case "prefix":
        return prefixClassValue(left.prefix);
      case "suffix":
        return prefixSuffixClassValue(
          left.prefix,
          right.suffix,
          left.prefix.length + right.suffix.length,
          "concatKnownEdges",
        );
      case "prefixSuffix":
        return prefixSuffixClassValue(
          left.prefix,
          right.suffix,
          left.prefix.length + right.minLength,
          "concatKnownEdges",
        );
      default:
        return TOP_CLASS_VALUE;
    }
  }

  if (left.kind === "finiteSet" && right.kind === "finiteSet") {
    return finiteSetClassValue(
      left.values.flatMap((leftValue) => right.values.map((rightValue) => leftValue + rightValue)),
    );
  }

  return topClassValueWithProvenance("concatenationUnrepresentable");
}

export function concatenateWithUnknownRight(value: AbstractClassValue): AbstractClassValue {
  switch (value.kind) {
    case "bottom":
      return BOTTOM_CLASS_VALUE;
    case "exact":
      return value.value.length > 0
        ? prefixClassValue(value.value, "concatUnknownRight")
        : TOP_CLASS_VALUE;
    case "finiteSet": {
      const prefix = meaningfulLongestCommonPrefix(value.values);
      return prefix.length > 0
        ? prefixClassValue(prefix, "concatUnknownRight")
        : topClassValueWithProvenance("concatenationUnrepresentable");
    }
    case "prefix":
      return value;
    case "suffix":
      return topClassValueWithProvenance("concatenationUnrepresentable");
    case "prefixSuffix":
      return prefixClassValue(value.prefix);
    case "charInclusion":
      return charInclusionClassValue(value.mustChars, value.mayChars, "concatUnknownRight", true);
    case "composite":
      return compositeClassValue({
        mustChars: value.mustChars,
        mayChars: value.mayChars,
        mayIncludeOtherChars: true,
        ...(value.prefix ? { prefix: value.prefix } : {}),
        provenance: value.provenance,
      });
    case "top":
      return value;
    default:
      value satisfies never;
      return TOP_CLASS_VALUE;
  }
}

export function concatenateWithUnknownLeft(value: AbstractClassValue): AbstractClassValue {
  switch (value.kind) {
    case "bottom":
      return BOTTOM_CLASS_VALUE;
    case "exact":
      return value.value.length > 0
        ? suffixClassValue(value.value, "concatUnknownLeft")
        : TOP_CLASS_VALUE;
    case "finiteSet":
      return suffixFromFiniteValues(value.values, "concatUnknownLeft");
    case "prefix":
      return topClassValueWithProvenance("concatenationUnrepresentable");
    case "suffix":
      return value;
    case "prefixSuffix":
      return suffixClassValue(value.suffix);
    case "charInclusion":
      return charInclusionClassValue(value.mustChars, value.mayChars, "concatUnknownLeft", true);
    case "composite":
      return compositeClassValue({
        mustChars: value.mustChars,
        mayChars: value.mayChars,
        mayIncludeOtherChars: true,
        ...(value.suffix ? { suffix: value.suffix } : {}),
        provenance: value.provenance,
      });
    case "top":
      return value;
    default:
      value satisfies never;
      return TOP_CLASS_VALUE;
  }
}

export function joinClassValues(
  left: AbstractClassValue,
  right: AbstractClassValue,
): AbstractClassValue {
  if (abstractClassValueIsSubset(left, right)) return right;
  if (abstractClassValueIsSubset(right, left)) return left;

  if (left.kind === "bottom") return right;
  if (right.kind === "bottom") return left;
  if (left.kind === "top" || right.kind === "top") return TOP_CLASS_VALUE;

  if (left.kind === "charInclusion" && right.kind === "charInclusion") {
    return joinCharInclusions(left, right);
  }

  if (left.kind === "charInclusion") {
    switch (right.kind) {
      case "exact":
      case "finiteSet":
        return joinCharInclusions(left, toCharInclusion(right));
      case "composite":
        return joinCharInclusions(left, toCompositeCharInclusion(right));
      default:
        return TOP_CLASS_VALUE;
    }
  }

  if (right.kind === "charInclusion") {
    switch (left.kind) {
      case "exact":
      case "finiteSet":
        return joinCharInclusions(toCharInclusion(left), right);
      case "composite":
        return joinCharInclusions(toCompositeCharInclusion(left), right);
      default:
        return TOP_CLASS_VALUE;
    }
  }

  if (left.kind === "composite") {
    switch (right.kind) {
      case "exact":
      case "finiteSet":
        return joinCompositeWithValue(left, right);
      case "composite":
        return joinComposites(left, right);
      case "prefix":
        return joinCompositeWithPrefix(left, right);
      case "suffix":
        return joinCompositeWithSuffix(left, right);
      case "prefixSuffix":
        return joinCompositeWithPrefixSuffix(left, right);
      default:
        return TOP_CLASS_VALUE;
    }
  }

  if (right.kind === "composite") {
    switch (left.kind) {
      case "exact":
      case "finiteSet":
        return joinCompositeWithValue(right, left);
      case "prefix":
        return joinCompositeWithPrefix(right, left);
      case "suffix":
        return joinCompositeWithSuffix(right, left);
      case "prefixSuffix":
        return joinCompositeWithPrefixSuffix(right, left);
      default:
        return TOP_CLASS_VALUE;
    }
  }

  if (left.kind === "prefix" && right.kind === "prefix") {
    if (left.prefix === right.prefix) {
      return left.provenance ? left : right;
    }
    const prefix = meaningfulLongestCommonPrefix([left.prefix, right.prefix]);
    return prefix.length > 0 ? prefixClassValue(prefix, "prefixJoinLcp") : TOP_CLASS_VALUE;
  }

  if (left.kind === "suffix" && right.kind === "suffix") {
    if (left.suffix === right.suffix) {
      return left.provenance ? left : right;
    }
    const suffix = meaningfulLongestCommonSuffix([left.suffix, right.suffix]);
    return suffix.length > 0 ? suffixClassValue(suffix, "suffixJoinLcs") : TOP_CLASS_VALUE;
  }

  if (left.kind === "prefixSuffix" && right.kind === "prefixSuffix") {
    const prefix = meaningfulLongestCommonPrefix([left.prefix, right.prefix]);
    const suffix = meaningfulLongestCommonSuffix([left.suffix, right.suffix]);
    if (prefix.length > 0 && suffix.length > 0) {
      return prefixSuffixClassValue(
        prefix,
        suffix,
        Math.max(prefix.length + suffix.length, Math.min(left.minLength, right.minLength)),
        "prefixSuffixJoin",
      );
    }
    if (prefix.length > 0) return prefixClassValue(prefix, "prefixJoinLcp");
    if (suffix.length > 0) return suffixClassValue(suffix, "suffixJoinLcs");
    return TOP_CLASS_VALUE;
  }

  if (left.kind === "prefix") {
    switch (right.kind) {
      case "exact":
      case "finiteSet":
        return joinPrefixWithValue(left, right);
      case "prefixSuffix":
        return joinPrefixWithPrefixSuffix(left, right);
      case "suffix":
      case "prefix":
        return TOP_CLASS_VALUE;
      default:
        right satisfies never;
        return TOP_CLASS_VALUE;
    }
  }

  if (right.kind === "prefix") {
    switch (left.kind) {
      case "exact":
      case "finiteSet":
        return joinPrefixWithValue(right, left);
      case "prefixSuffix":
        return joinPrefixWithPrefixSuffix(right, left);
      case "suffix":
        return TOP_CLASS_VALUE;
      default:
        return TOP_CLASS_VALUE;
    }
  }

  if (left.kind === "suffix") {
    switch (right.kind) {
      case "exact":
      case "finiteSet":
        return joinSuffixWithValue(left, right);
      case "suffix":
        return left.provenance ? left : right;
      case "prefixSuffix":
        return right.suffix.endsWith(left.suffix) ? left : TOP_CLASS_VALUE;
      default:
        return TOP_CLASS_VALUE;
    }
  }

  if (right.kind === "suffix") {
    switch (left.kind) {
      case "exact":
      case "finiteSet":
        return joinSuffixWithValue(right, left);
      case "prefixSuffix":
        return left.suffix.endsWith(right.suffix) ? right : TOP_CLASS_VALUE;
      default:
        return TOP_CLASS_VALUE;
    }
  }

  if (left.kind === "prefixSuffix") {
    switch (right.kind) {
      case "exact":
      case "finiteSet":
        return joinPrefixSuffixWithValue(left, right);
      default:
        return TOP_CLASS_VALUE;
    }
  }

  if (right.kind === "prefixSuffix") {
    switch (left.kind) {
      case "exact":
      case "finiteSet":
        return joinPrefixSuffixWithValue(right, left);
      default:
        left satisfies never;
        return TOP_CLASS_VALUE;
    }
  }

  return finiteSetClassValue([...toFiniteValues(left), ...toFiniteValues(right)]);
}

export function enumerateFiniteClassValues(value: AbstractClassValue): readonly string[] | null {
  switch (value.kind) {
    case "bottom":
      return [];
    case "exact":
      return [value.value];
    case "finiteSet":
      return value.values;
    case "prefix":
    case "suffix":
    case "prefixSuffix":
    case "charInclusion":
    case "composite":
    case "top":
      return null;
    default:
      value satisfies never;
      return null;
  }
}

export function classValueMatchesCandidate(value: AbstractClassValue, candidate: string): boolean {
  switch (value.kind) {
    case "bottom":
      return false;
    case "exact":
      return candidate === value.value;
    case "finiteSet":
      return value.values.includes(candidate);
    case "prefix":
      return candidate.startsWith(value.prefix);
    case "suffix":
      return candidate.endsWith(value.suffix);
    case "prefixSuffix":
      return (
        candidate.length >= value.minLength &&
        candidate.startsWith(value.prefix) &&
        candidate.endsWith(value.suffix)
      );
    case "charInclusion":
      return matchesCharConstraints(value, candidate);
    case "composite":
      return matchesCompositeConstraints(value, candidate);
    case "top":
      return true;
    default:
      value satisfies never;
      return true;
  }
}

export function abstractClassValueIsSubset(
  left: AbstractClassValue,
  right: AbstractClassValue,
): boolean {
  if (left.kind === "bottom" || right.kind === "top") return true;
  if (left.kind === "top") return false;

  const finiteValues = enumerateFiniteClassValues(left);
  if (finiteValues) {
    return finiteValues.every((value) => classValueMatchesCandidate(right, value));
  }

  return constrainedValueIsSubset(left, right) || reducedProductValueIsSubset(left, right);
}

function constrainedValueIsSubset(left: AbstractClassValue, right: AbstractClassValue): boolean {
  switch (left.kind) {
    case "prefix":
      return right.kind === "prefix" && left.prefix.startsWith(right.prefix);
    case "suffix":
      return right.kind === "suffix" && left.suffix.endsWith(right.suffix);
    case "prefixSuffix":
      switch (right.kind) {
        case "prefix":
          return left.prefix.startsWith(right.prefix);
        case "suffix":
          return left.suffix.endsWith(right.suffix);
        case "prefixSuffix":
          return (
            left.prefix.startsWith(right.prefix) &&
            left.suffix.endsWith(right.suffix) &&
            left.minLength >= right.minLength
          );
        default:
          return false;
      }
    default:
      return false;
  }
}

interface ClassValueReductionFacts {
  readonly prefix?: string;
  readonly suffix?: string;
  readonly minLength?: number;
  readonly mustChars: string;
  readonly allowedChars?: string;
}

function reducedProductValueIsSubset(left: AbstractClassValue, right: AbstractClassValue): boolean {
  const leftFacts = reductionFactsFromAbstractValue(left);
  const rightFacts = reductionFactsFromAbstractValue(right);
  if (!leftFacts || !rightFacts) return false;
  return reductionFactsAreSubset(leftFacts, rightFacts);
}

function reductionFactsFromAbstractValue(
  value: AbstractClassValue,
): ClassValueReductionFacts | null {
  switch (value.kind) {
    case "bottom":
    case "exact":
    case "finiteSet":
      return null;
    case "prefix":
      return { prefix: value.prefix, mustChars: "" };
    case "suffix":
      return { suffix: value.suffix, mustChars: "" };
    case "prefixSuffix":
      return {
        prefix: value.prefix,
        suffix: value.suffix,
        minLength: value.minLength,
        mustChars: "",
      };
    case "charInclusion":
      return {
        mustChars: value.mustChars,
        ...(!value.mayIncludeOtherChars ? { allowedChars: value.mayChars } : {}),
      };
    case "composite":
      return {
        ...(value.prefix ? { prefix: value.prefix } : {}),
        ...(value.suffix ? { suffix: value.suffix } : {}),
        ...(value.minLength !== undefined ? { minLength: value.minLength } : {}),
        mustChars: value.mustChars,
        ...(!value.mayIncludeOtherChars ? { allowedChars: value.mayChars } : {}),
      };
    case "top":
      return { mustChars: "" };
    default:
      value satisfies never;
      return null;
  }
}

function reductionFactsAreSubset(
  left: ClassValueReductionFacts,
  right: ClassValueReductionFacts,
): boolean {
  if (right.prefix && !left.prefix?.startsWith(right.prefix)) return false;
  if (right.suffix && !left.suffix?.endsWith(right.suffix)) return false;
  if (right.minLength !== undefined && lowerBoundLength(left) < right.minLength) return false;
  if (!charSetIsSubset(right.mustChars, guaranteedChars(left))) return false;

  if (right.allowedChars !== undefined) {
    if (left.allowedChars === undefined) return false;
    if (!charSetIsSubset(left.allowedChars, right.allowedChars)) return false;
  }

  return true;
}

function lowerBoundLength(value: ClassValueReductionFacts): number {
  if (value.minLength !== undefined) return value.minLength;
  const edgeLength = (value.prefix?.length ?? 0) + (value.suffix?.length ?? 0);
  return Math.max(edgeLength, Array.from(value.mustChars).length);
}

function guaranteedChars(value: ClassValueReductionFacts): string {
  return unionCharSets(
    value.mustChars,
    charSetForString(`${value.prefix ?? ""}${value.suffix ?? ""}`),
  );
}

function joinPrefixWithValue(
  prefixValue: PrefixClassValue,
  other: Exclude<
    AbstractClassValue,
    | BottomClassValue
    | PrefixClassValue
    | SuffixClassValue
    | PrefixSuffixClassValue
    | CharInclusionClassValue
    | CompositeClassValue
    | TopClassValue
  >,
): AbstractClassValue {
  const finiteValues = toFiniteValues(other);
  return finiteValues.every((value) => value.startsWith(prefixValue.prefix))
    ? prefixValue
    : TOP_CLASS_VALUE;
}

function joinPrefixWithPrefixSuffix(
  prefixValue: PrefixClassValue,
  prefixSuffixValue: PrefixSuffixClassValue,
): AbstractClassValue {
  if (prefixSuffixValue.prefix.startsWith(prefixValue.prefix)) {
    return prefixValue;
  }
  const sharedPrefix = meaningfulLongestCommonPrefix([
    prefixValue.prefix,
    prefixSuffixValue.prefix,
  ]);
  return sharedPrefix.length > 0
    ? prefixClassValue(sharedPrefix, "prefixJoinLcp")
    : TOP_CLASS_VALUE;
}

function joinSuffixWithValue(
  suffixValue: SuffixClassValue,
  other: Exclude<
    AbstractClassValue,
    | BottomClassValue
    | PrefixClassValue
    | SuffixClassValue
    | PrefixSuffixClassValue
    | CharInclusionClassValue
    | CompositeClassValue
    | TopClassValue
  >,
): AbstractClassValue {
  const finiteValues = toFiniteValues(other);
  return finiteValues.every((value) => value.endsWith(suffixValue.suffix))
    ? suffixValue
    : TOP_CLASS_VALUE;
}

function joinPrefixSuffixWithValue(
  prefixSuffixValue: PrefixSuffixClassValue,
  other: Exclude<
    AbstractClassValue,
    | BottomClassValue
    | PrefixClassValue
    | SuffixClassValue
    | PrefixSuffixClassValue
    | CharInclusionClassValue
    | CompositeClassValue
    | TopClassValue
  >,
): AbstractClassValue {
  const finiteValues = toFiniteValues(other);
  return finiteValues.every(
    (value) =>
      value.startsWith(prefixSuffixValue.prefix) && value.endsWith(prefixSuffixValue.suffix),
  )
    ? prefixSuffixValue
    : finiteValues.every((value) => value.startsWith(prefixSuffixValue.prefix))
      ? prefixClassValue(prefixSuffixValue.prefix, "prefixJoinLcp")
      : finiteValues.every((value) => value.endsWith(prefixSuffixValue.suffix))
        ? suffixClassValue(prefixSuffixValue.suffix, "suffixJoinLcs")
        : TOP_CLASS_VALUE;
}

function concatenateCharInclusions(
  left: CharInclusionClassValue,
  right: CharInclusionClassValue,
): AbstractClassValue {
  return charInclusionClassValue(
    unionCharSets(left.mustChars, right.mustChars),
    unionCharSets(left.mayChars, right.mayChars),
    "charInclusionConcat",
    Boolean(left.mayIncludeOtherChars || right.mayIncludeOtherChars),
  );
}

function joinCharInclusions(
  left: CharInclusionClassValue,
  right: CharInclusionClassValue,
): AbstractClassValue {
  return charInclusionClassValue(
    intersectCharSets(left.mustChars, right.mustChars),
    unionCharSets(left.mayChars, right.mayChars),
    "charInclusionJoin",
    Boolean(left.mayIncludeOtherChars || right.mayIncludeOtherChars),
  );
}

function joinCompositeWithValue(
  compositeValue: CompositeClassValue,
  other: ExactClassValue | FiniteSetClassValue,
): AbstractClassValue {
  const finiteValues = toFiniteValues(other);
  const allMatchComposite = finiteValues.every((value) =>
    matchesCompositeConstraints(compositeValue, value),
  );
  if (allMatchComposite) return compositeValue;
  if (
    compositeValue.prefix &&
    finiteValues.every((value) => value.startsWith(compositeValue.prefix!))
  ) {
    return prefixClassValue(compositeValue.prefix, "prefixJoinLcp");
  }
  if (
    compositeValue.suffix &&
    finiteValues.every((value) => value.endsWith(compositeValue.suffix!))
  ) {
    return suffixClassValue(compositeValue.suffix, "suffixJoinLcs");
  }
  if (finiteValues.every((value) => matchesCharConstraints(compositeValue, value))) {
    return charInclusionClassValue(
      compositeValue.mustChars,
      compositeValue.mayChars,
      "charInclusionJoin",
      Boolean(compositeValue.mayIncludeOtherChars),
    );
  }
  return TOP_CLASS_VALUE;
}

function joinCompositeWithPrefix(
  compositeValue: CompositeClassValue,
  prefixValue: PrefixClassValue,
): AbstractClassValue {
  if (!compositeValue.prefix) return TOP_CLASS_VALUE;
  const sharedPrefix = meaningfulLongestCommonPrefix([compositeValue.prefix, prefixValue.prefix]);
  return sharedPrefix.length > 0
    ? prefixClassValue(sharedPrefix, "prefixJoinLcp")
    : TOP_CLASS_VALUE;
}

function joinCompositeWithSuffix(
  compositeValue: CompositeClassValue,
  suffixValue: SuffixClassValue,
): AbstractClassValue {
  if (!compositeValue.suffix) return TOP_CLASS_VALUE;
  const sharedSuffix = meaningfulLongestCommonSuffix([compositeValue.suffix, suffixValue.suffix]);
  return sharedSuffix.length > 0
    ? suffixClassValue(sharedSuffix, "suffixJoinLcs")
    : TOP_CLASS_VALUE;
}

function joinCompositeWithPrefixSuffix(
  compositeValue: CompositeClassValue,
  prefixSuffixValue: PrefixSuffixClassValue,
): AbstractClassValue {
  const sharedPrefix = compositeValue.prefix
    ? meaningfulLongestCommonPrefix([compositeValue.prefix, prefixSuffixValue.prefix])
    : "";
  const sharedSuffix = compositeValue.suffix
    ? meaningfulLongestCommonSuffix([compositeValue.suffix, prefixSuffixValue.suffix])
    : "";
  if (sharedPrefix.length > 0 && sharedSuffix.length > 0) {
    return prefixSuffixClassValue(
      sharedPrefix,
      sharedSuffix,
      Math.max(
        sharedPrefix.length + sharedSuffix.length,
        Math.min(compositeValue.minLength ?? 0, prefixSuffixValue.minLength),
      ),
      "prefixSuffixJoin",
    );
  }
  if (sharedPrefix.length > 0) return prefixClassValue(sharedPrefix, "prefixJoinLcp");
  if (sharedSuffix.length > 0) return suffixClassValue(sharedSuffix, "suffixJoinLcs");
  return TOP_CLASS_VALUE;
}

function joinComposites(left: CompositeClassValue, right: CompositeClassValue): AbstractClassValue {
  const prefix =
    left.prefix && right.prefix
      ? meaningfulLongestCommonPrefix([left.prefix, right.prefix]) || undefined
      : undefined;
  const suffix =
    left.suffix && right.suffix
      ? meaningfulLongestCommonSuffix([left.suffix, right.suffix]) || undefined
      : undefined;
  return compositeClassValue({
    ...(prefix ? { prefix } : {}),
    ...(suffix ? { suffix } : {}),
    ...(prefix || suffix
      ? {
          minLength: Math.max(
            (prefix?.length ?? 0) + (suffix?.length ?? 0),
            Math.min(left.minLength ?? 0, right.minLength ?? 0),
          ),
        }
      : {}),
    mustChars: intersectCharSets(left.mustChars, right.mustChars),
    mayChars: unionCharSets(left.mayChars, right.mayChars),
    ...(left.mayIncludeOtherChars || right.mayIncludeOtherChars
      ? { mayIncludeOtherChars: true }
      : {}),
    provenance: "compositeJoin",
  });
}

function charInclusionFromFiniteValues(
  values: readonly string[],
  provenance: Extract<CharInclusionClassValue["provenance"], "finiteSetWideningChars">,
): AbstractClassValue {
  const charSets = values.map((value) => charSetForString(value));
  const mustChars = charSets.reduce((acc, next, index) => {
    if (index === 0) return next;
    return intersectCharSets(acc, next);
  }, "");
  const mayChars = charSets.reduce((acc, next) => unionCharSets(acc, next), "");
  return charInclusionClassValue(mustChars, mayChars, provenance);
}

function toCompositeCharInclusion(value: CompositeClassValue): CharInclusionClassValue {
  return charInclusionClassValue(
    value.mustChars,
    value.mayChars,
    "charInclusionJoin",
    Boolean(value.mayIncludeOtherChars),
  ) as CharInclusionClassValue;
}

function toCharInclusion(
  value: ExactClassValue | FiniteSetClassValue | CharInclusionClassValue,
): CharInclusionClassValue {
  switch (value.kind) {
    case "exact": {
      const chars = charSetForString(value.value);
      return charInclusionClassValue(chars, chars) as CharInclusionClassValue;
    }
    case "finiteSet":
      return charInclusionFromFiniteValues(
        value.values,
        "finiteSetWideningChars",
      ) as CharInclusionClassValue;
    case "charInclusion":
      return value;
    default:
      value satisfies never;
      return TOP_CLASS_VALUE as never;
  }
}

function toFiniteValues(
  value: Exclude<
    AbstractClassValue,
    | BottomClassValue
    | PrefixClassValue
    | SuffixClassValue
    | PrefixSuffixClassValue
    | CharInclusionClassValue
    | CompositeClassValue
    | TopClassValue
  >,
): readonly string[] {
  return value.kind === "exact" ? [value.value] : value.values;
}

function normalizeValues(values: readonly string[]): readonly string[] {
  return Array.from(new Set(values)).toSorted();
}

function normalizeCharSet(chars: string): string {
  return Array.from(new Set(Array.from(chars)))
    .toSorted()
    .join("");
}

function unionCharSets(left: string, right: string): string {
  return normalizeCharSet(left + right);
}

function intersectCharSets(left: string, right: string): string {
  const rightSet = new Set(Array.from(right));
  return Array.from(new Set(Array.from(left).filter((char) => rightSet.has(char))))
    .toSorted()
    .join("");
}

function charSetIsSubset(left: string, right: string): boolean {
  const rightSet = new Set(Array.from(right));
  return Array.from(left).every((char) => rightSet.has(char));
}

function charSetForString(value: string): string {
  return normalizeCharSet(value);
}

function matchesCompositeConstraints(composite: CompositeClassValue, value: string): boolean {
  if (composite.minLength !== undefined && value.length < composite.minLength) return false;
  if (composite.prefix && !value.startsWith(composite.prefix)) return false;
  if (composite.suffix && !value.endsWith(composite.suffix)) return false;
  return matchesCharConstraints(composite, value);
}

function matchesCharConstraints(
  value: Pick<
    CharInclusionClassValue | CompositeClassValue,
    "mustChars" | "mayChars" | "mayIncludeOtherChars"
  >,
  candidate: string,
): boolean {
  const charSet = new Set(Array.from(candidate));
  if (Array.from(value.mustChars).some((char) => !charSet.has(char))) return false;
  if (value.mayIncludeOtherChars) return true;
  const maySet = new Set(Array.from(value.mayChars));
  return Array.from(charSet).every((char) => maySet.has(char));
}

function longestCommonPrefix(values: readonly string[]): string {
  if (values.length === 0) return "";
  let prefix = values[0]!;
  for (let index = 1; index < values.length && prefix.length > 0; index++) {
    const value = values[index]!;
    let matchLength = 0;
    while (
      matchLength < prefix.length &&
      matchLength < value.length &&
      prefix[matchLength] === value[matchLength]
    ) {
      matchLength++;
    }
    prefix = prefix.slice(0, matchLength);
  }
  return prefix;
}

function meaningfulLongestCommonPrefix(values: readonly string[]): string {
  const prefix = longestCommonPrefix(values);
  if (prefix.length === 0) return "";
  return isMeaningfulClassPrefix(prefix, values) ? prefix : "";
}

function suffixFromFiniteValues(
  values: readonly string[],
  provenance?: SuffixClassValue["provenance"],
): AbstractClassValue {
  const suffix = meaningfulLongestCommonSuffix(values);
  return suffix.length > 0
    ? suffixClassValue(suffix, provenance)
    : topClassValueWithProvenance("concatenationUnrepresentable");
}

function longestCommonSuffix(values: readonly string[]): string {
  if (values.length === 0) return "";
  const reversed = values.map((value) => [...value].toReversed().join(""));
  return [...longestCommonPrefix(reversed)].toReversed().join("");
}

function meaningfulLongestCommonSuffix(values: readonly string[]): string {
  const suffix = longestCommonSuffix(values);
  if (suffix.length === 0) return "";
  return isMeaningfulClassSuffix(suffix, values) ? suffix : "";
}

function isMeaningfulClassPrefix(prefix: string, values: readonly string[]): boolean {
  if (prefix.length === 0) return false;
  if (endsAtClassBoundary(prefix)) return true;
  return values.every(
    (value) => value.length === prefix.length || isClassBoundaryChar(value[prefix.length]),
  );
}

function isMeaningfulClassSuffix(suffix: string, values: readonly string[]): boolean {
  if (suffix.length === 0) return false;
  if (startsAtClassBoundary(suffix)) return true;
  return values.every((value) => {
    if (value.length === suffix.length) return true;
    return isClassBoundaryChar(value[value.length - suffix.length - 1]);
  });
}

function endsAtClassBoundary(value: string): boolean {
  const last = value[value.length - 1];
  return isClassBoundaryChar(last);
}

function startsAtClassBoundary(value: string): boolean {
  return isClassBoundaryChar(value[0]);
}

function isClassBoundaryChar(char: string | undefined): boolean {
  return char === "-" || char === "_";
}
