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
          customProperties: {
            declFacts: [
              {
                name: "--gap",
                sourceOrder: 0,
                range: range(20, 25),
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
    expect(document.sassModuleForwards[0]).toMatchObject({
      source: "./theme",
      visibilityKind: "all",
    });
  });
});
