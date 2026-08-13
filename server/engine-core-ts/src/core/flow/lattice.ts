import {
  enumerateFiniteClassValues,
  exactClassValue as exactAbstractClassValue,
  finiteSetClassValue,
  joinClassValues,
  type AbstractClassValue,
  type Utf16CodeUnitLength,
} from "../abstract-value/class-value-domain";
import type { EdgeCertainty } from "../semantic/certainty";

export interface ClassValueLattice {
  readonly abstractValue: AbstractClassValue;
  readonly reason: "flowLiteral" | "flowBranch";
}

export interface FlowResolution {
  readonly abstractValue: AbstractClassValue;
  readonly valueCertainty: EdgeCertainty;
  readonly reason: "flowLiteral" | "flowBranch" | "typeUnion";
}

export function exactValue(value: string): ClassValueLattice {
  return { abstractValue: exactAbstractClassValue(value), reason: "flowLiteral" };
}

export function mergeValues(
  left: ClassValueLattice | null,
  right: ClassValueLattice | null,
): ClassValueLattice | null {
  if (!left) return right;
  if (!right) return left;
  const abstractValue = joinClassValues(left.abstractValue, right.abstractValue);
  const reason =
    left.reason === "flowBranch" ||
    right.reason === "flowBranch" ||
    !sameAbstractValue(left.abstractValue, right.abstractValue)
      ? "flowBranch"
      : "flowLiteral";
  return {
    abstractValue,
    reason,
  };
}

export function markBranched(value: ClassValueLattice | null): ClassValueLattice | null {
  if (!value || value.reason === "flowBranch") return value;
  return { ...value, reason: "flowBranch" };
}

export function toFlowResolution(value: ClassValueLattice | null): FlowResolution | null {
  if (!value) return null;
  if (value.abstractValue.kind === "bottom") return null;
  const values = enumerateFiniteClassValues(value.abstractValue);
  if (!values) {
    return {
      abstractValue: value.abstractValue,
      valueCertainty: value.abstractValue.kind === "top" ? "possible" : "inferred",
      reason: value.reason,
    };
  }
  if (values.length === 0) return null;
  return {
    abstractValue: value.abstractValue,
    valueCertainty: value.abstractValue.kind === "exact" ? "exact" : "inferred",
    reason: value.reason,
  };
}

export function typeUnionResolution(values: readonly string[]): FlowResolution | null {
  const abstractValue = finiteSetClassValue(values);
  if (abstractValue.kind === "bottom") return null;
  return {
    abstractValue,
    valueCertainty:
      abstractValue.kind === "exact"
        ? "exact"
        : abstractValue.kind === "top"
          ? "possible"
          : "inferred",
    reason: "typeUnion",
  };
}

function sameAbstractValue(left: AbstractClassValue, right: AbstractClassValue): boolean {
  if (left.kind === "bottom" && right.kind === "bottom") return true;
  if (left.kind === "top" && right.kind === "top") return true;
  if (left.kind === "exact" && right.kind === "exact") return left.value === right.value;
  if (left.kind === "prefix" && right.kind === "prefix") return left.prefix === right.prefix;
  if (left.kind === "suffix" && right.kind === "suffix") return left.suffix === right.suffix;
  if (left.kind === "prefixSuffix" && right.kind === "prefixSuffix") {
    return (
      left.prefix === right.prefix &&
      left.suffix === right.suffix &&
      sameUtf16CodeUnitLength(left.minLength, right.minLength)
    );
  }
  if (left.kind === "charInclusion" && right.kind === "charInclusion") {
    return (
      left.mustChars === right.mustChars &&
      left.mayChars === right.mayChars &&
      Boolean(left.mayIncludeOtherChars) === Boolean(right.mayIncludeOtherChars)
    );
  }
  if (left.kind === "composite" && right.kind === "composite") {
    return (
      left.prefix === right.prefix &&
      left.suffix === right.suffix &&
      sameOptionalUtf16CodeUnitLength(left.minLength, right.minLength) &&
      left.mustChars === right.mustChars &&
      left.mayChars === right.mayChars &&
      Boolean(left.mayIncludeOtherChars) === Boolean(right.mayIncludeOtherChars)
    );
  }
  if (left.kind === "finiteSet" && right.kind === "finiteSet") {
    return left.values.length === right.values.length
      ? left.values.every((value, index) => value === right.values[index])
      : false;
  }
  return false;
}

function sameUtf16CodeUnitLength(left: Utf16CodeUnitLength, right: Utf16CodeUnitLength): boolean {
  return left === right;
}

function sameOptionalUtf16CodeUnitLength(
  left: Utf16CodeUnitLength | undefined,
  right: Utf16CodeUnitLength | undefined,
): boolean {
  return left === right;
}
