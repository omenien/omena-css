import { describe, it, expect } from "vitest";
import type { Rule } from "postcss";
import {
  buildChildContext,
  enumerateGroups,
  findBemSuffixSpan,
  parseStyleDocument,
  styleDocumentSemanticFingerprint,
  StyleIndexCache,
} from "../../../server/engine-core-ts/src/core/scss/scss-index";
import { parseStyleSelectorMap, selectorMapFromDocument } from "../../_fixtures/style-documents";

describe("parseStyleSelectorMap / flat classes", () => {
  it("extracts a single flat class", () => {
    const map = parseStyleSelectorMap(
      `.button { color: red; padding: 8px; }`,
      "/fake/a.module.scss",
    );
    expect(map.has("button")).toBe(true);
    const info = map.get("button")!;
    expect(info.name).toBe("button");
    expect(info.fullSelector).toBe(".button");
    expect(info.declarations).toContain("color: red");
    expect(info.declarations).toContain("padding: 8px");
  });

  it("extracts multiple flat classes", () => {
    const map = parseStyleSelectorMap(
      `.one { color: red; }\n.two { color: blue; }\n.three { color: green; }`,
      "/fake/a.module.scss",
    );
    expect(Array.from(map.keys())).toEqual(["one", "two", "three"]);
  });

  it("returns an empty map for files with no class selectors", () => {
    const map = parseStyleSelectorMap(
      `:root { --bg: red; }\nbody { margin: 0; }`,
      "/fake/a.module.scss",
    );
    expect(map.size).toBe(0);
  });

  it("returns an empty map on parse error without throwing", () => {
    // Unterminated block — postcss throws; parser must catch.
    const map = parseStyleSelectorMap(`.broken { color: `, "/fake/a.module.scss");
    expect(map.size).toBe(0);
  });

  it("uses vanilla postcss for .module.css (no SCSS syntax)", () => {
    const map = parseStyleSelectorMap(`.plain { color: red; }`, "/fake/a.module.css");
    expect(map.has("plain")).toBe(true);
  });

  it("records the class token range (0-based)", () => {
    const map = parseStyleSelectorMap(`.indicator { color: red; }`, "/fake/a.module.scss");
    const info = map.get("indicator")!;
    // ".indicator" starts at col 0; the class name 'indicator' starts at col 1
    expect(info.range.start.line).toBe(0);
    expect(info.range.start.character).toBe(1);
    expect(info.range.end.character).toBe(1 + "indicator".length);
  });

  it("records the full rule block range (ruleRange)", () => {
    const map = parseStyleSelectorMap(`.indicator {\n  color: red;\n}`, "/fake/a.module.scss");
    const info = map.get("indicator")!;
    expect(info.ruleRange.start.line).toBe(0);
    expect(info.ruleRange.end.line).toBeGreaterThanOrEqual(2);
  });

  it("records wrapper at-rule context for selectors", () => {
    const document = parseStyleDocument(
      `@media (min-width: 600px) {
  @supports (display: grid) {
    .indicator { color: red; }
  }
}`,
      "/fake/a.module.scss",
    );

    expect(document.selectors[0]).toMatchObject({
      name: "indicator",
      context: {
        wrapperAtRules: [
          { name: "media", params: "(min-width: 600px)" },
          { name: "supports", params: "(display: grid)" },
        ],
      },
    });
  });

  it("records composes declaration ranges", () => {
    const source = `.button {
  composes: base from "./base.module.scss";
  color: red;
}`;
    const document = parseStyleDocument(source, "/fake/a.module.scss");
    const ref = document.selectors[0]!.composes[0]!;

    expect(ref.range).toEqual({
      start: { line: 1, character: 2 },
      end: { line: 1, character: 42 },
    });
    expect(sliceRange(source, ref.range!)).toBe(`composes: base from "./base.module.scss"`);
  });
});

describe("parseStyleDocument / CSS custom properties", () => {
  it("captures custom property declarations and var() references", () => {
    const document = parseStyleDocument(
      `:root { --color-gray-700: #767678; }
.title { color: var(--color-gray-700); }`,
      "/fake/tokens.module.css",
    );

    expect(document.customPropertyDecls).toMatchObject([
      {
        name: "--color-gray-700",
        value: "#767678",
        range: {
          start: { line: 0, character: 8 },
          end: { line: 0, character: 24 },
        },
        context: {
          containerKind: "rule",
          selectorText: ":root",
          atRuleName: null,
          atRuleParams: null,
          wrapperAtRules: [],
        },
      },
    ]);
    expect(document.customPropertyRefs).toMatchObject([
      {
        name: "--color-gray-700",
        range: {
          start: { line: 1, character: 20 },
          end: { line: 1, character: 36 },
        },
        context: {
          containerKind: "rule",
          selectorText: ".title",
          atRuleName: null,
          atRuleParams: null,
          wrapperAtRules: [],
        },
      },
    ]);
  });

  it("captures custom property declaration wrapper context", () => {
    const document = parseStyleDocument(
      `@layer theme {
  @media (prefers-color-scheme: dark) {
    [data-theme="dark"] { --brand: #fff; }
  }
}`,
      "/fake/tokens.module.css",
    );

    expect(document.customPropertyDecls).toMatchObject([
      {
        name: "--brand",
        value: "#fff",
        context: {
          containerKind: "rule",
          selectorText: `[data-theme="dark"]`,
          atRuleName: null,
          atRuleParams: null,
          wrapperAtRules: [
            { name: "layer", params: "theme" },
            { name: "media", params: "(prefers-color-scheme: dark)" },
          ],
        },
      },
    ]);
  });

  it("captures custom property reference wrapper context", () => {
    const document = parseStyleDocument(
      `@media (min-width: 600px) {
  [data-theme="dark"] .title { color: var(--brand); }
}`,
      "/fake/theme.module.css",
    );

    expect(document.customPropertyRefs).toMatchObject([
      {
        name: "--brand",
        context: {
          containerKind: "rule",
          selectorText: `[data-theme="dark"] .title`,
          atRuleName: null,
          atRuleParams: null,
          wrapperAtRules: [{ name: "media", params: "(min-width: 600px)" }],
        },
      },
    ]);
  });
});

describe("parseStyleSelectorMap / edge cases", () => {
  describe(":global() wrapping", () => {
    it("excludes :global(.foo) from the class map", () => {
      const map = parseStyleSelectorMap(`:global(.btn) { color: red; }`, "/fake/a.module.scss");
      expect(map.has("btn")).toBe(false);
    });

    it("still includes sibling local classes in the same file", () => {
      const map = parseStyleSelectorMap(
        `:global(.btn) { color: red; }\n.card { padding: 8px; }`,
        "/fake/a.module.scss",
      );
      expect(map.has("btn")).toBe(false);
      expect(map.has("card")).toBe(true);
    });
  });

  describe(":local() wrapping", () => {
    it("includes :local(.foo) as 'foo'", () => {
      const map = parseStyleSelectorMap(`:local(.btn) { color: red; }`, "/fake/a.module.scss");
      expect(map.has("btn")).toBe(true);
    });
  });

  describe("& nesting", () => {
    it("resolves .button { &--primary { ... } } to button--primary", () => {
      const map = parseStyleSelectorMap(
        `.button { color: red; &--primary { background: blue; } }`,
        "/fake/a.module.scss",
      );
      expect(map.has("button")).toBe(true);
      expect(map.has("button--primary")).toBe(true);
    });

    it("handles deeply nested ampersand", () => {
      const map = parseStyleSelectorMap(
        `.card { &__header { &--large { font-size: 20px; } } }`,
        "/fake/a.module.scss",
      );
      expect(map.has("card__header--large")).toBe(true);
    });

    it("handles plain nested selectors without &", () => {
      const map = parseStyleSelectorMap(
        `.wrapper { .inner { color: red; } }`,
        "/fake/a.module.scss",
      );
      // Only 'inner' is exposed on the styles object (last class wins).
      expect(map.has("inner")).toBe(true);
      expect(map.get("inner")!.nestedSafety).toBe("nestedUnsafe");
    });

    it("preserves repeated nested definitions for the same class name", () => {
      const document = parseStyleDocument(
        `.panel {
  .action { color: red; }
  button.action { color: blue; }
}
.drawer {
  .action { color: green; }
}`,
        "/fake/a.module.scss",
      );
      expect(document.selectors.filter((selector) => selector.name === "action")).toMatchObject([
        { fullSelector: ".panel .action", nestedSafety: "nestedUnsafe" },
        { fullSelector: ".panel button.action", nestedSafety: "nestedUnsafe" },
        { fullSelector: ".drawer .action", nestedSafety: "nestedUnsafe" },
      ]);
    });

    it("keeps the flat parent in flat nested-safety when nested is '&:hover'", () => {
      const map = parseStyleSelectorMap(
        `.button { color: red; &:hover { color: blue; } }`,
        "/fake/a.module.scss",
      );
      expect(map.has("button")).toBe(true);
      // `.button { &:hover {} }` must keep `.button` as a flat entry
      // so rename is not silently rejected on this BEM+SCSS shape.
      expect(map.get("button")!.nestedSafety).toBe("flat");
    });

    it("&--suffix: flat parent stays flat, BEM variant is nested", () => {
      const map = parseStyleSelectorMap(
        `.button { color: red; &--primary { background: blue; } }`,
        "/fake/a.module.scss",
      );
      expect(map.get("button")!.nestedSafety).toBe("flat");
      expect(map.get("button--primary")!.nestedSafety).toBe("bemSuffixSafe");
    });

    it("&.active: flat parent stays flat, compound sibling is nested", () => {
      const map = parseStyleSelectorMap(
        `.button { color: red; &.active { color: blue; } }`,
        "/fake/a.module.scss",
      );
      expect(map.get("button")!.nestedSafety).toBe("flat");
      // `active` is a brand-new class introduced inside the nested rule,
      // so it inherits the nested flag.
      expect(map.get("active")!.nestedSafety).toBe("nestedUnsafe");
    });

    it("recognizes single-dash ampersand suffixes as nested classes", () => {
      const map = parseStyleSelectorMap(
        `.status { &-active { color: green; } &-error { color: red; } }`,
        "/fake/a.module.scss",
      );
      expect(map.get("status-active")?.fullSelector).toBe(".status-active");
      expect(map.get("status-active")?.nestedSafety).toBe("nestedUnsafe");
      expect(map.get("status-active")?.bemSuffix).toBeUndefined();
      expect(map.get("status-error")?.fullSelector).toBe(".status-error");
      expect(map.get("status-error")?.nestedSafety).toBe("nestedUnsafe");
    });

    it("recognizes non-BEM ampersand suffix continuations without widening rename safety", () => {
      const map = parseStyleSelectorMap(
        `.status { &_active { color: green; } &suffix { color: blue; } }`,
        "/fake/a.module.scss",
      );
      expect(map.get("status_active")?.fullSelector).toBe(".status_active");
      expect(map.get("status_active")?.nestedSafety).toBe("nestedUnsafe");
      expect(map.get("status_active")?.bemSuffix).toBeUndefined();
      expect(map.get("statussuffix")?.fullSelector).toBe(".statussuffix");
      expect(map.get("statussuffix")?.nestedSafety).toBe("nestedUnsafe");
      expect(map.get("statussuffix")?.bemSuffix).toBeUndefined();
    });

    it("keeps nested suffix continuations as future bare BEM bases only after direct ampersand expansion", () => {
      const map = parseStyleSelectorMap(`.btn { &-legacy { &--x {} } }`, "/fake/a.module.scss");
      expect(map.get("btn-legacy")?.nestedSafety).toBe("nestedUnsafe");
      expect(map.get("btn-legacy")?.bemSuffix).toBeUndefined();
      expect(map.get("btn-legacy--x")?.nestedSafety).toBe("bemSuffixSafe");
      expect(map.get("btn-legacy--x")?.bemSuffix?.parentResolvedName).toBe("btn-legacy");
    });

    it("does not expand BEM suffixes from descendant-nested parents", () => {
      const map = parseStyleSelectorMap(`.wrapper { .inner { &--mod {} } }`, "/fake/a.module.scss");
      expect(map.has("inner--mod")).toBe(false);
      expect(map.get("inner")?.nestedSafety).toBe("nestedUnsafe");
    });

    it("does not treat selector syntax after `&` as a class suffix continuation", () => {
      const map = parseStyleSelectorMap(
        `.status { &:hover {} &[aria-current] {} & + .peer {} }`,
        "/fake/a.module.scss",
      );
      expect(map.get("status")?.nestedSafety).toBe("flat");
      expect(map.has("statushover")).toBe(false);
      expect(map.has("statusaria-current")).toBe(false);
      expect(map.has("statuspeer")).toBe(false);
      expect(map.has("peer")).toBe(true);
      expect(map.get("peer")?.nestedSafety).toBe("nestedUnsafe");
    });

    it("registers classes introduced alongside `&` compounds", () => {
      const map = parseStyleSelectorMap(
        `.item { &.type-card { &.compact .body { max-width: 320px; } } }`,
        "/fake/a.module.scss",
      );
      expect(map.get("compact")?.fullSelector).toBe(".item.type-card.compact .body");
      expect(map.get("body")?.fullSelector).toBe(".item.type-card.compact .body");
    });

    it("does not overwrite earlier nested parent classes from later `&.class` compounds", () => {
      const map = parseStyleSelectorMap(
        `.item { &.type-inline { color: red; &.disabled { opacity: .5; } } }`,
        "/fake/a.module.scss",
      );
      expect(map.get("type-inline")?.fullSelector).toBe(".item.type-inline");
      expect(map.get("disabled")?.fullSelector).toBe(".item.type-inline.disabled");
    });
  });

  describe("group selectors", () => {
    it("indexes each class in '.a, .b'", () => {
      const map = parseStyleSelectorMap(
        `.primary, .secondary { color: red; }`,
        "/fake/a.module.scss",
      );
      expect(map.has("primary")).toBe(true);
      expect(map.has("secondary")).toBe(true);
    });

    it("shares declarations across grouped selectors", () => {
      const map = parseStyleSelectorMap(
        `.a, .b { color: red; font-size: 14px; }`,
        "/fake/a.module.scss",
      );
      expect(map.get("a")!.declarations).toContain("color: red");
      expect(map.get("b")!.declarations).toContain("color: red");
    });
  });

  describe("compound selectors", () => {
    it("indexes every class in '.foo.bar { ... }'", () => {
      const map = parseStyleSelectorMap(`.foo.bar { color: red; }`, "/fake/a.module.scss");
      expect(map.has("foo")).toBe(true);
      expect(map.has("bar")).toBe(true);
    });

    it("indexes only the rightmost compound segment in '.a .b.c'", () => {
      // CSS Modules exposes every class in the rightmost compound
      // segment. `.a` is an ancestor; `.b.c` is the compound that
      // keys the exported name.
      const map = parseStyleSelectorMap(`.a .b.c { color: red; }`, "/fake/a.module.scss");
      expect(map.has("b")).toBe(true);
      expect(map.has("c")).toBe(true);
      expect(map.has("a")).toBe(false);
    });
  });

  describe("token range word-boundary", () => {
    it("points to the standalone '.btn' in '.btn-primary .btn'", () => {
      const map = parseStyleSelectorMap(`.btn-primary .btn { color: red; }`, "/fake/a.module.scss");
      const btnInfo = map.get("btn")!;
      // '.btn-primary ' is 13 characters. '.btn' starts at column 13,
      // the class name 'btn' starts at column 14.
      expect(btnInfo.range.start.character).toBe(14);
      expect(btnInfo.range.end.character).toBe(14 + "btn".length);
    });

    it("does not collide class prefixes when .btn-primary is first", () => {
      const map = parseStyleSelectorMap(
        `.btn-primary { color: red; }\n.btn { color: blue; }`,
        "/fake/a.module.scss",
      );
      expect(map.get("btn")!.range.start.line).toBe(1);
      expect(map.get("btn-primary")!.range.start.line).toBe(0);
    });
  });

  describe("partial parse recovery", () => {
    it("returns an empty map when a late syntax error breaks the whole file", () => {
      // postcss throws on the broken trailing rule; parse failure
      // returns an empty map — no partial results.
      const map = parseStyleSelectorMap(
        `.valid { color: red; }\n.broken { color: `,
        "/fake/a.module.scss",
      );
      expect(map.size).toBe(0);
    });
  });

  describe("CSS variables", () => {
    it("includes --var-name declarations in the declarations text", () => {
      const map = parseStyleSelectorMap(
        `.theme { --bg: red; --fg: white; }`,
        "/fake/a.module.scss",
      );
      expect(map.get("theme")!.declarations).toContain("--bg: red");
      expect(map.get("theme")!.declarations).toContain("--fg: white");
    });
  });

  describe("cascade last-wins", () => {
    it("uses the last declaration when a class is defined multiple times", () => {
      const map = parseStyleSelectorMap(
        `.btn { color: red; }\n.btn { color: blue; }`,
        "/fake/a.module.scss",
      );
      const info = map.get("btn")!;
      expect(info.declarations).toContain("color: blue");
      expect(info.declarations).not.toContain("color: red");
    });
  });

  describe("@keyframes / @font-face exclusion", () => {
    it("does not index identifiers inside @keyframes", () => {
      const document = parseStyleDocument(
        `@keyframes fade { from { opacity: 0; } to { opacity: 1; } }`,
        "/fake/a.module.scss",
      );
      const map = selectorMapFromDocument(document);
      expect(map.has("fade")).toBe(false);
      expect(map.has("from")).toBe(false);
      expect(map.has("to")).toBe(false);
      expect(document.keyframes.map((keyframes) => keyframes.name)).toEqual(["fade"]);
    });

    it("records animation name references from animation declarations", () => {
      const document = parseStyleDocument(
        `@keyframes fade { from { opacity: 0; } to { opacity: 1; } }
.spinner { animation: fade 1s linear; }
.pulse { animation-name: fade; }`,
        "/fake/a.module.scss",
      );
      expect(document.keyframes.map((keyframes) => keyframes.name)).toEqual(["fade"]);
      expect(document.animationNameRefs.map((ref) => [ref.name, ref.property])).toEqual([
        ["fade", "animation"],
        ["fade", "animation-name"],
      ]);
    });

    it("does not confuse @font-face blocks for rules", () => {
      const map = parseStyleSelectorMap(
        `@font-face { font-family: 'Inter'; src: url('x.woff2'); }`,
        "/fake/a.module.scss",
      );
      expect(map.size).toBe(0);
    });
  });

  describe("@value first pass", () => {
    it("records local @value declarations and same-file refs", () => {
      const document = parseStyleDocument(
        `@value primary: #ff3355;
@value accent: primary;
.button { color: primary; border-color: accent; }
.copy { color: "primary"; }`,
        "/fake/a.module.scss",
      );
      expect(document.valueDecls.map((valueDecl) => [valueDecl.name, valueDecl.value])).toEqual([
        ["primary", "#ff3355"],
        ["accent", "primary"],
      ]);
      expect(document.valueRefs.map((valueRef) => [valueRef.name, valueRef.source])).toEqual([
        ["primary", "valueDecl"],
        ["primary", "declaration"],
        ["accent", "declaration"],
      ]);
    });

    it("records imported value bindings from direct and aliased sources", () => {
      const document = parseStyleDocument(
        `@value colors: "./colors.module.scss";
@value primary, secondary as accent from colors;
@value danger from "./tokens.module.scss";
.button { color: primary; border-color: accent; background: danger; }`,
        "/fake/a.module.scss",
      );
      expect(document.valueDecls).toEqual([]);
      expect(
        document.valueImports.map((valueImport) => [
          valueImport.name,
          valueImport.importedName,
          valueImport.from,
          valueImport.importedNameRange?.start.character ?? null,
        ]),
      ).toEqual([
        ["primary", "primary", "./colors.module.scss", null],
        ["accent", "secondary", "./colors.module.scss", 16],
        ["danger", "danger", "./tokens.module.scss", null],
      ]);
      expect(document.valueRefs.map((valueRef) => valueRef.name)).toEqual([
        "primary",
        "accent",
        "danger",
      ]);
    });
  });

  describe("Sass module uses", () => {
    it("records @use namespace edges with source ranges", () => {
      const source = `@use "./plain";
@use "./reset" as *;
@use "./tokens" as tokens;
@use "sass:color";
@use "../theme/_spacing.scss?raw";`;
      const document = parseStyleDocument(source, "/fake/a.module.scss");

      expect(
        document.sassModuleUses.map((moduleUse) => [
          moduleUse.source,
          moduleUse.namespaceKind,
          moduleUse.namespace,
          sliceRange(source, moduleUse.range),
        ]),
      ).toEqual([
        ["./plain", "default", "plain", "./plain"],
        ["./reset", "wildcard", null, "./reset"],
        ["./tokens", "alias", "tokens", "./tokens"],
        ["sass:color", "default", "color", "sass:color"],
        ["../theme/_spacing.scss?raw", "default", "spacing", "../theme/_spacing.scss?raw"],
      ]);
    });

    it("records legacy Sass @import as wildcard module uses", () => {
      const source = `@import "./legacy";
@import "./tokens", "@design/foundation/scss/utils";
@import "https://cdn.example/reset.css";
@import "./plain.css";`;
      const document = parseStyleDocument(source, "/fake/a.module.scss");

      expect(
        document.sassModuleUses.map((moduleUse) => [
          moduleUse.source,
          moduleUse.namespaceKind,
          moduleUse.namespace,
          sliceRange(source, moduleUse.range),
        ]),
      ).toEqual([
        ["./legacy", "wildcard", null, "./legacy"],
        ["./tokens", "wildcard", null, "./tokens"],
        ["@design/foundation/scss/utils", "wildcard", null, "@design/foundation/scss/utils"],
      ]);
    });

    it("records @forward edges with source ranges", () => {
      const source = `@forward "./tokens" as theme-* show $gap, raised;
@forward "../theme/_spacing.scss?raw" hide $private, hidden;`;
      const document = parseStyleDocument(source, "/fake/a.module.scss");

      expect(
        document.sassModuleForwards.map((moduleForward) => [
          moduleForward.source,
          moduleForward.prefix,
          moduleForward.visibilityKind,
          moduleForward.visibilityMembers,
          sliceRange(source, moduleForward.range),
        ]),
      ).toEqual([
        [
          "./tokens",
          "theme-",
          "show",
          [
            { name: "gap", symbolKind: "variable" },
            { name: "raised", symbolKind: null },
          ],
          "./tokens",
        ],
        [
          "../theme/_spacing.scss?raw",
          "",
          "hide",
          [
            { name: "private", symbolKind: "variable" },
            { name: "hidden", symbolKind: null },
          ],
          "../theme/_spacing.scss?raw",
        ],
      ]);
    });
  });

  describe("Sass symbol occurrences", () => {
    it("records selector-scoped same-file Sass symbol references", () => {
      const source = `$gap: 1rem;
@mixin raised($depth) { box-shadow: 0 0 $depth black; }
@function tone($value) { @return $value; }
.button {
  color: $gap;
  @include raised($gap);
  border-color: tone($gap);
}
.ghost {
  color: $missing;
  @include absent($gap);
}`;
      const document = parseStyleDocument(source, "/fake/a.module.scss");

      expect(
        document.sassSymbols.map((symbol) => [
          symbol.selectorName,
          symbol.symbolKind,
          symbol.name,
          symbol.role,
          symbol.resolution,
          sliceRange(source, symbol.range),
        ]),
      ).toEqual([
        ["button", "variable", "gap", "reference", "resolved", "$gap"],
        ["button", "mixin", "raised", "include", "resolved", "raised"],
        ["button", "variable", "gap", "reference", "resolved", "$gap"],
        ["button", "function", "tone", "call", "resolved", "tone"],
        ["button", "variable", "gap", "reference", "resolved", "$gap"],
        ["ghost", "variable", "missing", "reference", "unresolved", "$missing"],
        ["ghost", "mixin", "absent", "include", "unresolved", "absent"],
        ["ghost", "variable", "gap", "reference", "resolved", "$gap"],
      ]);
    });

    it("records @include mixin references consistently across argument shapes", () => {
      const source = `.icon {
  &::before {
    @include position(absolute, all);
  }
  .dot {
    @include position(absolute, (top -1px, left -1px));
  }
}
.image {
  &::after {
    @include position(absolute, all);
  }
}
.link-detail {
  @include position(absolute, all);
}
`;
      const document = parseStyleDocument(source, "/fake/a.module.scss");

      expect(
        document.sassSymbols
          .filter((symbol) => symbol.symbolKind === "mixin" && symbol.name === "position")
          .map((symbol) => [symbol.selectorName, symbol.role, sliceRange(source, symbol.range)]),
      ).toEqual([
        ["icon", "include", "position"],
        ["dot", "include", "position"],
        ["image", "include", "position"],
        ["link-detail", "include", "position"],
      ]);
    });

    it("records same-file Sass symbol declarations", () => {
      const source = `$gap: 1rem;
@mixin raised($depth) { box-shadow: 0 0 $depth black; }
@function tone($value) { @return $value; }
.button {
  color: $gap;
  @include raised($gap);
  border-color: tone($gap);
}`;
      const document = parseStyleDocument(source, "/fake/a.module.scss");

      expect(
        document.sassSymbolDecls.map((decl) => [
          decl.symbolKind,
          decl.name,
          sliceRange(source, decl.range),
        ]),
      ).toEqual([
        ["variable", "gap", "$gap"],
        ["mixin", "raised", "raised"],
        ["variable", "depth", "$depth"],
        ["function", "tone", "tone"],
        ["variable", "value", "$value"],
      ]);
    });

    it("keeps Sass symbol ranges in UTF-16 editor positions", () => {
      const source = `$gap: 1rem;
.button { content: "한🙂"; color: $gap; }`;
      const document = parseStyleDocument(source, "/fake/a.module.scss");
      const symbol = document.sassSymbols[0]!;

      expect(sliceRange(source, symbol.range)).toBe("$gap");
      expect(symbol.range.start.character).toBe(source.split("\n")[1]!.indexOf("$gap"));
    });

    it("does not resolve Sass variables from another local scope", () => {
      const source = `.one {
  $gap: 1rem;
}
.two {
  color: $gap;
}`;
      const document = parseStyleDocument(source, "/fake/a.module.scss");

      expect(
        document.sassSymbols.map((symbol) => [
          symbol.selectorName,
          symbol.name,
          symbol.resolution,
          sliceRange(source, symbol.range),
        ]),
      ).toEqual([["two", "gap", "unresolved", "$gap"]]);
    });

    it("does not treat module-qualified Sass references as same-file symbols", () => {
      const source = `@use "./tokens" as tokens;
@mixin raised { box-shadow: none; }
@function tone($value) { @return $value; }
.button {
  color: tokens.$gap;
  @include tokens.raised;
  border-color: tokens.tone(tokens.$gap);
}`;
      const document = parseStyleDocument(source, "/fake/a.module.scss");

      expect(document.sassSymbols).toEqual([]);
    });

    it("records scoped Less variable declarations and references", () => {
      const source = `@gap: 1rem;
.card {
  @gap: 2rem;
  color: @gap;
}
.other {
  color: @gap;
  margin: @missing;
}`;
      const document = parseStyleDocument(source, "/fake/a.module.less");

      expect(
        document.sassSymbolDecls.map((decl) => [
          decl.syntax,
          decl.symbolKind,
          decl.name,
          sliceRange(source, decl.range),
        ]),
      ).toEqual([
        ["less", "variable", "gap", "@gap"],
        ["less", "variable", "gap", "@gap"],
      ]);
      expect(
        document.sassSymbols.map((symbol) => [
          symbol.syntax,
          symbol.selectorName,
          symbol.name,
          symbol.resolution,
          sliceRange(source, symbol.range),
        ]),
      ).toEqual([
        ["less", "card", "gap", "resolved", "@gap"],
        ["less", "other", "gap", "resolved", "@gap"],
        ["less", "other", "missing", "unresolved", "@missing"],
      ]);
    });

    it("records unresolved function calls when wildcard Sass modules may provide them", () => {
      const source = `@use "./tokens" as *;
.button {
  border-color: tone($gap);
}`;
      const document = parseStyleDocument(source, "/fake/a.module.scss");

      expect(
        document.sassSymbols.map((symbol) => [
          symbol.selectorName,
          symbol.symbolKind,
          symbol.name,
          symbol.role,
          symbol.resolution,
          sliceRange(source, symbol.range),
        ]),
      ).toEqual([
        ["button", "function", "tone", "call", "unresolved", "tone"],
        ["button", "variable", "gap", "reference", "unresolved", "$gap"],
      ]);
    });

    it("records module-qualified Sass member references separately", () => {
      const source = `@use "./tokens" as tokens;
@mixin raised { box-shadow: none; }
@function tone($value) { @return $value; }
.button {
  color: tokens.$gap;
  @include tokens.raised;
  border-color: tokens.tone(tokens.$gap);
}`;
      const document = parseStyleDocument(source, "/fake/a.module.scss");

      expect(
        document.sassModuleMemberRefs.map((memberRef) => [
          memberRef.selectorName,
          memberRef.namespace,
          memberRef.symbolKind,
          memberRef.name,
          memberRef.role,
          sliceRange(source, memberRef.range),
        ]),
      ).toEqual([
        ["button", "tokens", "variable", "gap", "reference", "$gap"],
        ["button", "tokens", "mixin", "raised", "include", "raised"],
        ["button", "tokens", "function", "tone", "call", "tone"],
        ["button", "tokens", "variable", "gap", "reference", "$gap"],
      ]);
    });
  });

  describe("@media / @at-root unwrapping", () => {
    it("indexes classes inside @media", () => {
      const map = parseStyleSelectorMap(
        `@media (min-width: 600px) { .wide { padding: 16px; } }`,
        "/fake/a.module.scss",
      );
      expect(map.has("wide")).toBe(true);
    });

    it("indexes classes inside @at-root", () => {
      const map = parseStyleSelectorMap(
        `.parent { @at-root .escaped { color: red; } }`,
        "/fake/a.module.scss",
      );
      expect(map.has("escaped")).toBe(true);
    });
  });
});

function sliceRange(
  source: string,
  range: {
    readonly start: { readonly line: number; readonly character: number };
    readonly end: { readonly line: number; readonly character: number };
  },
): string {
  const lines = source.split("\n");
  if (range.start.line === range.end.line) {
    return lines[range.start.line]!.slice(range.start.character, range.end.character);
  }
  return [
    lines[range.start.line]!.slice(range.start.character),
    ...lines.slice(range.start.line + 1, range.end.line),
    lines[range.end.line]!.slice(0, range.end.character),
  ].join("\n");
}

describe("StyleIndexCache", () => {
  it("keeps grouped and nested selector definition ranges on the source token", () => {
    const document = parseStyleDocument(
      `.a, .b { &__icon { &--small { color: red; } } }`,
      "/fake/a.module.scss",
    );
    const selectorRanges = new Map(
      document.selectors.map((selector) => [selector.name, selector.range]),
    );

    expect(selectorRanges.get("a")).toEqual({
      start: { line: 0, character: 1 },
      end: { line: 0, character: 2 },
    });
    expect(selectorRanges.get("b")).toEqual({
      start: { line: 0, character: 5 },
      end: { line: 0, character: 6 },
    });
    expect(selectorRanges.get("a__icon")).toEqual({
      start: { line: 0, character: 10 },
      end: { line: 0, character: 16 },
    });
    expect(selectorRanges.get("a__icon--small")).toEqual({
      start: { line: 0, character: 20 },
      end: { line: 0, character: 27 },
    });
  });

  it("uses an injected style document builder as the parser choke point", () => {
    let buildCount = 0;
    const cache = new StyleIndexCache({
      max: 10,
      buildStyleDocument: (filePath, content) => {
        buildCount += 1;
        return parseStyleDocument(content, filePath);
      },
    });

    const first = cache.getStyleDocument("/fake/a.module.scss", `.btn { color: red; }`);
    const second = cache.getStyleDocument("/fake/a.module.scss", `.btn { color: red; }`);
    const third = cache.getStyleDocument("/fake/a.module.scss", `.btn { color: blue; }`);

    expect(second).toBe(first);
    expect(third).not.toBe(first);
    expect(buildCount).toBe(2);
  });

  it("returns the same style document for identical content", () => {
    const cache = new StyleIndexCache({ max: 10 });
    const first = cache.getStyleDocument("/fake/a.module.scss", `.btn { color: red; }`);
    const second = cache.getStyleDocument("/fake/a.module.scss", `.btn { color: red; }`);
    expect(second).toBe(first);
  });

  it("re-parses when content changes", () => {
    const cache = new StyleIndexCache({ max: 10 });
    const first = cache.getStyleDocument("/fake/a.module.scss", `.btn { color: red; }`);
    const second = cache.getStyleDocument("/fake/a.module.scss", `.btn { color: blue; }`);
    expect(second).not.toBe(first);
    expect(selectorMapFromDocument(second).get("btn")!.declarations).toContain("color: blue");
  });

  it("invalidate(path) drops the cached entry", () => {
    const cache = new StyleIndexCache({ max: 10 });
    const first = cache.getStyleDocument("/fake/a.module.scss", `.btn { color: red; }`);
    cache.invalidate("/fake/a.module.scss");
    const second = cache.getStyleDocument("/fake/a.module.scss", `.btn { color: red; }`);
    expect(second).not.toBe(first);
  });

  it("clear() drops everything", () => {
    const cache = new StyleIndexCache({ max: 10 });
    cache.getStyleDocument("/fake/a.module.scss", `.a { color: red; }`);
    cache.getStyleDocument("/fake/b.module.scss", `.b { color: red; }`);
    cache.clear();
    const after = cache.getStyleDocument("/fake/a.module.scss", `.a { color: red; }`);
    expect(selectorMapFromDocument(after).has("a")).toBe(true);
  });

  it("evicts the least-recently-used entry beyond the max", () => {
    const cache = new StyleIndexCache({ max: 2 });
    const a = cache.getStyleDocument("/a.module.scss", `.a{}`);
    cache.getStyleDocument("/b.module.scss", `.b{}`);
    // LRU order: a, b
    cache.getStyleDocument("/a.module.scss", `.a{}`); // touch a → order: b, a
    cache.getStyleDocument("/c.module.scss", `.c{}`); // evicts b
    const aAgain = cache.getStyleDocument("/a.module.scss", `.a{}`);
    expect(aAgain).toBe(a);
    // b was evicted, so re-getting it reparses (different content string
    // still produces an equivalent document, but identity differs).
    const bAgain = cache.getStyleDocument("/b.module.scss", `.b{}`);
    expect(selectorMapFromDocument(bAgain).has("b")).toBe(true);
  });

  it("stores a transformed style document entry", () => {
    const cache = new StyleIndexCache({ max: 10 });
    const entry = cache.getEntry(
      "/fake/a.module.scss",
      `.button { color: red; &--primary { color: blue; } }`,
    );

    expect(entry.styleDocument.filePath).toBe("/fake/a.module.scss");
    expect(entry.styleDocument.selectors.map((selector) => selector.name)).toEqual([
      "button",
      "button--primary",
    ]);
  });

  it("returns the same cached style document for identical content", () => {
    const cache = new StyleIndexCache({ max: 10 });
    const first = cache.getStyleDocument("/fake/a.module.scss", `.btn { color: red; }`);
    const second = cache.getStyleDocument("/fake/a.module.scss", `.btn { color: red; }`);
    expect(second).toBe(first);
  });

  // ── classnameTransform integration ──

  it("different modes coexist for the same file path", () => {
    const cache = new StyleIndexCache({ max: 10 });
    const asIsDoc = cache.getStyleDocument("/f.module.scss", `.btn-primary { color: red; }`);
    const asIsMap = selectorMapFromDocument(asIsDoc);
    expect(asIsMap.has("btnPrimary")).toBe(false);
    const expandedDoc = cache.getStyleDocument(
      "/f.module.scss",
      `.btn-primary { color: red; }`,
      "camelCase",
    );
    const expanded = selectorMapFromDocument(expandedDoc);
    expect(expandedDoc).not.toBe(asIsDoc);
    expect(expanded.has("btn-primary")).toBe(true);
    expect(expanded.has("btnPrimary")).toBe(true);
    expect(expanded.get("btnPrimary")?.originalName).toBe("btn-primary");
  });

  it("same mode still returns the same cached style document", () => {
    const cache = new StyleIndexCache({ max: 10 });
    const first = cache.getStyleDocument("/f.module.scss", `.btn-primary { color: red; }`, "asIs");
    const second = cache.getStyleDocument("/f.module.scss", `.btn-primary { color: red; }`, "asIs");
    expect(second).toBe(first);
  });

  it("invalidate(path) drops entries for every mode", () => {
    const cache = new StyleIndexCache({ max: 10 });
    const asIsDoc = cache.getStyleDocument("/f.module.scss", `.btn-primary { color: red; }`);
    const camelDoc = cache.getStyleDocument(
      "/f.module.scss",
      `.btn-primary { color: red; }`,
      "camelCase",
    );
    cache.invalidate("/f.module.scss");
    const asIsAgain = cache.getStyleDocument("/f.module.scss", `.btn-primary { color: red; }`);
    const camelAgain = cache.getStyleDocument(
      "/f.module.scss",
      `.btn-primary { color: red; }`,
      "camelCase",
    );
    expect(asIsAgain).not.toBe(asIsDoc);
    expect(camelAgain).not.toBe(camelDoc);
  });

  it("semantic fingerprint ignores declaration-only edits", () => {
    const cache = new StyleIndexCache({ max: 10 });
    const first = cache.getStyleDocument("/f.module.scss", `.btn { color: red; }`);
    const second = cache.getStyleDocument("/f.module.scss", `.btn { color: blue; }`);
    expect(styleDocumentSemanticFingerprint(second)).toBe(styleDocumentSemanticFingerprint(first));
  });
});

describe("buildChildContext", () => {
  it("bare single class: sets className, no isGrouped", () => {
    const ctx = buildChildContext([".a"], ".a");
    expect(ctx.selector).toBe(".a");
    expect(ctx.className).toBe("a");
    expect(ctx.isGrouped).toBeUndefined();
  });

  it("pseudo-bearing parent: no className, no isGrouped", () => {
    const ctx = buildChildContext([".a:hover"], ".a:hover");
    expect(ctx.selector).toBe(".a:hover");
    expect(ctx.className).toBeUndefined();
    expect(ctx.isGrouped).toBeUndefined();
  });

  it("grouped parent: isGrouped true, className undefined on each branch", () => {
    const first = buildChildContext([".a", ".b"], ".a");
    expect(first.selector).toBe(".a");
    expect(first.className).toBeUndefined();
    expect(first.isGrouped).toBe(true);

    const second = buildChildContext([".a", ".b"], ".b");
    expect(second.selector).toBe(".b");
    expect(second.className).toBeUndefined();
    expect(second.isGrouped).toBe(true);
  });

  it("descendant compound parent: no className", () => {
    const ctx = buildChildContext([".a .b"], ".a .b");
    expect(ctx.selector).toBe(".a .b");
    expect(ctx.className).toBeUndefined();
    expect(ctx.isGrouped).toBeUndefined();
  });
});

describe("enumerateGroups", () => {
  it("splits a simple comma-separated selector", () => {
    expect(enumerateGroups(".a, .b")).toEqual([
      { raw: ".a", offset: 0 },
      { raw: ".b", offset: 4 },
    ]);
  });

  it("respects paren depth inside :is(...)", () => {
    const result = enumerateGroups(":is(.a, .b) .c");
    expect(result).toHaveLength(1);
    expect(result[0]!.raw).toBe(":is(.a, .b) .c");
    expect(result[0]!.offset).toBe(0);
  });

  it("passes comment text through verbatim", () => {
    const result = enumerateGroups(".a /* x */ b");
    expect(result).toHaveLength(1);
    expect(result[0]!.raw).toBe(".a /* x */ b");
  });
});

describe("findBemSuffixSpan", () => {
  // Helper: build a minimal postcss-Rule-like mock whose
  // `source.start.offset === 0` and `input.fromOffset` returns a
  // fake 1-based { line, col } computed from the given content.
  // Sufficient for exercising the offset math in isolation.
  function mockRule(content: string): Rule {
    return {
      source: {
        start: { offset: 0, line: 1, column: 1 },
        input: {
          fromOffset(offset: number): { line: number; col: number } | null {
            if (offset < 0 || offset > content.length) return null;
            let line = 1;
            let col = 1;
            for (let i = 0; i < offset; i++) {
              if (content[i] === "\n") {
                line++;
                col = 1;
              } else {
                col++;
              }
            }
            return { line, col };
          },
        },
      },
    } as unknown as Rule;
  }

  it("accepts `&--primary`", () => {
    const result = findBemSuffixSpan(mockRule("&--primary"), 0, "&--primary");
    expect(result).not.toBeNull();
    expect(result!.rawToken).toBe("&--primary");
    expect(result!.range.start).toEqual({ line: 0, character: 0 });
    expect(result!.range.end).toEqual({ line: 0, character: 10 });
  });

  it("accepts `&__icon`", () => {
    const result = findBemSuffixSpan(mockRule("&__icon"), 0, "&__icon");
    expect(result).not.toBeNull();
    expect(result!.rawToken).toBe("&__icon");
    expect(result!.range.end.character - result!.range.start.character).toBe(7);
  });

  it("rejects compound `&.active`", () => {
    expect(findBemSuffixSpan(mockRule("&.active"), 0, "&.active")).toBeNull();
  });

  it("rejects pseudo `&:hover`", () => {
    expect(findBemSuffixSpan(mockRule("&:hover"), 0, "&:hover")).toBeNull();
  });

  it("rejects combinator-before `.a &--x`", () => {
    expect(findBemSuffixSpan(mockRule(".a &--x"), 0, ".a &--x")).toBeNull();
  });

  it("rejects descendant-after `&--x .y`", () => {
    expect(findBemSuffixSpan(mockRule("&--x .y"), 0, "&--x .y")).toBeNull();
  });

  it("accepts hyphenated `&--primary-inverse`", () => {
    const result = findBemSuffixSpan(mockRule("&--primary-inverse"), 0, "&--primary-inverse");
    expect(result).not.toBeNull();
    expect(result!.rawToken).toBe("&--primary-inverse");
    expect(result!.range.end.character - result!.range.start.character).toBe(18);
  });

  it("returns null when `rule.source.start` is undefined", () => {
    const rule = { source: {} } as unknown as Rule;
    expect(findBemSuffixSpan(rule, 0, "&--primary")).toBeNull();
  });

  it("accepts trailing-whitespace-only `&--primary  ` (off-by-one regression catcher)", () => {
    // The step 6 slice must be `slice(ampIndex + fragment.length).trim()`,
    // NOT `... - 1`. An off-by-one implementation would see `"y  "` and
    // reject — this test locks the correct semantics.
    const result = findBemSuffixSpan(mockRule("&--primary  "), 0, "&--primary  ");
    expect(result).not.toBeNull();
    expect(result!.rawToken).toBe("&--primary");
  });

  it("rejects descendant-after sibling `&--primary .y`", () => {
    expect(findBemSuffixSpan(mockRule("&--primary .y"), 0, "&--primary .y")).toBeNull();
  });

  it("rejects immediate-compound prefix `.a&--x` / `#id&--x` / `[data-x]&--x` / `tag&--x`", () => {
    // Without the step 3 "nothing before the &" check, the
    // resolved selector would be `.a.parent--x` which
    // extractClassNames turns into TWO classes — violating the
    // single-class invariant the BEM suffix rename depends on.
    expect(findBemSuffixSpan(mockRule(".a&--x"), 0, ".a&--x")).toBeNull();
    expect(findBemSuffixSpan(mockRule("#id&--x"), 0, "#id&--x")).toBeNull();
    expect(findBemSuffixSpan(mockRule("[data-x]&--x"), 0, "[data-x]&--x")).toBeNull();
    expect(findBemSuffixSpan(mockRule("tag&--x"), 0, "tag&--x")).toBeNull();
  });
});

describe("BEM suffix info", () => {
  // Positive: bemSuffix populated
  it("populates bemSuffix for `.button { &--primary {} }`", () => {
    const map = parseStyleSelectorMap(`.button { &--primary {} }`, "/f.module.scss");
    const info = map.get("button--primary");
    expect(info).toBeDefined();
    expect(info!.nestedSafety).toBe("bemSuffixSafe");
    expect(info!.bemSuffix).toBeDefined();
    expect(info!.bemSuffix!.rawToken).toBe("&--primary");
    expect(info!.bemSuffix!.parentResolvedName).toBe("button");
    const r = info!.bemSuffix!.rawTokenRange;
    expect(r.end.character - r.start.character).toBe(10);
  });

  it("populates bemSuffix for `.button { &__icon {} }`", () => {
    const map = parseStyleSelectorMap(`.button { &__icon {} }`, "/f.module.scss");
    const info = map.get("button__icon");
    expect(info).toBeDefined();
    expect(info!.bemSuffix!.rawToken).toBe("&__icon");
    expect(info!.bemSuffix!.parentResolvedName).toBe("button");
    const r = info!.bemSuffix!.rawTokenRange;
    expect(r.end.character - r.start.character).toBe(7);
  });

  it("populates bemSuffix on multi-line source — rawTokenRange lands on the `&` line", () => {
    const src = `.button {\n  &--primary {}\n}`;
    const map = parseStyleSelectorMap(src, "/f.module.scss");
    const info = map.get("button--primary");
    expect(info).toBeDefined();
    const r = info!.bemSuffix!.rawTokenRange;
    expect(r.start.line).toBe(1);
    expect(r.start.character).toBe(2);
    expect(r.end.character - r.start.character).toBe(10);
  });

  it("handles CRLF source correctly", () => {
    // CRLF column-shift regression catcher: a handler that counted
    // \r into line length would drift by 1 character.
    const src = `.button {\r\n  &--primary {}\r\n}`;
    const map = parseStyleSelectorMap(src, "/f.module.scss");
    const info = map.get("button--primary");
    expect(info).toBeDefined();
    const r = info!.bemSuffix!.rawTokenRange;
    expect(r.start.line).toBe(1);
    expect(r.start.character).toBe(2);
    expect(r.end.character - r.start.character).toBe(10);
  });

  it("populates bemSuffix for deep-nested `.card { &__icon { &--small {} } }`", () => {
    const map = parseStyleSelectorMap(`.card { &__icon { &--small {} } }`, "/f.module.scss");
    const inner = map.get("card__icon--small");
    expect(inner).toBeDefined();
    expect(inner!.bemSuffix!.rawToken).toBe("&--small");
    expect(inner!.bemSuffix!.parentResolvedName).toBe("card__icon");

    const middle = map.get("card__icon");
    expect(middle).toBeDefined();
    expect(middle!.bemSuffix!.rawToken).toBe("&__icon");
    expect(middle!.bemSuffix!.parentResolvedName).toBe("card");
  });

  it("populates bemSuffix via comment-bearing parent `.a /* x */ { &--b {} }`", () => {
    const src = `.a /* x */ { &--b {} }`;
    const map = parseStyleSelectorMap(src, "/f.module.scss");
    const info = map.get("a--b");
    expect(info).toBeDefined();
    expect(info!.bemSuffix!.rawToken).toBe("&--b");
    expect(info!.bemSuffix!.parentResolvedName).toBe("a");
  });

  // Negative: nested selector with no BEM-safe metadata
  it("leaves bemSuffix undefined for compound `.button { &.active {} }` (reject compound)", () => {
    const map = parseStyleSelectorMap(`.button { &.active {} }`, "/f.module.scss");
    const active = map.get("active");
    expect(active).toBeDefined();
    expect(active!.nestedSafety).toBe("nestedUnsafe");
    expect(active!.bemSuffix).toBeUndefined();
    // button parent entry must survive unchanged
    const button = map.get("button");
    expect(button).toBeDefined();
    expect(button!.nestedSafety).toBe("flat");
  });

  it("preserves flat parent for `.button { &:hover {} }` (dedup guard)", () => {
    const map = parseStyleSelectorMap(`.button { &:hover {} }`, "/f.module.scss");
    const button = map.get("button");
    expect(button).toBeDefined();
    expect(button!.nestedSafety).toBe("flat");
    expect(button!.bemSuffix).toBeUndefined();
  });

  it("handles non-bare parent `.card:hover { &--primary {} }` safely", () => {
    // extractClassNames strips `:hover--primary` greedily (pseudo
    // regex eats the hyphenated suffix), so the nested rule
    // resolves to class `card`. The dedup guard keeps the flat
    // `.card:hover` entry from being downgraded. Net result: no
    // `card--primary` entry exists, and the flat `card` entry is
    // unchanged — safe.
    const map = parseStyleSelectorMap(`.card:hover { &--primary {} }`, "/f.module.scss");
    expect(map.has("card--primary")).toBe(false);
    const card = map.get("card");
    expect(card).toBeDefined();
    expect(card!.nestedSafety).toBe("flat");
    expect(card!.bemSuffix).toBeUndefined();
  });

  it("leaves bemSuffix undefined for grouped parent `.a, .b { &--c {} }`", () => {
    const map = parseStyleSelectorMap(`.a, .b { &--c {} }`, "/f.module.scss");
    const info = map.get("a--c") ?? map.get("b--c");
    expect(info).toBeDefined();
    expect(info!.nestedSafety).toBe("nestedUnsafe");
    expect(info!.bemSuffix).toBeUndefined();
  });

  it("leaves bemSuffix undefined for grouped-nested child `.btn { &--a, &--b {} }`", () => {
    const map = parseStyleSelectorMap(`.btn { &--a, &--b {} }`, "/f.module.scss");
    const a = map.get("btn--a");
    const b = map.get("btn--b");
    expect(a).toBeDefined();
    expect(b).toBeDefined();
    expect(a!.bemSuffix).toBeUndefined();
    expect(b!.bemSuffix).toBeUndefined();
  });

  it("leaves bemSuffix undefined for multi-`&` `.btn { & + &--x {} }`", () => {
    const map = parseStyleSelectorMap(`.btn { & + &--x {} }`, "/f.module.scss");
    const info = map.get("btn--x");
    if (info !== undefined) {
      expect(info.bemSuffix).toBeUndefined();
    }
  });

  it("skips interpolated top-level rules `.#{$prefix}--primary {}`", () => {
    // extractClassNames doesn't match .#{...}, so no entry is
    // produced. No crash is the goal.
    const map = parseStyleSelectorMap(`.#{$prefix}--primary { color: red; }`, "/f.module.scss");
    expect(map.size).toBe(0);
  });

  it("does not crash on `@at-root &--escape {}`", () => {
    // Invalid SCSS shape — we just don't want a throw.
    const content = `.btn { @at-root &--escape {} }`;
    expect(() => parseStyleSelectorMap(content, "/f.module.scss")).not.toThrow();
  });
});
