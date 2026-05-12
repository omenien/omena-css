import { describe, expect, it } from "vitest";
import { buildStyleDocumentWithOmenaParser } from "../../../server/engine-host-node/src/omena-parser-style-document-builder";
import { SELECTED_QUERY_RUNNER_COMMANDS } from "../../../server/engine-host-node/src/selected-query-backend";

const range = (start: number, end: number) => ({
  start: { line: 0, character: start },
  end: { line: 0, character: end },
});

describe("buildStyleDocumentWithOmenaParser", () => {
  it("maps parser intermediate selector facts into style HIR", () => {
    const document = buildStyleDocumentWithOmenaParser(
      "/workspace/Button.module.scss",
      ".card { &__icon { color: red; } }",
      (command, input) => {
        expect(command).toBe(SELECTED_QUERY_RUNNER_COMMANDS.omenaParserCssModulesIntermediate);
        expect(input).toEqual({
          styleSource: ".card { &__icon { color: red; } }",
          dialect: "scss",
        });
        return {
          schemaVersion: "0",
          language: "scss",
          selectors: {
            definitionFacts: [
              {
                name: "card",
                sourceOrder: 0,
                range: range(1, 5),
                ruleRange: range(0, 33),
                fullSelector: ".card",
                declarations: "&__icon { color: red; }",
                nestedSafetyKind: "flat",
              },
              {
                name: "card__icon",
                sourceOrder: 1,
                range: range(9, 15),
                ruleRange: range(7, 31),
                fullSelector: "&__icon",
                declarations: "color: red;",
                nestedSafetyKind: "bemSuffixSafe",
                bemSuffixParentName: "card",
              },
            ],
          },
          values: { declFacts: [], importFacts: [], refFacts: [] },
          keyframes: { declFacts: [], refFacts: [] },
          customProperties: {
            declFacts: [
              {
                name: "--gap",
                value: "1rem",
                sourceOrder: 0,
                range: range(20, 25),
                ruleRange: range(0, 33),
                selectorContexts: [".card"],
                underMedia: false,
                underSupports: false,
                underLayer: false,
              },
            ],
            refFacts: [
              {
                name: "--gap",
                sourceOrder: 0,
                range: range(30, 35),
                selectorContexts: [".card"],
                underMedia: false,
                underSupports: false,
                underLayer: false,
              },
            ],
          },
          sass: {
            symbolDeclFacts: [],
            selectorSymbolFacts: [],
            moduleUseEdges: [],
            moduleForwardSources: [],
          },
          composes: {
            edges: [
              {
                kind: "external",
                ownerSelectorNames: ["card"],
                targetNames: ["base"],
                importSource: "./base.module.css",
              },
            ],
          },
        };
      },
    );

    expect(document.selectors.map((selector) => selector.name)).toEqual(["card", "card__icon"]);
    expect(document.selectors[0]).toMatchObject({
      canonicalName: "card",
      fullSelector: ".card",
      declarations: "&__icon { color: red; }",
      composes: [{ classNames: ["base"], from: "./base.module.css" }],
    });
    expect(document.selectors[1]).toMatchObject({
      canonicalName: "card__icon",
      nestedSafety: "bemSuffixSafe",
      bemSuffix: {
        rawToken: "&__icon",
        parentResolvedName: "card",
      },
    });
    expect(document.customPropertyDecls[0]?.context.selectorText).toBe(".card");
    expect(document.customPropertyDecls[0]?.value).toBe("1rem");
    expect(document.customPropertyDecls[0]?.ruleRange).toEqual(range(0, 33));
    expect(document.customPropertyRefs[0]?.range).toEqual(range(30, 35));
  });

  it("maps Sass module and selector symbol facts for runtime consumers", () => {
    const document = buildStyleDocumentWithOmenaParser(
      "/workspace/Button.module.scss",
      '@use "./tokens" as t; .card { color: t.$color; }',
      () => ({
        schemaVersion: "0",
        language: "scss",
        selectors: {
          definitionFacts: [
            {
              name: "card",
              sourceOrder: 0,
              range: range(23, 27),
              ruleRange: range(22, 47),
              fullSelector: ".card",
              declarations: "color: t.$color;",
              nestedSafetyKind: "flat",
            },
          ],
        },
        values: { declFacts: [], importFacts: [], refFacts: [] },
        keyframes: { declFacts: [], refFacts: [] },
        customProperties: { declFacts: [], refFacts: [] },
        sass: {
          symbolDeclFacts: [
            {
              symbolKind: "variable",
              name: "color",
              range: range(40, 46),
            },
          ],
          selectorSymbolFacts: [
            {
              selectorName: "card",
              symbolKind: "variable",
              name: "color",
              role: "reference",
              resolution: "external",
              range: range(40, 46),
            },
            {
              selectorName: "card",
              symbolKind: "mixin",
              name: "raised",
              namespace: "t",
              role: "include",
              resolution: "external",
              range: range(48, 54),
            },
          ],
          moduleUseEdges: [
            {
              source: "./tokens",
              namespaceKind: "alias",
              namespace: "t",
              range: range(0, 20),
            },
          ],
          moduleForwardSources: ["./theme"],
          moduleForwardEdges: [
            {
              source: "./theme",
              prefix: "theme-",
              visibilityKind: "show",
              visibilityMembers: [
                { name: "gap", symbolKind: "variable" },
                { name: "raised", symbolKind: null },
              ],
              range: range(58, 65),
              ruleRange: range(46, 88),
            },
          ],
        },
        composes: { edges: [] },
      }),
    );

    expect(document.sassModuleUses[0]).toMatchObject({
      source: "./tokens",
      namespaceKind: "alias",
      namespace: "t",
    });
    expect(document.sassSymbols[0]).toMatchObject({
      selectorName: "card",
      syntax: "sass",
      symbolKind: "variable",
      name: "color",
      role: "reference",
      resolution: "unresolved",
    });
    expect(document.sassModuleMemberRefs[0]).toMatchObject({
      selectorName: "card",
      namespace: "t",
      symbolKind: "mixin",
      name: "raised",
      role: "include",
    });
    expect(document.sassModuleForwards[0]).toMatchObject({
      source: "./theme",
      prefix: "theme-",
      visibilityKind: "show",
      visibilityMembers: [
        { name: "gap", symbolKind: "variable" },
        { name: "raised", symbolKind: null },
      ],
      range: range(58, 65),
    });
  });

  it("maps @value and keyframes facts for definitions, references, and diagnostics", () => {
    const document = buildStyleDocumentWithOmenaParser(
      "/workspace/Button.module.scss",
      '@value primary: #fff; @value secondary as accent from "./tokens.module.scss"; @keyframes fade { to { opacity: 1; } } .card { color: primary; animation: fade 1s; animation-name: missing; }',
      () => ({
        schemaVersion: "0",
        language: "scss",
        selectors: {
          definitionFacts: [
            {
              name: "card",
              sourceOrder: 0,
              range: range(124, 128),
              ruleRange: range(123, 183),
              fullSelector: ".card",
              declarations: "color: primary; animation: fade 1s; animation-name: missing;",
              nestedSafetyKind: "flat",
            },
          ],
        },
        values: {
          declFacts: [
            {
              name: "primary",
              value: "#fff",
              sourceOrder: 0,
              range: range(7, 14),
              ruleRange: range(0, 21),
            },
          ],
          importFacts: [
            {
              name: "accent",
              importedName: "secondary",
              from: "./tokens.module.scss",
              sourceOrder: 0,
              range: range(42, 48),
              importedNameRange: range(29, 38),
              ruleRange: range(22, 77),
            },
          ],
          refFacts: [
            {
              name: "primary",
              source: "declaration",
              sourceOrder: 0,
              range: range(139, 146),
            },
          ],
        },
        keyframes: {
          declFacts: [
            {
              name: "fade",
              sourceOrder: 0,
              range: range(89, 93),
              ruleRange: range(78, 119),
            },
          ],
          refFacts: [
            {
              name: "fade",
              property: "animation",
              sourceOrder: 0,
              range: range(158, 162),
            },
            {
              name: "missing",
              property: "animation-name",
              sourceOrder: 1,
              range: range(181, 188),
            },
          ],
        },
        customProperties: { declFacts: [], refFacts: [] },
        sass: {
          symbolDeclFacts: [],
          selectorSymbolFacts: [],
          moduleUseEdges: [],
          moduleForwardSources: [],
        },
        composes: { edges: [] },
      }),
    );

    expect(document.valueDecls[0]).toMatchObject({ name: "primary", value: "#fff" });
    expect(document.valueImports[0]).toMatchObject({
      name: "accent",
      importedName: "secondary",
      from: "./tokens.module.scss",
      importedNameRange: range(29, 38),
    });
    expect(document.valueRefs[0]).toMatchObject({ name: "primary", source: "declaration" });
    expect(document.keyframes[0]).toMatchObject({ name: "fade", ruleRange: range(78, 119) });
    expect(document.animationNameRefs.map((ref) => [ref.name, ref.property])).toEqual([
      ["fade", "animation"],
      ["missing", "animation-name"],
    ]);
  });
});
