import { describe, expect, it } from "vitest";
import { cssModulesClassnamesBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/binder-plugin";
import { cvaRecipeBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/cva-recipe-plugin";
import { tailwindUnoUtilityBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/tailwind-utility-plugin";
import { vanillaExtractRecipeBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/vanilla-extract-recipe-plugin";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import {
  readClassValueUniverseSummary,
  readDomainClassReferenceSummary,
} from "../../../server/engine-core-ts/src/core/query";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import {
  EMPTY_ALIAS_RESOLVER,
  createTestSourceFrontendAnalysis,
} from "../../_fixtures/test-helpers";

describe("readDomainClassReferenceSummary", () => {
  it("summarizes utility-domain class tracking separately from CSS Module references", () => {
    const binderPlugins = [
      cssModulesClassnamesBinderPluginV0,
      tailwindUnoUtilityBinderPluginV0,
      vanillaExtractRecipeBinderPluginV0,
      cvaRecipeBinderPluginV0,
    ];
    const cache = new DocumentAnalysisCache({
      sourceFileCache: new SourceFileCache({ max: 10 }),
      sourceFrontendAnalysis: createTestSourceFrontendAnalysis({
        fileExists: () => true,
        aliasResolver: EMPTY_ALIAS_RESOLVER,
        binderPlugins,
      }),
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });
    const entry = cache.get(
      "file:///fake/Card.tsx",
      `
        import classNames from 'classnames/bind';
        import clsx from 'clsx';
        import { recipe } from '@vanilla-extract/recipes';
        import { cva } from 'class-variance-authority';
        import styles from './Card.module.scss';
        const cardRecipe = recipe({
          base: "recipe_base",
          variants: {
            tone: {
              primary: "card_tone_primary",
            },
          },
          defaultVariants: {
            tone: "primary",
          },
        });
        const cvaRecipe = cva("cva_base", {
          variants: {
            tone: {
              primary: "cva_tone_primary",
            },
          },
        });
        const cx = classNames.bind(styles);
        const el = <div className={clsx(cx('card'), "flex", \`tone-\${state}\`)} />;
        const recipeClass = cardRecipe({ tone: "primary" });
        const cvaClass = cvaRecipe({ tone: "primary" });
      `,
      "/fake/Card.tsx",
      1,
    );

    const summary = readDomainClassReferenceSummary(entry.sourceDocument);
    const universeSummary = readClassValueUniverseSummary(entry.classValueUniverses);

    expect(entry.sourceDocument.classExpressions).toMatchObject([
      { kind: "literal", className: "card" },
    ]);
    expect(summary).toMatchObject({
      totalReferences: 4,
      hasUtilityDomainReferences: true,
      groups: [
        {
          pluginId: "cva-recipe-domain",
          domain: "cva-recipe",
          literalCount: 1,
          templatePrefixCount: 0,
        },
        {
          pluginId: "tailwind-uno-utility-domain",
          domain: "utility-css",
          literalCount: 1,
          templatePrefixCount: 1,
        },
        {
          pluginId: "vanilla-extract-recipe-domain",
          domain: "vanilla-extract-recipe",
          literalCount: 1,
          templatePrefixCount: 0,
        },
      ],
    });
    expect(universeSummary).toMatchObject({
      totalUniverses: 2,
      hasReducedProductUniverse: true,
      groups: [
        {
          pluginId: "cva-recipe-domain",
          domain: "cva-recipe",
          reducedProductCount: 1,
          classNames: ["cva_base", "cva_tone_primary"],
        },
        {
          pluginId: "vanilla-extract-recipe-domain",
          domain: "vanilla-extract-recipe",
          reducedProductCount: 1,
          classNames: ["card_tone_primary", "recipe_base"],
        },
      ],
    });
  });
});
