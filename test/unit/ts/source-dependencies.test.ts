import { describe, expect, it } from "vitest";
import { AliasResolver } from "../../../server/engine-core-ts/src/core/cx/alias-resolver";
import { collectSourceDependencyPaths } from "../../../server/engine-core-ts/src/core/ts/source-dependencies";

describe("collectSourceDependencyPaths", () => {
  it("collects self path and relative import candidates", () => {
    expect(
      collectSourceDependencyPaths("/fake/ws/src/App.tsx", [
        "./theme",
        "./tokens",
        "./App.module.scss",
      ]),
    ).toEqual(
      expect.arrayContaining([
        "/fake/ws/src/App.tsx",
        "/fake/ws/src/theme.ts",
        "/fake/ws/src/theme.tsx",
        "/fake/ws/src/theme/index.ts",
        "/fake/ws/src/tokens.ts",
        "/fake/ws/src/tokens/index.ts",
      ]),
    );
  });

  it("keeps explicit source extensions and ignores non-source extensions", () => {
    expect(
      collectSourceDependencyPaths("/fake/ws/src/App.tsx", [
        "./theme.ts",
        "./data.json",
        "./App.module.scss",
      ]),
    ).toEqual(["/fake/ws/src/App.tsx", "/fake/ws/src/theme.ts"]);
  });

  it("collects aliased source candidates through the shared alias resolver", () => {
    const aliasResolver = new AliasResolver(
      "/fake/ws",
      {},
      {
        basePath: "/fake/ws/src",
        paths: {
          "@/*": ["*"],
        },
      },
    );

    expect(
      collectSourceDependencyPaths("/fake/ws/src/App.tsx", ["@/theme"], aliasResolver),
    ).toEqual(
      expect.arrayContaining([
        "/fake/ws/src/App.tsx",
        "/fake/ws/src/theme.ts",
        "/fake/ws/src/theme/index.ts",
      ]),
    );
  });
});
