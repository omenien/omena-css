import { describe, expect, it } from "vitest";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import { createRuntimeDependencySnapshot } from "../../../server/engine-host-node/src/runtime/dependency-snapshot";
import { collectWatchedFileChangeInputs } from "../../../server/engine-host-node/src/runtime/watched-file-changes";
import { makeBaseDeps, semanticSiteAt } from "../../_fixtures/test-helpers";
import {
  buildStyleDocumentFromSelectorMap,
  makeTestSelector,
} from "../../_fixtures/style-documents";

describe("collectWatchedFileChangeInputs", () => {
  it("marks project config source changes and preserves dependent source URIs", () => {
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record(
      "file:///fake/ws/src/App.tsx",
      [
        semanticSiteAt(
          "file:///fake/ws/src/App.tsx",
          "button",
          1,
          "/fake/ws/src/Button.module.scss",
        ),
      ],
      [],
      {
        workspaceRoot: "/fake/ws",
        settingsKey: "transform:asIs;alias:",
        stylePaths: ["/fake/ws/src/Button.module.scss"],
        sourcePaths: ["/fake/ws/tsconfig.json"],
      },
    );
    const deps = makeBaseDeps({
      workspaceRoot: "/fake/ws",
      workspaceFolderUri: "file:///fake/ws",
      semanticReferenceIndex,
    });
    const snapshot = createRuntimeDependencySnapshot([deps], []);

    const changes = collectWatchedFileChangeInputs(
      [{ uri: "file:///fake/ws/tsconfig.json", type: "changed" }],
      {
        documents: { get: () => undefined },
        getDepsForFilePath: () => deps,
      },
      snapshot,
    );

    expect(changes).toEqual([
      {
        kind: "source",
        workspaceRoot: "/fake/ws",
        filePath: "/fake/ws/tsconfig.json",
        projectConfigChange: true,
        dependentSourceUris: ["file:///fake/ws/src/App.tsx"],
      },
    ]);
  });

  it("marks bundler config source changes as project config changes", () => {
    const deps = makeBaseDeps({
      workspaceRoot: "/fake/ws",
      workspaceFolderUri: "file:///fake/ws",
    });
    const snapshot = createRuntimeDependencySnapshot([deps], []);

    const changes = collectWatchedFileChangeInputs(
      [
        { uri: "file:///fake/ws/vite.config.ts", type: "changed" },
        { uri: "file:///fake/ws/webpack.config.cjs", type: "changed" },
      ],
      {
        documents: { get: () => undefined },
        getDepsForFilePath: () => deps,
      },
      snapshot,
    );

    expect(
      changes.map((change) => [
        change.filePath,
        change.kind === "source" ? change.projectConfigChange : null,
      ]),
    ).toEqual([
      ["/fake/ws/vite.config.ts", true],
      ["/fake/ws/webpack.config.cjs", true],
    ]);
  });

  it("marks declaration-only style changes as non-semantic", () => {
    const previous = buildStyleDocumentFromSelectorMap(
      "/fake/ws/src/Button.module.scss",
      new Map([["button", makeTestSelector("button", 1, { declarations: "color: red;" })]]),
    );
    const deps = makeBaseDeps({
      workspaceRoot: "/fake/ws",
      workspaceFolderUri: "file:///fake/ws",
      peekStyleDocument: () => previous,
      buildStyleDocument: (filePath, content) =>
        buildStyleDocumentFromSelectorMap(
          filePath,
          new Map([["button", makeTestSelector("button", 1, { declarations: content })]]),
        ),
      readStyleFile: () => "color: blue;",
    });
    const snapshot = createRuntimeDependencySnapshot([deps], []);

    const changes = collectWatchedFileChangeInputs(
      [{ uri: "file:///fake/ws/src/Button.module.scss", type: "changed" }],
      {
        documents: { get: () => undefined },
        getDepsForFilePath: () => deps,
      },
      snapshot,
    );

    expect(changes).toEqual([
      {
        kind: "style",
        workspaceRoot: "/fake/ws",
        filePath: "/fake/ws/src/Button.module.scss",
        changeType: "changed",
        semanticsChanged: false,
        dependentSourceUris: [],
      },
    ]);
  });
});
