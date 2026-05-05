import { describe, expect, it } from "vitest";
import { cssModulesClassnamesBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/binder-plugin";
import { tailwindUnoUtilityBinderPluginV0 } from "../../../server/engine-core-ts/src/core/binder/tailwind-utility-plugin";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { readDomainClassReferenceSummary } from "../../../server/engine-core-ts/src/core/query";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import { EMPTY_ALIAS_RESOLVER } from "../../_fixtures/test-helpers";

describe("readDomainClassReferenceSummary", () => {
  it("summarizes utility-domain class tracking separately from CSS Module references", () => {
    const cache = new DocumentAnalysisCache({
      sourceFileCache: new SourceFileCache({ max: 10 }),
      binderPlugins: [cssModulesClassnamesBinderPluginV0, tailwindUnoUtilityBinderPluginV0],
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });
    const entry = cache.get(
      "file:///fake/Card.tsx",
      `
        import classNames from 'classnames/bind';
        import clsx from 'clsx';
        import styles from './Card.module.scss';
        const cx = classNames.bind(styles);
        const el = <div className={clsx(cx('card'), "flex", \`tone-\${state}\`)} />;
      `,
      "/fake/Card.tsx",
      1,
    );

    const summary = readDomainClassReferenceSummary(entry.sourceDocument);

    expect(entry.sourceDocument.classExpressions).toMatchObject([
      { kind: "literal", className: "card" },
    ]);
    expect(summary).toMatchObject({
      totalReferences: 2,
      hasUtilityDomainReferences: true,
      groups: [
        {
          pluginId: "tailwind-uno-utility-domain",
          domain: "utility-css",
          literalCount: 1,
          templatePrefixCount: 1,
        },
      ],
    });
  });
});
