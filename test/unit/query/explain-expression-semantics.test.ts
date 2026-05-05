import { describe, expect, it } from "vitest";
import {
  describeAbstractValue,
  describeAbstractValueReason,
  describeSelectorCertaintyReason,
  describeValueCertaintyReason,
  messageForInvalidClassFinding,
} from "../../../server/engine-core-ts/src/core/query/explain-expression-semantics";

describe("describeAbstractValue", () => {
  it("labels widened prefixes explicitly", () => {
    expect(
      describeAbstractValue({
        kind: "prefix",
        prefix: "btn-",
        provenance: "finiteSetWidening",
      }),
    ).toBe("prefix `btn-` (widened)");
  });

  it("keeps non-widened prefixes plain", () => {
    expect(
      describeAbstractValue({
        kind: "prefix",
        prefix: "btn-",
        provenance: "concatUnknownRight",
      }),
    ).toBe("prefix `btn-`");
  });

  it("explains prefix provenance reasons", () => {
    expect(
      describeAbstractValueReason({
        kind: "prefix",
        prefix: "btn-",
        provenance: "finiteSetConcatPrefixLcp",
      }),
    ).toBe("finite candidates concatenated with a prefix and reduced to their shared prefix");

    expect(
      describeAbstractValueReason({
        kind: "prefix",
        prefix: "btn-",
        provenance: "concatUnknownRight",
      }),
    ).toBe("known prefix preserved while concatenating an unknown suffix");

    expect(
      describeAbstractValueReason({
        kind: "suffix",
        suffix: "-chip",
        provenance: "concatUnknownLeft",
      }),
    ).toBe("known suffix preserved while prepending an unknown prefix");

    expect(
      describeAbstractValueReason({
        kind: "prefixSuffix",
        prefix: "btn-",
        suffix: "-chip",
        minLength: 9,
        provenance: "concatKnownEdges",
      }),
    ).toBe("known prefix and suffix were preserved across concatenation");

    expect(
      describeAbstractValueReason({
        kind: "charInclusion",
        mustChars: "-",
        mayChars: "-012abc",
        provenance: "finiteSetWideningChars",
      }),
    ).toBe("finite candidates widened to shared character inclusion constraints");

    expect(
      describeAbstractValueReason({
        kind: "composite",
        prefix: "btn-",
        minLength: 5,
        mustChars: "-btn",
        mayChars: "-012345678btn",
        provenance: "finiteSetWideningComposite",
      }),
    ).toBe("finite candidates widened to shared edge and character constraints");

    expect(
      describeAbstractValueReason({
        kind: "composite",
        suffix: "-chip",
        minLength: 5,
        mustChars: "-chipst",
        mayChars: "-chipst",
        mayIncludeOtherChars: true,
        provenance: "compositeConcat",
      }),
    ).toBe("concatenation preserved edge and character constraints together");
  });

  it("explains inferred and possible certainty from domain provenance", () => {
    expect(
      describeValueCertaintyReason(
        {
          kind: "prefix",
          prefix: "btn-",
          provenance: "finiteSetWidening",
        },
        "inferred",
        "flowBranch",
      ),
    ).toBe("finite candidates widened to a shared prefix");

    expect(
      describeValueCertaintyReason(
        {
          kind: "finiteSet",
          values: ["active", "indicator"],
        },
        "inferred",
        "typeUnion",
      ),
    ).toBe("TypeScript exposed multiple string-literal candidates");

    expect(
      describeValueCertaintyReason(
        {
          kind: "suffix",
          suffix: "-chip",
          provenance: "concatUnknownLeft",
        },
        "inferred",
        "flowLiteral",
      ),
    ).toBe("known suffix preserved while prepending an unknown prefix");

    expect(
      describeValueCertaintyReason(
        {
          kind: "prefixSuffix",
          prefix: "btn-",
          suffix: "-chip",
          minLength: 9,
          provenance: "concatKnownEdges",
        },
        "inferred",
        "flowLiteral",
      ),
    ).toBe("known prefix and suffix were preserved across concatenation");

    expect(
      describeValueCertaintyReason(
        {
          kind: "charInclusion",
          mustChars: "-",
          mayChars: "-012abc",
          provenance: "finiteSetWideningChars",
        },
        "inferred",
        "flowBranch",
      ),
    ).toBe("finite candidates widened to shared character inclusion constraints");

    expect(
      describeValueCertaintyReason(
        {
          kind: "composite",
          prefix: "btn-",
          minLength: 5,
          mustChars: "-btn",
          mayChars: "-012345678btn",
          provenance: "finiteSetWideningComposite",
        },
        "inferred",
        "flowBranch",
      ),
    ).toBe("finite candidates widened to shared edge and character constraints");

    expect(describeValueCertaintyReason({ kind: "top" }, "possible", "flowBranch")).toBe(
      "analysis lost finite shape information for this value",
    );
  });

  it("explains selector certainty from value domain and match count", () => {
    expect(
      describeSelectorCertaintyReason(
        {
          kind: "prefix",
          prefix: "btn-",
          provenance: "finiteSetWidening",
        },
        "inferred",
        3,
      ),
    ).toBe("finite candidates widened to a shared prefix");

    expect(
      describeSelectorCertaintyReason(
        {
          kind: "suffix",
          suffix: "-chip",
          provenance: "concatUnknownLeft",
        },
        "inferred",
        2,
      ),
    ).toBe("known suffix preserved while prepending an unknown prefix");

    expect(
      describeSelectorCertaintyReason(
        {
          kind: "prefixSuffix",
          prefix: "btn-",
          suffix: "-chip",
          minLength: 9,
          provenance: "concatKnownEdges",
        },
        "inferred",
        3,
      ),
    ).toBe("known prefix and suffix were preserved across concatenation");

    expect(
      describeSelectorCertaintyReason(
        {
          kind: "charInclusion",
          mustChars: "-",
          mayChars: "-012abc",
          provenance: "finiteSetWideningChars",
        },
        "inferred",
        3,
      ),
    ).toBe("finite candidates widened to shared character inclusion constraints");

    expect(
      describeSelectorCertaintyReason(
        {
          kind: "composite",
          prefix: "btn-",
          minLength: 5,
          mustChars: "-btn",
          mayChars: "-012345678btn",
          provenance: "finiteSetWideningComposite",
        },
        "inferred",
        3,
      ),
    ).toBe("finite candidates widened to shared edge and character constraints");

    expect(
      describeSelectorCertaintyReason(
        {
          kind: "finiteSet",
          values: ["small", "large"],
        },
        "inferred",
        1,
      ),
    ).toBe("finite candidate values matched a bounded selector set");

    expect(describeSelectorCertaintyReason({ kind: "top" }, "possible", 0)).toBe(
      "no selector could be proven for this value",
    );
  });

  it("renders constrained missing domain diagnostics with concrete constraints", () => {
    expect(
      messageForInvalidClassFinding({
        kind: "missingResolvedClassDomain",
        expression: null as never,
        range: null as never,
        abstractValue: {
          kind: "composite",
          prefix: "btn-",
          suffix: "-active",
          minLength: 16,
          mustChars: "-abceintv",
          mayChars: "-abceinprtv",
          provenance: "compositeConcat",
        },
        valueCertainty: "inferred",
        selectorCertainty: "possible",
        reason: "flowBranch",
      }),
    ).toContain(
      "No class matched resolved composite constraints (prefix 'btn-', suffix '-active', min length 16, must include '-abceintv', may only include '-abceinprtv').",
    );
  });
});
