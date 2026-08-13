import { describe, expect, it } from "vitest";
import {
  compositeClassValue,
  joinClassValues,
  prefixSuffixClassValue,
} from "../../../server/engine-core-ts/src/core/abstract-value/class-value-domain";
import {
  exactValue,
  markBranched,
  mergeValues,
  toFlowResolution,
  typeUnionResolution,
} from "../../../server/engine-core-ts/src/core/flow/lattice";

describe("flow/lattice", () => {
  it("keeps exact flow results exact", () => {
    expect(toFlowResolution(exactValue("button"))).toEqual({
      abstractValue: {
        kind: "exact",
        value: "button",
      },
      valueCertainty: "exact",
      reason: "flowLiteral",
    });
  });

  it("marks merged flow values as inferred branch results", () => {
    expect(toFlowResolution(mergeValues(exactValue("sm"), exactValue("lg")))).toEqual({
      abstractValue: {
        kind: "finiteSet",
        values: ["lg", "sm"],
      },
      valueCertainty: "inferred",
      reason: "flowBranch",
    });
  });

  it("preserves exactness across branched-but-equal values", () => {
    expect(toFlowResolution(markBranched(mergeValues(exactValue("sm"), exactValue("sm"))))).toEqual(
      {
        abstractValue: {
          kind: "exact",
          value: "sm",
        },
        valueCertainty: "exact",
        reason: "flowBranch",
      },
    );
  });

  it("compares constrained minimum lengths only in UTF-16 code units", () => {
    const overlapValue = {
      abstractValue: prefixSuffixClassValue("ab-", "-cd"),
      reason: "flowLiteral" as const,
    };
    const overlapValueFromJoin = {
      abstractValue: joinClassValues(
        compositeClassValue({
          prefix: "ab-",
          suffix: "-cd",
          mustChars: "-abcd",
          mayChars: "-abcdx",
        }),
        prefixSuffixClassValue("ab-", "-cd"),
      ),
      reason: "flowLiteral" as const,
    };
    expect(overlapValueFromJoin.abstractValue).not.toBe(overlapValue.abstractValue);
    expect(mergeValues(overlapValue, overlapValueFromJoin)).toMatchObject({
      reason: "flowLiteral",
    });
    expect(
      mergeValues(overlapValue, {
        abstractValue: prefixSuffixClassValue("ab-", "-cd", 6),
        reason: "flowLiteral",
      }),
    ).toMatchObject({ reason: "flowBranch" });

    const compositeValue = {
      abstractValue: compositeClassValue({
        prefix: "ab-",
        suffix: "-cd",
        mustChars: "-abcd",
        mayChars: "-abcdxy",
      }),
      reason: "flowLiteral" as const,
    };
    const compositeValueFromJoin = {
      abstractValue: joinClassValues(
        compositeClassValue({
          prefix: "ab-",
          suffix: "-cd",
          mustChars: "-abcd",
          mayChars: "-abcdx",
        }),
        compositeClassValue({
          prefix: "ab-",
          suffix: "-cd",
          mustChars: "-abcd",
          mayChars: "-abcdy",
        }),
      ),
      reason: "flowLiteral" as const,
    };
    expect(compositeValueFromJoin.abstractValue).not.toBe(compositeValue.abstractValue);
    expect(mergeValues(compositeValue, compositeValueFromJoin)).toMatchObject({
      reason: "flowLiteral",
    });
    expect(
      mergeValues(compositeValue, {
        abstractValue: compositeClassValue({
          prefix: "ab-",
          suffix: "-cd",
          minLength: 6,
          mustChars: "-abcd",
          mayChars: "-abcdxy",
        }),
        reason: "flowLiteral",
      }),
    ).toMatchObject({ reason: "flowBranch" });
  });

  it("derives finite type-union results through the shared domain", () => {
    expect(typeUnionResolution(["lg", "sm", "lg"])).toEqual({
      abstractValue: {
        kind: "finiteSet",
        values: ["lg", "sm"],
      },
      valueCertainty: "inferred",
      reason: "typeUnion",
    });
    expect(typeUnionResolution(["button"])).toEqual({
      abstractValue: {
        kind: "exact",
        value: "button",
      },
      valueCertainty: "exact",
      reason: "typeUnion",
    });
  });

  it("widens large type unions to a prefix when a meaningful LCP exists", () => {
    expect(
      typeUnionResolution([
        "btn-primary",
        "btn-secondary",
        "btn-danger",
        "btn-success",
        "btn-warning",
        "btn-info",
        "btn-muted",
        "btn-ghost",
        "btn-outline",
      ]),
    ).toEqual({
      abstractValue: {
        kind: "composite",
        prefix: "btn-",
        mustChars: "-bnt",
        mayChars: "-abcdefghilmnoprstuwy",
        minLength: 8,
        provenance: "finiteSetWideningComposite",
      },
      valueCertainty: "inferred",
      reason: "typeUnion",
    });
  });

  it("widens large type unions to top when no meaningful LCP exists", () => {
    expect(
      typeUnionResolution([
        "stateOne",
        "stateTwo",
        "stateThree",
        "stateFour",
        "stateFive",
        "stateSix",
        "stateSeven",
        "stateEight",
        "stateNine",
      ]),
    ).toEqual({
      abstractValue: {
        kind: "charInclusion",
        mustChars: "aest",
        mayChars: "EFNOSTaeghinorstuvwx",
        provenance: "finiteSetWideningChars",
      },
      valueCertainty: "inferred",
      reason: "typeUnion",
    });
  });
});
