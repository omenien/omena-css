import { describe, expect, it } from "vitest";
import ts from "typescript";
import { AliasResolver } from "../../../server/engine-core-ts/src/core/cx/alias-resolver";
import {
  composeBinderPluginsV0,
  cssModulesClassnamesBinderPluginV0,
} from "../../../server/engine-core-ts/src/core/binder/binder-plugin";
import { cvaRecipeBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/cva-recipe-plugin";
import { tailwindUnoUtilityBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/tailwind-utility-plugin";
import { vanillaExtractRecipeBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/vanilla-extract-recipe-plugin";
import { vueStyleModuleBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/vue-style-module-plugin";
import { buildSourceBinder } from "../../../server/engine-core-ts/src/core/binder/binder-builder";
import { projectVueSfcScriptToTypeScriptSource } from "../../../server/engine-core-ts/src/core/ts/vue-sfc-source";

const EMPTY_ALIAS_RESOLVER = new AliasResolver("/fake", {});

function parse(source: string, filePath = "/fake/src/Card.tsx"): ts.SourceFile {
  return ts.createSourceFile(filePath, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
}

describe("cssModulesClassnamesBinderPluginV0", () => {
  it("declares the built-in binder plugin boundary without exposing an external ABI", () => {
    expect(cssModulesClassnamesBinderPluginV0).toMatchObject({
      id: "css-modules-classnames-bind",
      version: "0",
      stability: "builtIn",
      domains: ["css-modules"],
    });
    expect(cssModulesClassnamesBinderPluginV0.importTargets).toEqual([
      "*.module.css",
      "*.module.scss",
      "*.module.less",
    ]);
    expect(cssModulesClassnamesBinderPluginV0.utilityTargets).toEqual([
      "classnames/bind",
      "classnames",
      "clsx",
      "clsx/lite",
    ]);
  });

  it("ports current CSS Modules, cx, style access, and class util facts behind one plugin", () => {
    const sourceFile = parse(`
      import classNames from 'classnames/bind';
      import clsx from 'clsx';
      import styles from './Card.module.scss';
      const cx = classNames.bind(styles);
      const tone = 'primary';
      const el = <div className={clsx(cx('card', \`tone-\${tone}\`), styles.icon)} />;
    `);
    const sourceBinder = buildSourceBinder(sourceFile);

    const result = cssModulesClassnamesBinderPluginV0.analyzeSource({
      sourceFile,
      filePath: "/fake/src/Card.tsx",
      sourceBinder,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
    });

    expect(result.pluginId).toBe("css-modules-classnames-bind");
    expect(result.stylesBindings.get("styles")).toEqual({
      kind: "resolved",
      absolutePath: "/fake/src/Card.module.scss",
    });
    expect(result.cxBindings).toMatchObject([
      {
        cxVarName: "cx",
        stylesVarName: "styles",
        scssModulePath: "/fake/src/Card.module.scss",
      },
    ]);
    expect(result.classUtilNames).toEqual(["clsx"]);
    expect(result.classExpressions.map((entry) => entry.kind)).toEqual([
      "literal",
      "template",
      "styleAccess",
    ]);
    expect(result.domainClassReferences).toEqual([]);
    expect(result.classValueUniverses).toEqual([]);
  });

  it("tracks Tailwind/Uno utility classes without pretending to own a CSS Module source", () => {
    const sourceFile = parse(`
      import clsx from 'clsx';
      export function Card({ active }: { active: boolean }) {
        return (
          <section className={clsx("flex gap-2", active && "bg-blue-500", \`tone-\${active}\`)}>
            <div className="text-sm hover:underline" />
          </section>
        );
      }
    `);
    const sourceBinder = buildSourceBinder(sourceFile);

    const result = tailwindUnoUtilityBinderPluginV0.analyzeSource({
      sourceFile,
      filePath: "/fake/src/Card.tsx",
      sourceBinder,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
    });

    expect(result.stylesBindings.size).toBe(0);
    expect(result.classExpressions).toEqual([]);
    expect(result.domainClassReferences).toMatchObject([
      { matchKind: "literal", className: "flex", domain: "utility-css" },
      { matchKind: "literal", className: "gap-2", domain: "utility-css" },
      { matchKind: "literal", className: "bg-blue-500", domain: "utility-css" },
      { matchKind: "templatePrefix", staticPrefix: "tone-" },
      { matchKind: "literal", className: "text-sm", origin: "jsxClassAttribute" },
      { matchKind: "literal", className: "hover:underline", origin: "jsxClassAttribute" },
    ]);
  });

  it("composes CSS Modules and utility-domain plugins into one BinderPluginV0 runtime slot", () => {
    const sourceFile = parse(`
      import classNames from 'classnames/bind';
      import clsx from 'clsx';
      import styles from './Card.module.scss';
      const cx = classNames.bind(styles);
      const el = <div className={clsx(cx('card'), "flex")} />;
    `);
    const sourceBinder = buildSourceBinder(sourceFile);
    const plugin = composeBinderPluginsV0([
      cssModulesClassnamesBinderPluginV0,
      tailwindUnoUtilityBinderPluginV0,
    ]);

    const result = plugin.analyzeSource({
      sourceFile,
      filePath: "/fake/src/Card.tsx",
      sourceBinder,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
    });

    expect(result.stylesBindings.get("styles")).toEqual({
      kind: "resolved",
      absolutePath: "/fake/src/Card.module.scss",
    });
    expect(result.classExpressions).toMatchObject([{ kind: "literal", className: "card" }]);
    expect(result.domainClassReferences).toMatchObject([
      { matchKind: "literal", className: "flex", domain: "utility-css" },
    ]);
  });

  it("tracks vanilla-extract recipe variant calls without a CSS Module source", () => {
    const sourceFile = parse(`
      import { recipe as defineRecipe } from '@vanilla-extract/recipes';

      const button = defineRecipe({
        base: "button_base",
        variants: {
          tone: {
            primary: "button_tone_primary",
            danger: "button_tone_danger",
          },
          size: {
            sm: "button_size_sm",
            lg: "button_size_lg",
          },
        },
        compoundVariants: [
          {
            variants: { tone: "primary", size: "sm" },
            style: "button_primary_sm",
          },
        ],
        defaultVariants: {
          tone: "primary",
        },
      });

      const el = button({
        tone: active ? "primary" : "danger",
        size: "sm",
      });
    `);
    const sourceBinder = buildSourceBinder(sourceFile);

    const result = vanillaExtractRecipeBinderPluginV0.analyzeSource({
      sourceFile,
      filePath: "/fake/src/Button.css.ts",
      sourceBinder,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
    });

    expect(result.stylesBindings.size).toBe(0);
    expect(result.classExpressions).toEqual([]);
    expect(result.domainClassReferences).toMatchObject([
      {
        matchKind: "literal",
        className: "button.tone.primary",
        domain: "vanilla-extract-recipe",
      },
      {
        matchKind: "literal",
        className: "button.tone.danger",
        domain: "vanilla-extract-recipe",
      },
      {
        matchKind: "literal",
        className: "button.size.sm",
        domain: "vanilla-extract-recipe",
      },
    ]);
    expect(result.classValueUniverses).toMatchObject([
      {
        pluginId: "vanilla-extract-recipe-domain",
        domain: "vanilla-extract-recipe",
        ownerName: "button",
        universe: {
          kind: "reduced-product",
          baseClassNames: ["button_base"],
          axes: [
            {
              axisName: "tone",
              defaultValue: "primary",
              role: "variant",
              values: [
                { name: "danger", classNames: ["button_tone_danger"] },
                { name: "primary", classNames: ["button_tone_primary"] },
              ],
            },
            {
              axisName: "size",
              role: "variant",
              values: [
                { name: "lg", classNames: ["button_size_lg"] },
                { name: "sm", classNames: ["button_size_sm"] },
              ],
            },
            {
              axisName: "slots",
              role: "slot",
              reserved: true,
              values: [],
            },
          ],
          compoundVariants: [
            {
              conditions: [
                { axisName: "size", value: "sm" },
                { axisName: "tone", value: "primary" },
              ],
              classNames: ["button_primary_sm"],
            },
          ],
        },
      },
    ]);
  });

  it("does not invent a vanilla-extract base class when base is omitted", () => {
    const sourceFile = parse(`
      import { recipe } from '@vanilla-extract/recipes';

      const badge = recipe({
        variants: {
          tone: {
            info: "badge_info",
          },
        },
      });
    `);
    const sourceBinder = buildSourceBinder(sourceFile);

    const result = vanillaExtractRecipeBinderPluginV0.analyzeSource({
      sourceFile,
      filePath: "/fake/src/Badge.css.ts",
      sourceBinder,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
    });

    expect(result.classValueUniverses).toMatchObject([
      {
        pluginId: "vanilla-extract-recipe-domain",
        domain: "vanilla-extract-recipe",
        ownerName: "badge",
        universe: {
          kind: "reduced-product",
          baseClassNames: [],
          axes: [
            {
              axisName: "tone",
              role: "variant",
              values: [{ name: "info", classNames: ["badge_info"] }],
            },
            {
              axisName: "slots",
              role: "slot",
              reserved: true,
              values: [],
            },
          ],
        },
      },
    ]);
  });

  it("tracks cva phase-1 recipes through the class value universe provider", () => {
    const sourceFile = parse(`
      import { cva as defineCva } from 'class-variance-authority';

      const button = defineCva("button_base", {
        variants: {
          tone: {
            primary: "button_tone_primary",
            danger: "button_tone_danger",
          },
          size: {
            sm: "button_size_sm",
            lg: "button_size_lg",
          },
        },
        compoundVariants: [
          { tone: "primary", size: "sm", class: "button_primary_sm" },
        ],
        defaultVariants: {
          size: "sm",
        },
      });

      const el = button({
        tone: "primary",
        size: active ? "sm" : "lg",
      });
    `);
    const sourceBinder = buildSourceBinder(sourceFile);

    const result = cvaRecipeBinderPluginV0.analyzeSource({
      sourceFile,
      filePath: "/fake/src/Button.tsx",
      sourceBinder,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
    });

    expect(result.domainClassReferences).toMatchObject([
      {
        matchKind: "literal",
        className: "button.tone.primary",
        domain: "cva-recipe",
      },
      {
        matchKind: "literal",
        className: "button.size.sm",
        domain: "cva-recipe",
      },
      {
        matchKind: "literal",
        className: "button.size.lg",
        domain: "cva-recipe",
      },
    ]);
    expect(result.classValueUniverses).toMatchObject([
      {
        pluginId: "cva-recipe-domain",
        domain: "cva-recipe",
        ownerName: "button",
        universe: {
          kind: "reduced-product",
          baseClassNames: ["button_base"],
          axes: [
            {
              axisName: "tone",
              role: "variant",
              values: [
                { name: "danger", classNames: ["button_tone_danger"] },
                { name: "primary", classNames: ["button_tone_primary"] },
              ],
            },
            {
              axisName: "size",
              defaultValue: "sm",
              role: "variant",
              values: [
                { name: "lg", classNames: ["button_size_lg"] },
                { name: "sm", classNames: ["button_size_sm"] },
              ],
            },
            {
              axisName: "slots",
              role: "slot",
              reserved: true,
              values: [],
            },
          ],
          compoundVariants: [
            {
              conditions: [
                { axisName: "size", value: "sm" },
                { axisName: "tone", value: "primary" },
              ],
              classNames: ["button_primary_sm"],
            },
          ],
        },
      },
    ]);
  });

  it("tracks Vue useCssModule style references from projected SFC script", () => {
    const vueSource = `
      <template>
        <button :class="$style.button">Save</button>
      </template>
      <script setup lang="ts">
      import { useCssModule } from 'vue';
      const styles = useCssModule();
      const named = useCssModule("theme");
      const className = styles.button + named["accent"];
      </script>
      <style module>
      .button {}
      </style>
    `;
    const sourceFile = parse(
      projectVueSfcScriptToTypeScriptSource(vueSource),
      "/fake/src/Card.vue",
    );
    const sourceBinder = buildSourceBinder(sourceFile);

    const result = vueStyleModuleBinderPluginV0.analyzeSource({
      sourceFile,
      filePath: "/fake/src/Card.vue",
      sourceBinder,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
    });

    expect(result.stylesBindings.size).toBe(0);
    expect(result.classExpressions).toEqual([]);
    expect(result.domainClassReferences).toMatchObject([
      {
        matchKind: "literal",
        className: "default.button",
        domain: "vue-style-module",
        origin: "styleAccess",
      },
      {
        matchKind: "literal",
        className: "theme.accent",
        domain: "vue-style-module",
        origin: "styleAccess",
      },
    ]);
  });
});
