import {
  makeStyleDocumentHIR,
  type SelectorDeclHIR,
  type StyleDocumentHIR,
} from "../hir/style-types";

/**
 * 5-mode classname transformation for CSS Modules. Mirrors
 * css-loader's `localsConvention` option and common CSS Modules
 * classname transform behavior.
 *
 * Takes an original SCSS class name and returns the list of names
 * the JS-side class-export family should expose for it:
 *
 * | Mode             | Original | Output for `.btn-primary` |
 * |------------------|:--------:|---------------------------|
 * | `asIs`           |    ✓     | `[btn-primary]`           |
 * | `camelCase`      |    ✓     | `[btn-primary, btnPrimary]` |
 * | `camelCaseOnly`  |    ✗     | `[btnPrimary]`            |
 * | `dashes`         |    ✓     | `[btn-primary, btnPrimary]` |
 * | `dashesOnly`     |    ✗     | `[btnPrimary]`            |
 *
 * Dedup rule: if the transformed name equals the original, only
 * the original is emitted.
 *
 * `camelCase*` consumes `-`, `_`, whitespace as separators (ASCII
 * subset of lodash-camelcase). `dashes*` consumes only `-`, leaving
 * underscores and whitespace untouched. Unicode class names are
 * out of scope; `.classNameB` or `.btn-primary` style names
 * produce lodash-equivalent output for all real-world inputs.
 */
export type ClassnameTransformMode =
  | "asIs"
  | "camelCase"
  | "camelCaseOnly"
  | "dashes"
  | "dashesOnly";

export function transformClassname(mode: ClassnameTransformMode, name: string): string[] {
  switch (mode) {
    case "asIs":
      return [name];
    case "camelCase": {
      const camel = toCamelCase(name);
      return camel === name ? [name] : [name, camel];
    }
    case "camelCaseOnly":
      return [toCamelCase(name)];
    case "dashes": {
      const dashed = dashesToCamel(name);
      return dashed === name ? [name] : [name, dashed];
    }
    case "dashesOnly":
      return [dashesToCamel(name)];
  }
}

/**
 * `-+(\w)` → uppercase. Only dashes are consumed; underscores and
 * whitespace pass through.
 */
function dashesToCamel(name: string): string {
  return name.replace(/-+(\w)/g, (_, ch: string) => ch.toUpperCase());
}

/**
 * ASCII camelCase: treats `-`, `_`, whitespace as separators.
 * Leading separators are stripped, consecutive separators collapse
 * to a single boundary, the first segment stays lowercase, the rest
 * are capitalized. Output matches lodash.camelcase for ASCII input.
 */
function toCamelCase(name: string): string {
  const parts = name.split(/[\s_-]+/).filter(Boolean);
  if (parts.length === 0) return name;
  return parts
    .map((p, i) =>
      i === 0 ? p.charAt(0).toLowerCase() + p.slice(1) : p.charAt(0).toUpperCase() + p.slice(1),
    )
    .join("");
}

export function expandStyleDocumentWithTransform(
  base: StyleDocumentHIR,
  mode: ClassnameTransformMode,
): StyleDocumentHIR {
  if (mode === "asIs") return base;

  const expandedSelectors: SelectorDeclHIR[] = [];
  for (const selector of base.selectors) {
    const aliases = transformClassname(mode, selector.name);
    for (const alias of aliases) {
      if (alias === selector.name) {
        expandedSelectors.push(selector);
        continue;
      }
      expandedSelectors.push({
        ...selector,
        id: `${selector.id}:alias:${alias}`,
        name: alias,
        viewKind: "alias",
        originalName: selector.name,
      });
    }
  }

  return makeStyleDocumentHIR(
    base.filePath,
    expandedSelectors,
    base.keyframes,
    base.animationNameRefs,
    base.valueDecls,
    base.valueImports,
    base.valueRefs,
    base.customPropertyDecls,
    base.customPropertyRefs,
    base.sassSymbols,
    base.sassSymbolDecls,
    base.sassModuleUses,
    base.sassModuleMemberRefs,
    base.sassModuleForwards,
  );
}
