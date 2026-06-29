import { strict as assert } from "node:assert";
import ts from "../server/engine-core-ts/src/ts-facade";
import {
  finiteSetClassValue,
  prefixClassValue,
  TOP_CLASS_VALUE,
} from "../server/engine-core-ts/src/core/abstract-value/class-value-domain";
import {
  classValueUniverseFromStyleDocument,
  classNamesForUniverse,
} from "../server/engine-core-ts/src/core/abstract-value/class-value-universe";
import {
  resolveAbstractValueClassNames,
  resolveAbstractValueSelectors,
} from "../server/engine-core-ts/src/core/abstract-value/selector-projection";
import {
  composeBinderPluginsV0,
  cssModulesClassnamesBinderPluginV0,
} from "../server/engine-core-ts/src/core/binder/binder-plugin";
import { cvaRecipeBinderPluginV0 } from "../server/engine-core-ts/src/core/binder/cva-recipe-plugin";
import { vanillaExtractRecipeBinderPluginV0 } from "../server/engine-core-ts/src/core/binder/vanilla-extract-recipe-plugin";
import { buildSourceBinder } from "../server/engine-core-ts/src/core/binder/binder-builder";
import { AliasResolver } from "../server/engine-core-ts/src/core/cx/alias-resolver";
import {
  makeStyleDocumentHIR,
  type SelectorDeclHIR,
} from "../server/engine-core-ts/src/core/hir/style-types";
import { DocumentAnalysisCache } from "../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import {
  readClassValueUniverseSummary,
  readDomainClassReferenceSummary,
} from "../server/engine-core-ts/src/core/query";
import { SourceFileCache } from "../server/engine-core-ts/src/core/ts/source-file-cache";
import { createRequiredRustSourceFrontendAnalysisProvider } from "../server/engine-host-node/src/source-frontend-analysis-provider";

const sourceText = `
  import classNames from 'classnames/bind';
  import { recipe } from '@vanilla-extract/recipes';
  import { cva } from 'class-variance-authority';
  import styles from './Card.module.scss';

  const cardRecipe = recipe({
    base: "recipe_base",
    variants: {
      tone: {
        primary: "recipe_tone_primary",
        danger: "recipe_tone_danger",
      },
      size: {
        sm: "recipe_size_sm",
        lg: "recipe_size_lg",
      },
    },
    compoundVariants: [
      { variants: { tone: "primary", size: "sm" }, style: "recipe_primary_sm" },
    ],
    defaultVariants: {
      tone: "primary",
    },
  });

  const cvaRecipe = cva("cva_base", {
    variants: {
      tone: {
        primary: "cva_tone_primary",
        danger: "cva_tone_danger",
      },
      size: {
        sm: "cva_size_sm",
        lg: "cva_size_lg",
      },
    },
    compoundVariants: [
      { tone: "primary", size: "sm", class: "cva_primary_sm" },
    ],
    defaultVariants: {
      size: "sm",
    },
  });

  const cx = classNames.bind(styles);
  const moduleClass = cx("card", "tone-primary");
  const recipeClass = cardRecipe({ tone: active ? "primary" : "danger", size: "sm" });
  const cvaClass = cvaRecipe({ tone: "primary", size: active ? "sm" : "lg" });
`;

const workspaceRoot = "/fake/ws";
const sourcePath = `${workspaceRoot}/src/Card.tsx`;
const sourceUri = "file:///fake/ws/src/Card.tsx";
const aliasResolver = new AliasResolver(workspaceRoot, {});
const sourceFile = ts.createSourceFile(
  sourcePath,
  sourceText,
  ts.ScriptTarget.Latest,
  true,
  ts.ScriptKind.TSX,
);
const sourceBinder = buildSourceBinder(sourceFile);
const binderPlugin = composeBinderPluginsV0([
  cssModulesClassnamesBinderPluginV0,
  vanillaExtractRecipeBinderPluginV0,
  cvaRecipeBinderPluginV0,
]);
const fileExists = (filePath: string) => filePath === `${workspaceRoot}/src/Card.module.scss`;
const sourceFrontendAnalysis = createRequiredRustSourceFrontendAnalysisProvider({
  aliasResolver: () => aliasResolver,
  fileExists,
});

const directBinderResult = binderPlugin.analyzeSource({
  sourceFile,
  filePath: sourcePath,
  sourceBinder,
  fileExists,
  aliasResolver,
});

const cache = new DocumentAnalysisCache({
  sourceFileCache: new SourceFileCache({ max: 10 }),
  sourceFrontendAnalysis,
  fileExists,
  aliasResolver,
  max: 10,
});
const analysisEntry = cache.get(sourceUri, sourceText, sourcePath, 1);
const styleDocument = makeStyleDocumentHIR(`${workspaceRoot}/src/Card.module.scss`, [
  selector("card", 1),
  selector("tone-primary", 5),
  selector("tone-danger", 9),
]);
const cssModulesUniverse = classValueUniverseFromStyleDocument(styleDocument);
const universeSummary = readClassValueUniverseSummary(analysisEntry.classValueUniverses);
const domainReferenceSummary = readDomainClassReferenceSummary(analysisEntry.sourceDocument);

assert.equal(cssModulesUniverse.kind, "finite");
assert.deepEqual(classNamesForUniverse(cssModulesUniverse), [
  "card",
  "tone-danger",
  "tone-primary",
]);
assert.deepEqual(
  resolveAbstractValueSelectors(finiteSetClassValue(["tone-primary", "card"]), styleDocument).map(
    (entry) => entry.name,
  ),
  ["card", "tone-primary"],
);

assert.equal(directBinderResult.stylesBindings.get("styles")?.kind, "resolved");
assert.deepEqual(
  directBinderResult.classExpressions.map((entry) => entry.kind),
  ["literal", "literal"],
);
assert.equal(directBinderResult.classValueUniverses.length, 2);
assert.deepEqual(analysisEntry.classValueUniverses, directBinderResult.classValueUniverses);
assert.equal(universeSummary.totalUniverses, 2);
assert.equal(universeSummary.hasReducedProductUniverse, true);
assert.equal(domainReferenceSummary.totalReferences, 6);

const vanillaUniverse = requiredUniverse("vanilla-extract-recipe-domain");
const cvaUniverse = requiredUniverse("cva-recipe-domain");
assertReducedProduct("vanilla-extract recipe", vanillaUniverse.universe, {
  baseClassNames: ["recipe_base"],
  classNames: [
    "recipe_base",
    "recipe_primary_sm",
    "recipe_size_lg",
    "recipe_size_sm",
    "recipe_tone_danger",
    "recipe_tone_primary",
  ],
  defaultAxis: ["tone", "primary"],
  prefixProjection: ["recipe_tone_danger", "recipe_tone_primary"],
});
assertReducedProduct("cva phase 1", cvaUniverse.universe, {
  baseClassNames: ["cva_base"],
  classNames: [
    "cva_base",
    "cva_primary_sm",
    "cva_size_lg",
    "cva_size_sm",
    "cva_tone_danger",
    "cva_tone_primary",
  ],
  defaultAxis: ["size", "sm"],
  prefixProjection: ["cva_tone_danger", "cva_tone_primary"],
});

process.stdout.write(
  JSON.stringify(
    {
      schemaVersion: "0",
      product: "release.m5-class-value-universe-matrix",
      fixtures: [
        {
          domain: "css-modules",
          universeKind: cssModulesUniverse.kind,
          classNames: classNamesForUniverse(cssModulesUniverse),
          evidence: "StyleDocumentHIR finite fallback plus selector projection",
        },
        {
          domain: vanillaUniverse.domain,
          universeKind: vanillaUniverse.universe.kind,
          classNames: classNamesForUniverse(vanillaUniverse.universe),
          evidence: "BinderPluginV0 -> DocumentAnalysisCache -> query summary",
        },
        {
          domain: cvaUniverse.domain,
          universeKind: cvaUniverse.universe.kind,
          classNames: classNamesForUniverse(cvaUniverse.universe),
          evidence: "BinderPluginV0 -> DocumentAnalysisCache -> query summary",
        },
      ],
      sharedAxes: ["base", "variants", "compoundVariants", "defaultVariants"],
      slots: {
        phase1: "reserved-deferred",
        evidencedByReservedAxis: true,
      },
      propagation: {
        composedBinder: true,
        documentAnalysisCache: true,
        querySummary: true,
        selectorProjection: true,
      },
    },
    null,
    2,
  ),
);
process.stdout.write("\n");

function requiredUniverse(pluginId: string) {
  const entry = analysisEntry.classValueUniverses.find(
    (candidate) => candidate.pluginId === pluginId,
  );
  assert.ok(entry, `missing class-value universe entry for ${pluginId}`);
  return entry;
}

function assertReducedProduct(
  label: string,
  universe: typeof vanillaUniverse.universe,
  expected: {
    readonly baseClassNames: readonly string[];
    readonly classNames: readonly string[];
    readonly defaultAxis: readonly [string, string];
    readonly prefixProjection: readonly string[];
  },
): void {
  assert.equal(universe.kind, "reduced-product", `${label}: expected reduced product`);
  assert.deepEqual(universe.baseClassNames, expected.baseClassNames, `${label}: base class names`);
  assert.deepEqual(classNamesForUniverse(universe), expected.classNames, `${label}: class names`);
  assert.ok(
    universe.compoundVariants.some((entry) => entry.classNames.length === 1),
    `${label}: missing compound variant class`,
  );
  assert.ok(
    universe.axes.some(
      (axis) =>
        axis.axisName === expected.defaultAxis[0] && axis.defaultValue === expected.defaultAxis[1],
    ),
    `${label}: missing default variant axis`,
  );
  assert.ok(
    universe.axes.some(
      (axis) => axis.axisName === "slots" && axis.role === "slot" && axis.reserved,
    ),
    `${label}: missing reserved slots axis`,
  );
  assert.deepEqual(
    resolveAbstractValueClassNames(
      prefixClassValue(`${expected.baseClassNames[0]!.split("_")[0]}_tone_`),
      universe,
    ),
    expected.prefixProjection,
    `${label}: prefix projection`,
  );
  assert.deepEqual(
    classNamesForUniverse(universe),
    resolveAbstractValueClassNames(TOP_CLASS_VALUE, universe),
  );
}

function selector(name: string, line: number): SelectorDeclHIR {
  return {
    kind: "selector",
    id: `selector:${line}:${name}`,
    name,
    canonicalName: name,
    viewKind: "canonical",
    range: { start: { line, character: 1 }, end: { line, character: 1 + name.length } },
    fullSelector: `.${name}`,
    declarations: "color: red",
    ruleRange: { start: { line, character: 0 }, end: { line: line + 2, character: 1 } },
    composes: [],
    nestedSafety: "flat",
  };
}
