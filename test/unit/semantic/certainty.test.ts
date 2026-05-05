import { describe, expect, it } from "vitest";
import {
  deriveValueCertaintyProfile,
  deriveValueCertaintyProfileV2,
  deriveReferenceExpansion,
  deriveSelectorProjectionCertainty,
  deriveSelectorCertaintyProfileV2,
} from "../../../server/engine-core-ts/src/core/semantic/certainty";

describe("semantic/certainty", () => {
  it("keeps finite selector sets exact when the selector universe matches them exactly", () => {
    expect(
      deriveSelectorProjectionCertainty({ kind: "finiteSet", values: ["lg", "md", "sm"] }, 3, 4),
    ).toBe("exact");
  });

  it("keeps constrained projections inferred even when they cover the current selector universe", () => {
    expect(deriveSelectorProjectionCertainty({ kind: "prefix", prefix: "btn-" }, 2, 3)).toBe(
      "inferred",
    );
    expect(deriveSelectorProjectionCertainty({ kind: "prefix", prefix: "btn-" }, 3, 3)).toBe(
      "inferred",
    );
  });

  it("keeps top projections possible", () => {
    expect(deriveSelectorProjectionCertainty({ kind: "top" }, 3, 3)).toBe("possible");
  });

  it("derives current certainty profiles without changing the public certainty enum", () => {
    expect(
      deriveValueCertaintyProfile(
        { kind: "finiteSet", values: ["active", "indicator"] },
        "inferred",
      ),
    ).toEqual({
      certainty: "inferred",
      shapeKind: "boundedFinite",
      shapeLabel: "bounded finite (2)",
    });

    expect(deriveValueCertaintyProfile({ kind: "prefix", prefix: "btn-" }, "inferred")).toEqual({
      certainty: "inferred",
      shapeKind: "constrainedPrefix",
      shapeLabel: "constrained prefix `btn-`",
    });

    expect(deriveValueCertaintyProfile({ kind: "top" }, "possible")).toEqual({
      certainty: "possible",
      shapeKind: "unknown",
      shapeLabel: "unknown",
    });
  });

  it("derives bundle-1 V2 certainty profiles with constrained sub-kinds", () => {
    expect(deriveValueCertaintyProfileV2({ kind: "suffix", suffix: "-chip" }, "inferred")).toEqual({
      certainty: "inferred",
      shapeKind: "constrained",
      valueConstraintKind: "suffix",
      shapeLabel: "constrained suffix `-chip`",
    });

    expect(
      deriveValueCertaintyProfileV2(
        { kind: "prefixSuffix", prefix: "btn-", suffix: "-chip", minLength: 9 },
        "inferred",
      ),
    ).toEqual({
      certainty: "inferred",
      shapeKind: "constrained",
      valueConstraintKind: "prefixSuffix",
      shapeLabel: "constrained prefix `btn-` + suffix `-chip`",
    });

    expect(
      deriveSelectorCertaintyProfileV2(2, "inferred", {
        kind: "prefixSuffix",
        prefix: "btn-",
        suffix: "-chip",
        minLength: 9,
      }),
    ).toEqual({
      certainty: "inferred",
      shapeKind: "constrained",
      selectorConstraintKind: "prefixSuffix",
      shapeLabel: "constrained edge selector set (2)",
    });

    expect(
      deriveValueCertaintyProfileV2(
        {
          kind: "charInclusion",
          mustChars: "aest",
          mayChars: "EFNOSTaeghinorstuvwx",
          provenance: "finiteSetWideningChars",
        },
        "inferred",
      ),
    ).toEqual({
      certainty: "inferred",
      shapeKind: "constrained",
      valueConstraintKind: "charInclusion",
      shapeLabel: "constrained character inclusion (aest)",
    });

    expect(
      deriveSelectorCertaintyProfileV2(3, "inferred", {
        kind: "charInclusion",
        mustChars: "aest",
        mayChars: "EFNOSTaeghinorstuvwx",
        provenance: "finiteSetWideningChars",
      }),
    ).toEqual({
      certainty: "inferred",
      shapeKind: "constrained",
      selectorConstraintKind: "charInclusion",
      shapeLabel: "constrained character selector set (3)",
    });

    expect(
      deriveValueCertaintyProfileV2(
        {
          kind: "composite",
          prefix: "btn-",
          minLength: 8,
          mustChars: "-bnt",
          mayChars: "-abcdefghilmnoprstuwy",
          provenance: "finiteSetWideningComposite",
        },
        "inferred",
      ),
    ).toEqual({
      certainty: "inferred",
      shapeKind: "constrained",
      valueConstraintKind: "composite",
      shapeLabel: "constrained composite",
    });

    expect(
      deriveSelectorCertaintyProfileV2(4, "inferred", {
        kind: "composite",
        prefix: "btn-",
        minLength: 8,
        mustChars: "-bnt",
        mayChars: "-abcdefghilmnoprstuwy",
        provenance: "finiteSetWideningComposite",
      }),
    ).toEqual({
      certainty: "inferred",
      shapeKind: "constrained",
      selectorConstraintKind: "composite",
      shapeLabel: "constrained composite selector set (4)",
    });
  });

  it("keeps dynamic expressions expanded even when their selector certainty is exact", () => {
    expect(deriveReferenceExpansion("literal")).toBe("direct");
    expect(deriveReferenceExpansion("styleAccess")).toBe("direct");
    expect(deriveReferenceExpansion("template")).toBe("expanded");
    expect(deriveReferenceExpansion("symbolRef")).toBe("expanded");
  });
});
