import { describe, expect, it } from "vitest";
import {
  planSettingsReload,
  planWatchedFileInvalidation,
  type OpenDocumentSnapshot,
} from "../../../server/engine-host-node/src/runtime/invalidation-planner";

function doc(uri: string, workspaceRoot: string, isStyle = false): OpenDocumentSnapshot {
  return {
    uri,
    filePath: uri.replace("file://", ""),
    isStyle,
    workspaceRoot,
  };
}

describe("planSettingsReload", () => {
  it("reschedules only affected roots and source documents", () => {
    const rootA = "/ws-a";
    const rootB = "/ws-b";
    const openDocuments = [
      doc("file:///ws-a/src/App.tsx", rootA),
      doc("file:///ws-a/src/Button.module.scss", rootA, true),
      doc("file:///ws-b/src/App.tsx", rootB),
      doc("file:///ws-b/src/Button.module.scss", rootB, true),
    ];

    const plan = planSettingsReload(
      [
        {
          workspaceRoot: rootA,
          aliasChanged: false,
          modeChanged: true,
          settingsKeyChanged: true,
          affectedSettingsDependencyUris: ["file:///ws-a/src/App.tsx"],
        },
        {
          workspaceRoot: rootB,
          aliasChanged: false,
          modeChanged: false,
          settingsKeyChanged: false,
          affectedSettingsDependencyUris: [],
        },
      ],
      openDocuments,
    );

    expect(plan.resourceChanged).toBe(true);
    expect(plan.aliasRebuildRoots).toEqual([]);
    expect(plan.affectedStyleRoots).toEqual([rootA]);
    expect(plan.affectedSourceUris).toEqual(["file:///ws-a/src/App.tsx"]);
  });

  it("rebuilds alias resolvers and invalidates all source docs in changed roots", () => {
    const root = "/ws";
    const openDocuments = [
      doc("file:///ws/src/App.tsx", root),
      doc("file:///ws/src/Other.tsx", root),
      doc("file:///ws/src/Button.module.scss", root, true),
    ];

    const plan = planSettingsReload(
      [
        {
          workspaceRoot: root,
          aliasChanged: true,
          modeChanged: false,
          settingsKeyChanged: true,
          affectedSettingsDependencyUris: [],
        },
      ],
      openDocuments,
    );

    expect(plan.aliasRebuildRoots).toEqual([root]);
    expect(plan.affectedStyleRoots).toEqual([]);
    expect(plan.affectedSourceUris).toEqual(["file:///ws/src/App.tsx", "file:///ws/src/Other.tsx"]);
  });
});

describe("planWatchedFileInvalidation", () => {
  it("limits source invalidation to dependent documents and schedules root style docs", () => {
    const root = "/ws";
    const openDocuments = [
      doc("file:///ws/src/App.tsx", root),
      doc("file:///ws/src/Other.tsx", root),
      doc("file:///ws/src/Button.module.scss", root, true),
    ];

    const plan = planWatchedFileInvalidation(
      [
        {
          kind: "style",
          workspaceRoot: root,
          filePath: "/ws/src/Button.module.scss",
          changeType: "changed",
          semanticsChanged: true,
          dependentSourceUris: ["file:///ws/src/App.tsx"],
        },
      ],
      openDocuments,
    );

    expect(plan.stylePathsToInvalidate).toEqual(["/ws/src/Button.module.scss"]);
    expect(plan.stylePathsToPush).toEqual(["/ws/src/Button.module.scss"]);
    expect(plan.affectedWorkspaceRoots).toEqual([root]);
    expect(plan.affectedSourceUris).toEqual(["file:///ws/src/App.tsx"]);
  });

  it("rebuilds alias resolver and invalidates every open source doc on project config change", () => {
    const root = "/ws";
    const openDocuments = [
      doc("file:///ws/src/App.tsx", root),
      doc("file:///ws/src/Other.tsx", root),
      doc("file:///ws/src/Button.module.scss", root, true),
    ];

    const plan = planWatchedFileInvalidation(
      [
        {
          kind: "source",
          workspaceRoot: root,
          filePath: "/ws/tsconfig.json",
          changeType: "changed",
          projectConfigChange: true,
          dependentSourceUris: [],
        },
      ],
      openDocuments,
    );

    expect(plan.aliasRebuildRoots).toEqual([root]);
    expect(plan.affectedWorkspaceRoots).toEqual([root]);
    expect(plan.typeResolverInvalidationRoots).toEqual([root]);
    expect(plan.affectedSourceUris).toEqual(["file:///ws/src/App.tsx", "file:///ws/src/Other.tsx"]);
  });

  it("does not reschedule source docs on declaration-only style changes", () => {
    const root = "/ws";
    const openDocuments = [
      doc("file:///ws/src/App.tsx", root),
      doc("file:///ws/src/Button.module.scss", root, true),
    ];

    const plan = planWatchedFileInvalidation(
      [
        {
          kind: "style",
          workspaceRoot: root,
          filePath: "/ws/src/Button.module.scss",
          changeType: "changed",
          semanticsChanged: false,
          dependentSourceUris: [],
        },
      ],
      openDocuments,
    );

    expect(plan.stylePathsToInvalidate).toEqual([]);
    expect(plan.stylePathsToPush).toEqual([]);
    expect(plan.affectedWorkspaceRoots).toEqual([root]);
    expect(plan.affectedSourceUris).toEqual([]);
  });
});
