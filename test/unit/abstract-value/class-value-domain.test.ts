import { describe, expect, it } from "vitest";
import {
  BOTTOM_CLASS_VALUE,
  MAX_FINITE_CLASS_VALUES,
  TOP_CLASS_VALUE,
  abstractClassValueIsSubset,
  type AbstractClassValue,
  concatenateClassValues,
  concatenateWithUnknownLeft,
  concatenateWithUnknownRight,
  classValueMatchesCandidate,
  charInclusionClassValue,
  compositeClassValue,
  enumerateFiniteClassValues,
  exactClassValue,
  finiteSetClassValue,
  joinClassValues,
  prefixClassValue,
  prefixSuffixClassValue,
  suffixClassValue,
} from "../../../server/engine-core-ts/src/core/abstract-value/class-value-domain";

describe("class-value-domain", () => {
  it("uses the expected concatenation table for core domain combinations", () => {
    const cases: ReadonlyArray<{
      name: string;
      left: AbstractClassValue;
      right: AbstractClassValue;
      expected: AbstractClassValue;
    }> = [
      {
        name: "bottom + exact => bottom",
        left: BOTTOM_CLASS_VALUE,
        right: exactClassValue("x"),
        expected: BOTTOM_CLASS_VALUE,
      },
      {
        name: "exact + bottom => bottom",
        left: exactClassValue("x"),
        right: BOTTOM_CLASS_VALUE,
        expected: BOTTOM_CLASS_VALUE,
      },
      {
        name: "top + exact => top",
        left: TOP_CLASS_VALUE,
        right: exactClassValue("x"),
        expected: TOP_CLASS_VALUE,
      },
      {
        name: "exact + top => top",
        left: exactClassValue("x"),
        right: TOP_CLASS_VALUE,
        expected: TOP_CLASS_VALUE,
      },
      {
        name: "exact + exact => exact",
        left: exactClassValue("btn-"),
        right: exactClassValue("lg"),
        expected: exactClassValue("btn-lg"),
      },
      {
        name: "exact + finiteSet => finiteSet",
        left: exactClassValue("btn-"),
        right: finiteSetClassValue(["sm", "lg"]),
        expected: finiteSetClassValue(["btn-sm", "btn-lg"]),
      },
      {
        name: "exact + prefix => prefix(left + right)",
        left: exactClassValue("btn-"),
        right: prefixClassValue("state-"),
        expected: prefixClassValue("btn-state-"),
      },
      {
        name: "exact + suffix => suffix(right)",
        left: exactClassValue("btn-"),
        right: suffixClassValue("-active"),
        expected: prefixSuffixClassValue("btn-", "-active", 11, "concatKnownEdges"),
      },
      {
        name: "finiteSet + exact => finiteSet",
        left: finiteSetClassValue(["btn-sm", "btn-lg"]),
        right: exactClassValue("--active"),
        expected: finiteSetClassValue(["btn-sm--active", "btn-lg--active"]),
      },
      {
        name: "finiteSet + finiteSet => cartesian finiteSet",
        left: finiteSetClassValue(["btn-", "card-"]),
        right: finiteSetClassValue(["sm", "lg"]),
        expected: finiteSetClassValue(["btn-sm", "btn-lg", "card-sm", "card-lg"]),
      },
      {
        name: "finiteSet + prefix preserves a meaningful shared prefix",
        left: finiteSetClassValue(["btn-sm-", "btn-lg-"]),
        right: prefixClassValue("state-"),
        expected: prefixClassValue("btn-", "finiteSetConcatPrefixLcp"),
      },
      {
        name: "finiteSet + suffix preserves the right suffix",
        left: finiteSetClassValue(["btn-sm-", "btn-lg-"]),
        right: suffixClassValue("-active"),
        expected: prefixSuffixClassValue("btn-", "-active", 11, "finiteSetConcatSuffixProduct"),
      },
      {
        name: "prefix + exact preserves left prefix",
        left: prefixClassValue("btn-"),
        right: exactClassValue("active"),
        expected: prefixSuffixClassValue("btn-", "active", 10, "concatKnownEdges"),
      },
      {
        name: "prefix + finiteSet recovers a shared suffix when present",
        left: prefixClassValue("btn-"),
        right: finiteSetClassValue(["sm-chip", "lg-chip"]),
        expected: prefixSuffixClassValue("btn-", "-chip", 9, "prefixFiniteSetSharedSuffix"),
      },
      {
        name: "prefix + prefix preserves left prefix",
        left: prefixClassValue("btn-"),
        right: prefixClassValue("state-"),
        expected: prefixClassValue("btn-"),
      },
      {
        name: "prefix + suffix becomes a prefix-suffix product",
        left: prefixClassValue("btn-"),
        right: suffixClassValue("-active"),
        expected: prefixSuffixClassValue("btn-", "-active", 11, "concatKnownEdges"),
      },
      {
        name: "prefix + char inclusion becomes a composite product",
        left: prefixClassValue("btn-"),
        right: charInclusionClassValue("s", "st"),
        expected: compositeClassValue({
          prefix: "btn-",
          minLength: 4,
          mustChars: "-bnst",
          mayChars: "-bnst",
          mayIncludeOtherChars: true,
          provenance: "compositeConcat",
        }),
      },
      {
        name: "suffix + exact keeps the final exact suffix",
        left: suffixClassValue("-active"),
        right: exactClassValue("--busy"),
        expected: suffixClassValue("--busy"),
      },
      {
        name: "suffix + finiteSet recovers a shared suffix",
        left: suffixClassValue("-active"),
        right: finiteSetClassValue(["btn-primary", "card-primary"]),
        expected: suffixClassValue("-primary"),
      },
      {
        name: "suffix + suffix keeps the right suffix",
        left: suffixClassValue("-active"),
        right: suffixClassValue("-busy"),
        expected: suffixClassValue("-busy"),
      },
      {
        name: "prefixSuffix + exact appends to the suffix side",
        left: prefixSuffixClassValue("btn-", "-chip", 9),
        right: exactClassValue("--active"),
        expected: prefixSuffixClassValue("btn-", "-chip--active", 17, "concatKnownEdges"),
      },
      {
        name: "char inclusion + suffix becomes a trailing composite product",
        left: charInclusionClassValue("s", "st"),
        right: suffixClassValue("-active"),
        expected: compositeClassValue({
          suffix: "-active",
          minLength: 7,
          mustChars: "-aceistv",
          mayChars: "-aceistv",
          provenance: "compositeConcat",
        }),
      },
      {
        name: "prefix-suffix + char inclusion preserves the leading edge with chars",
        left: prefixSuffixClassValue("btn-", "-chip", 9),
        right: charInclusionClassValue("s", "st"),
        expected: compositeClassValue({
          prefix: "btn-",
          minLength: 9,
          mustChars: "-bchinpst",
          mayChars: "-bchinpst",
          mayIncludeOtherChars: true,
          provenance: "compositeConcat",
        }),
      },
      {
        name: "char inclusion + prefix-suffix preserves the trailing edge with chars",
        left: charInclusionClassValue("s", "st"),
        right: prefixSuffixClassValue("btn-", "-chip", 9),
        expected: compositeClassValue({
          suffix: "-chip",
          minLength: 9,
          mustChars: "-bchinpst",
          mayChars: "-bchinpst",
          mayIncludeOtherChars: true,
          provenance: "compositeConcat",
        }),
      },
    ];

    for (const { name, left, right, expected } of cases) {
      expect(concatenateClassValues(left, right), name).toEqual(expected);
    }
  });

  it("canonicalizes singleton finite sets to exact values", () => {
    expect(finiteSetClassValue(["button"])).toEqual(exactClassValue("button"));
  });

  it("widens large non-prefix finite sets to character inclusion constraints", () => {
    expect(
      finiteSetClassValue(["a-0", "a-1", "a-2", "b-0", "b-1", "b-2", "c-0", "c-1", "c-2"]),
    ).toEqual(charInclusionClassValue("-", "-012abc", "finiteSetWideningChars"));
  });

  it("canonicalizes empty finite sets to bottom", () => {
    expect(finiteSetClassValue([])).toBe(BOTTOM_CLASS_VALUE);
  });

  it("normalizes finite sets by uniqueness and ordering", () => {
    expect(finiteSetClassValue(["lg", "sm", "lg", "md"])).toEqual({
      kind: "finiteSet",
      values: ["lg", "md", "sm"],
    });
  });

  it("joins exact values into a finite set", () => {
    expect(joinClassValues(exactClassValue("sm"), exactClassValue("lg"))).toEqual({
      kind: "finiteSet",
      values: ["lg", "sm"],
    });
  });

  it("keeps a prefix when all finite values fit inside it", () => {
    expect(joinClassValues(prefixClassValue("btn-"), exactClassValue("btn-primary"))).toEqual(
      prefixClassValue("btn-"),
    );
    expect(
      joinClassValues(
        prefixClassValue("btn-"),
        finiteSetClassValue(["btn-primary", "btn-secondary"]),
      ),
    ).toEqual(prefixClassValue("btn-"));
  });

  it("widens to top when a finite value escapes a prefix", () => {
    expect(joinClassValues(prefixClassValue("btn-"), exactClassValue("card"))).toBe(
      TOP_CLASS_VALUE,
    );
  });

  it("widens incompatible prefixes to top", () => {
    expect(joinClassValues(prefixClassValue("btn-"), prefixClassValue("card-"))).toBe(
      TOP_CLASS_VALUE,
    );
  });

  it("joins suffix-compatible values without losing the suffix", () => {
    expect(joinClassValues(suffixClassValue("-active"), exactClassValue("btn-active"))).toEqual(
      suffixClassValue("-active"),
    );
    expect(
      joinClassValues(
        suffixClassValue("-primary"),
        finiteSetClassValue(["btn-primary", "card-primary"]),
      ),
    ).toEqual(suffixClassValue("-primary"));
  });

  it("widens incompatible suffixes to top when no meaningful shared suffix survives", () => {
    expect(joinClassValues(suffixClassValue("-primary"), suffixClassValue("-secondary"))).toBe(
      TOP_CLASS_VALUE,
    );
  });

  it("joins compatible prefix-suffix products conservatively", () => {
    expect(
      joinClassValues(
        prefixSuffixClassValue("btn-", "-chip", 10),
        prefixSuffixClassValue("btn-", "-chip", 12),
      ),
    ).toEqual(prefixSuffixClassValue("btn-", "-chip", 10));
    expect(
      joinClassValues(prefixSuffixClassValue("btn-", "-chip", 10), exactClassValue("btn-sm-chip")),
    ).toEqual(prefixSuffixClassValue("btn-", "-chip", 10));
  });

  it("degrades composite joins when only the edge survives minLength mismatch", () => {
    expect(
      joinClassValues(
        compositeClassValue({
          prefix: "btn-",
          minLength: 20,
          mustChars: "-btn",
          mayChars: "-abcdeimnoprstuy",
          provenance: "compositeJoin",
        }),
        exactClassValue("btn-primary"),
      ),
    ).toEqual(prefixClassValue("btn-", "prefixJoinLcp"));
  });

  it("degrades composite joins to shared edge domains when joined with edge domains", () => {
    expect(
      joinClassValues(
        compositeClassValue({
          prefix: "btn-",
          suffix: "-chip",
          minLength: 9,
          mustChars: "-bchinpst",
          mayChars: "-bchinpst",
          mayIncludeOtherChars: true,
          provenance: "compositeJoin",
        }),
        prefixClassValue("btn-primary-"),
      ),
    ).toEqual(prefixClassValue("btn-", "prefixJoinLcp"));

    expect(
      joinClassValues(
        compositeClassValue({
          prefix: "btn-",
          suffix: "-chip",
          minLength: 9,
          mustChars: "-bchinpst",
          mayChars: "-bchinpst",
          mayIncludeOtherChars: true,
          provenance: "compositeJoin",
        }),
        prefixSuffixClassValue("btn-", "-chip", 12),
      ),
    ).toEqual(prefixSuffixClassValue("btn-", "-chip", 9, "prefixSuffixJoin"));
  });

  it("degrades composite joins to char inclusion when joined with char inclusion", () => {
    expect(
      joinClassValues(
        compositeClassValue({
          prefix: "btn-",
          minLength: 4,
          mustChars: "-bnt",
          mayChars: "-abcdefghilmnoprstuwy",
          provenance: "compositeJoin",
        }),
        charInclusionClassValue("a", "abc"),
      ),
    ).toEqual(charInclusionClassValue("", "-abcdefghilmnoprstuwy", "charInclusionJoin"));
  });

  it("preserves useful prefix information when joining prefixes with prefix-suffix products", () => {
    expect(
      joinClassValues(prefixClassValue("btn-"), prefixSuffixClassValue("btn-", "-chip", 10)),
    ).toEqual(prefixClassValue("btn-"));
    expect(
      joinClassValues(
        prefixClassValue("btn-primary-"),
        prefixSuffixClassValue("btn-", "-chip", 10),
      ),
    ).toEqual(prefixClassValue("btn-", "prefixJoinLcp"));
  });

  it("degrades prefix-suffix joins to one-sided constraints when only one edge survives", () => {
    expect(
      joinClassValues(prefixSuffixClassValue("btn-", "-chip", 10), exactClassValue("btn-sm-card")),
    ).toEqual(prefixClassValue("btn-", "prefixJoinLcp"));
    expect(
      joinClassValues(
        prefixSuffixClassValue("btn-", "-chip", 10),
        finiteSetClassValue(["card-chip", "icon-chip"]),
      ),
    ).toEqual(suffixClassValue("-chip", "suffixJoinLcs"));
  });

  it("keeps a meaningful longest common prefix when joining related prefixes", () => {
    expect(joinClassValues(prefixClassValue("btn-sm"), prefixClassValue("btn-lg"))).toEqual(
      prefixClassValue("btn-", "prefixJoinLcp"),
    );
    expect(joinClassValues(prefixClassValue("btn"), prefixClassValue("btn--danger"))).toEqual(
      prefixClassValue("btn"),
    );
  });

  it("drops unhelpful prefix joins that do not preserve a class boundary", () => {
    expect(
      joinClassValues(prefixClassValue("buttonPrimary"), prefixClassValue("buttonSecondary")),
    ).toBe(TOP_CLASS_VALUE);
  });

  it("enumerates only finite domains", () => {
    expect(enumerateFiniteClassValues(BOTTOM_CLASS_VALUE)).toEqual([]);
    expect(enumerateFiniteClassValues(exactClassValue("button"))).toEqual(["button"]);
    expect(enumerateFiniteClassValues(finiteSetClassValue(["sm", "lg"]))).toEqual(["lg", "sm"]);
    expect(enumerateFiniteClassValues(prefixClassValue("btn-"))).toBeNull();
    expect(enumerateFiniteClassValues(suffixClassValue("-active"))).toBeNull();
    expect(enumerateFiniteClassValues(prefixSuffixClassValue("btn-", "-chip"))).toBeNull();
    expect(enumerateFiniteClassValues(charInclusionClassValue("a", "abc"))).toBeNull();
    expect(enumerateFiniteClassValues(TOP_CLASS_VALUE)).toBeNull();
  });

  it("matches selector candidates against abstract value domains", () => {
    expect(classValueMatchesCandidate(exactClassValue("btn-primary"), "btn-primary")).toBe(true);
    expect(classValueMatchesCandidate(finiteSetClassValue(["btn-primary", "card"]), "card")).toBe(
      true,
    );
    expect(classValueMatchesCandidate(prefixClassValue("btn-"), "btn-primary")).toBe(true);
    expect(classValueMatchesCandidate(suffixClassValue("-active"), "btn-active")).toBe(true);
    expect(
      classValueMatchesCandidate(prefixSuffixClassValue("btn-", "-active"), "btn-large-active"),
    ).toBe(true);
    expect(classValueMatchesCandidate(charInclusionClassValue("ab", "abc"), "cab")).toBe(true);
    expect(
      classValueMatchesCandidate(
        compositeClassValue({ prefix: "btn-", mustChars: "a", mayChars: "abtmn-priy" }),
        "btn-primary",
      ),
    ).toBe(true);
    expect(classValueMatchesCandidate(prefixClassValue("btn-"), "card")).toBe(false);
    expect(classValueMatchesCandidate(charInclusionClassValue("z", "abc"), "cab")).toBe(false);
  });

  it("checks abstract value subset relations across reduced-product constraints", () => {
    expect(
      abstractClassValueIsSubset(exactClassValue("btn-primary"), prefixClassValue("btn-")),
    ).toBe(true);
    expect(
      abstractClassValueIsSubset(
        finiteSetClassValue(["btn-primary", "btn-secondary"]),
        prefixClassValue("btn-"),
      ),
    ).toBe(true);
    expect(
      abstractClassValueIsSubset(
        prefixSuffixClassValue("btn-primary-", "-active", 19),
        prefixSuffixClassValue("btn-", "-active", 11),
      ),
    ).toBe(true);

    const composite = compositeClassValue({
      prefix: "btn-",
      suffix: "-active",
      minLength: 16,
      mustChars: "-abceintv",
      mayChars: "-abceinprtv",
      provenance: "compositeJoin",
    });

    expect(abstractClassValueIsSubset(composite, prefixClassValue("btn-"))).toBe(true);
    expect(abstractClassValueIsSubset(composite, suffixClassValue("-active"))).toBe(true);
    expect(abstractClassValueIsSubset(composite, charInclusionClassValue("a", "-abceinprtv"))).toBe(
      true,
    );
    expect(abstractClassValueIsSubset(prefixClassValue("btn-"), composite)).toBe(false);
  });

  it("propagates character inclusion through concat and join", () => {
    expect(
      concatenateClassValues(charInclusionClassValue("a", "abc"), exactClassValue("-chip")),
    ).toEqual(charInclusionClassValue("-achip", "-abchip", "charInclusionConcat"));
    expect(
      joinClassValues(charInclusionClassValue("a", "abc"), finiteSetClassValue(["ab", "ac", "ad"])),
    ).toEqual(charInclusionClassValue("a", "abcd", "charInclusionJoin"));
  });

  it("concatenates exact and finite values into derived exact/finite results", () => {
    expect(concatenateClassValues(exactClassValue("btn-"), exactClassValue("lg"))).toEqual(
      exactClassValue("btn-lg"),
    );
    expect(
      concatenateClassValues(exactClassValue("btn-"), finiteSetClassValue(["sm", "lg"])),
    ).toEqual({
      kind: "finiteSet",
      values: ["btn-lg", "btn-sm"],
    });
  });

  it("preserves known prefix information while refining with more precise right values", () => {
    expect(concatenateClassValues(prefixClassValue("btn-"), exactClassValue("primary"))).toEqual(
      prefixSuffixClassValue("btn-", "primary", 11, "concatKnownEdges"),
    );
    expect(
      concatenateClassValues(prefixClassValue("btn-"), finiteSetClassValue(["sm", "lg"])),
    ).toEqual(prefixClassValue("btn-"));
    expect(concatenateClassValues(prefixClassValue("btn-"), prefixClassValue("variant-"))).toEqual(
      prefixClassValue("btn-"),
    );
  });

  it("recovers a shared prefix for finiteSet + prefix when the concatenated prefixes converge", () => {
    expect(
      concatenateClassValues(
        finiteSetClassValue(["chip-sm-", "chip-lg-"]),
        prefixClassValue("state-"),
      ),
    ).toEqual(prefixClassValue("chip-", "finiteSetConcatPrefixLcp"));
  });

  it("keeps finiteSet + prefix at top when no meaningful shared prefix survives", () => {
    expect(
      concatenateClassValues(finiteSetClassValue(["btn-", "card-"]), prefixClassValue("state-")),
    ).toBe(TOP_CLASS_VALUE);
  });

  it("derives prefixes from known left concatenation with unknown suffixes", () => {
    expect(concatenateWithUnknownRight(exactClassValue("btn-"))).toEqual(
      prefixClassValue("btn-", "concatUnknownRight"),
    );
    expect(concatenateWithUnknownRight(finiteSetClassValue(["btn-", "btn--"]))).toEqual(
      prefixClassValue("btn-", "concatUnknownRight"),
    );
    expect(concatenateWithUnknownRight(finiteSetClassValue(["btn-", "card-"]))).toBe(
      TOP_CLASS_VALUE,
    );
  });

  it("derives suffixes from unknown left concatenation with known suffixes", () => {
    expect(concatenateWithUnknownLeft(exactClassValue("-active"))).toEqual(
      suffixClassValue("-active", "concatUnknownLeft"),
    );
    expect(
      concatenateWithUnknownLeft(finiteSetClassValue(["btn-primary", "card-primary"])),
    ).toEqual(suffixClassValue("-primary", "concatUnknownLeft"));
    expect(concatenateWithUnknownLeft(prefixClassValue("btn-"))).toBe(TOP_CLASS_VALUE);
  });

  it("widens large finite sets to a composite when a meaningful LCP exists", () => {
    const values = Array.from(
      { length: MAX_FINITE_CLASS_VALUES + 1 },
      (_, index) => `btn-${index}`,
    );
    expect(finiteSetClassValue(values)).toEqual(
      compositeClassValue({
        prefix: "btn-",
        minLength: 5,
        mustChars: "-btn",
        mayChars: "-012345678btn",
        provenance: "finiteSetWideningComposite",
      }),
    );
  });

  it("tightens composite minLength from guaranteed distinct characters", () => {
    expect(
      compositeClassValue({
        prefix: "btn-",
        mustChars: "xyz",
        mayChars: "xyz",
        provenance: "compositeJoin",
      }),
    ).toEqual({
      kind: "composite",
      prefix: "btn-",
      minLength: 7,
      mustChars: "-bntxyz",
      mayChars: "-bntxyz",
      provenance: "compositeJoin",
    });
  });

  it("widens large finite sets to character inclusion when no meaningful LCP exists", () => {
    const values = Array.from(
      { length: MAX_FINITE_CLASS_VALUES + 1 },
      (_, index) => `state${index}`,
    );
    expect(finiteSetClassValue(values)).toEqual(
      charInclusionClassValue("aest", "012345678aest", "finiteSetWideningChars"),
    );
  });
});
