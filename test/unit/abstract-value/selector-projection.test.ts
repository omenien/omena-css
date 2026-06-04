import { describe, expect, it } from "vitest";
import {
  TOP_CLASS_VALUE,
  charInclusionClassValue,
  compositeClassValue,
  exactClassValue,
  finiteSetClassValue,
  prefixClassValue,
  prefixSuffixClassValue,
  suffixClassValue,
} from "../../../server/engine-core-ts/src/core/abstract-value/class-value-domain";
import { reducedProductClassValueUniverseV0 } from "../../../server/engine-core-ts/src/core/abstract-value/class-value-universe";
import {
  resolveAbstractValueClassNames,
  resolveAbstractValueSelectors,
} from "../../../server/engine-core-ts/src/core/abstract-value/selector-projection";
import { info } from "../../_fixtures/test-helpers";
import { buildStyleDocumentFromSelectorMap } from "../../_fixtures/style-documents";

const styleDocument = buildStyleDocumentFromSelectorMap(
  "/fake/ws/src/Button.module.scss",
  new Map([
    ["button", info("button")],
    ["btn-primary", info("btn-primary")],
    ["btn-secondary", info("btn-secondary")],
  ]),
);

describe("resolveAbstractValueSelectors", () => {
  it("projects exact values to canonical selectors", () => {
    expect(
      resolveAbstractValueSelectors(exactClassValue("button"), styleDocument).map(
        (selector) => selector.name,
      ),
    ).toEqual(["button"]);
  });

  it("projects finite sets to multiple selectors", () => {
    expect(
      resolveAbstractValueSelectors(
        finiteSetClassValue(["btn-secondary", "btn-primary"]),
        styleDocument,
      ).map((selector) => selector.name),
    ).toEqual(["btn-primary", "btn-secondary"]);
  });

  it("projects prefixes to matching canonical selectors", () => {
    expect(
      resolveAbstractValueSelectors(prefixClassValue("btn-"), styleDocument).map(
        (selector) => selector.name,
      ),
    ).toEqual(["btn-primary", "btn-secondary"]);
  });

  it("projects suffixes to matching canonical selectors", () => {
    expect(
      resolveAbstractValueSelectors(suffixClassValue("-primary"), styleDocument).map(
        (selector) => selector.name,
      ),
    ).toEqual(["btn-primary"]);
  });

  it("projects prefix-suffix products to matching canonical selectors", () => {
    expect(
      resolveAbstractValueSelectors(prefixSuffixClassValue("btn-", "-primary"), styleDocument).map(
        (selector) => selector.name,
      ),
    ).toEqual(["btn-primary"]);
  });

  it("projects character inclusion constraints to matching canonical selectors", () => {
    expect(
      resolveAbstractValueSelectors(
        charInclusionClassValue("-", "-abcdeimnoprstuy"),
        styleDocument,
      ).map((selector) => selector.name),
    ).toEqual(["btn-primary", "btn-secondary"]);
  });

  it("projects composite constraints to matching canonical selectors", () => {
    expect(
      resolveAbstractValueSelectors(
        compositeClassValue({
          prefix: "btn-",
          mustChars: "-btn",
          mayChars: "-abcdeimnoprstuy",
          provenance: "finiteSetWideningComposite",
        }),
        styleDocument,
      ).map((selector) => selector.name),
    ).toEqual(["btn-primary", "btn-secondary"]);
  });

  it("respects composite minLength constraints during projection", () => {
    expect(
      resolveAbstractValueSelectors(
        compositeClassValue({
          prefix: "btn-",
          minLength: 20,
          mustChars: "-btn",
          mayChars: "-abcdeimnoprstuy",
          provenance: "finiteSetWideningComposite",
        }),
        styleDocument,
      ).map((selector) => selector.name),
    ).toEqual([]);
  });

  it("treats top as the whole canonical selector universe", () => {
    expect(
      resolveAbstractValueSelectors(TOP_CLASS_VALUE, styleDocument)
        .map((selector) => selector.name)
        .toSorted(),
    ).toEqual(["btn-primary", "btn-secondary", "button"]);
  });

  it("projects abstract values over reduced-product class value universes", () => {
    const universe = reducedProductClassValueUniverseV0({
      baseClassNames: ["button_base"],
      axes: [
        {
          axisName: "tone",
          role: "variant",
          values: [
            { name: "primary", classNames: ["button_tone_primary"] },
            { name: "danger", classNames: ["button_tone_danger"] },
          ],
        },
        { axisName: "slots", role: "slot", reserved: true, values: [] },
      ],
      compoundVariants: [
        {
          conditions: [{ axisName: "tone", value: "primary" }],
          classNames: ["button_primary_compound"],
        },
      ],
    });

    expect(resolveAbstractValueClassNames(prefixClassValue("button_tone_"), universe)).toEqual([
      "button_tone_danger",
      "button_tone_primary",
    ]);
    expect(resolveAbstractValueClassNames(TOP_CLASS_VALUE, universe)).toEqual([
      "button_base",
      "button_primary_compound",
      "button_tone_danger",
      "button_tone_primary",
    ]);
  });

  it("uses owner-matched provider universes before falling back to the style document universe", () => {
    const recipeStyleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/ws/src/Button.module.scss",
      new Map([
        ["button_base", info("button_base")],
        ["button_tone_primary", info("button_tone_primary")],
        ["unrelated", info("unrelated")],
      ]),
    );
    const recipeUniverse = reducedProductClassValueUniverseV0({
      baseClassNames: ["button_base"],
      axes: [
        {
          axisName: "tone",
          role: "variant",
          values: [
            { name: "primary", classNames: ["button_tone_primary"] },
            { name: "danger", classNames: ["button_tone_danger"] },
          ],
        },
      ],
    });

    expect(
      resolveAbstractValueSelectors(TOP_CLASS_VALUE, recipeStyleDocument, {
        universeOwnerName: "button",
        classValueUniverses: [
          {
            id: "universe:button",
            pluginId: "cva-recipe-domain",
            domain: "cva-recipe",
            ownerName: "button",
            universe: recipeUniverse,
          },
        ],
      }).map((selector) => selector.name),
    ).toEqual(["button_base", "button_tone_primary"]);
  });
});
