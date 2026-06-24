import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { buildCheckerEngineParitySnapshotV1 } from "../../../server/engine-host-node/src/historical/engine-parity-v1";
import { runWorkspaceCheckCommand } from "../../../server/engine-host-node/src/checker-host";
import {
  collectSourceDocuments,
  createWorkspaceAnalysisHost,
  createWorkspaceStyleHost,
  resolveWorkspaceCheckFiles,
} from "../../../server/engine-host-node/src/checker-host/workspace-check-support";

const tempDirs: string[] = [];

afterEach(() => {
  for (const dir of tempDirs.splice(0)) {
    rmSync(dir, { recursive: true, force: true });
  }
});

describe("buildCheckerEngineParitySnapshotV1", () => {
  it("assembles a parity snapshot from current host outputs", async () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        "import classNames from 'classnames/bind';",
        "import styles from './Button.module.scss';",
        "const cx = classNames.bind(styles);",
        "const cls = cx('missing');",
        "",
      ].join("\n"),
      "src/Button.module.scss": ".button {}",
    });

    const { sourceFiles, styleFiles } = await resolveWorkspaceCheckFiles({ workspaceRoot });
    const styleHost = createWorkspaceStyleHost({
      styleFiles,
      classnameTransform: "asIs",
    });
    styleHost.preloadStyleDocuments();
    const analysisHost = createWorkspaceAnalysisHost({
      workspaceRoot,
      classnameTransform: "asIs",
      pathAlias: {},
      styleDocumentForPath: styleHost.styleDocumentForPath,
    });
    const sourceDocuments = collectSourceDocuments(sourceFiles, analysisHost.analysisCache);
    const command = await runWorkspaceCheckCommand({
      workspace: { workspaceRoot },
      filters: {
        preset: "ci",
        category: "all",
        severity: "all",
        includeBundles: ["ci-default"],
        includeCodes: [],
        excludeCodes: [],
      },
    });

    const snapshot = await buildCheckerEngineParitySnapshotV1({
      workspaceRoot,
      classnameTransform: "asIs",
      pathAlias: {},
      sourceDocuments,
      styleFiles,
      analysisCache: analysisHost.analysisCache,
      styleDocumentForPath: styleHost.styleDocumentForPath,
      typeResolver: analysisHost.typeResolver,
      checkerReport: command.checkerReport,
      semanticReferenceIndex: analysisHost.semanticReferenceIndex,
      styleDependencyGraph: styleHost.styleDependencyGraph,
    });

    expect(snapshot.input.version).toBe("1");
    expect(snapshot.output.version).toBe("1");
    expect(snapshot.output.checkerReport.version).toBe("1");
    expect(snapshot.output.checkerReport.summary).toEqual({
      warnings: 1,
      hints: 1,
      total: 2,
    });
    expect(snapshot.output.queryResults).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: "expression-semantics",
          filePath: path.join(workspaceRoot, "src/App.tsx"),
        }),
        expect.objectContaining({
          kind: "source-expression-resolution",
          filePath: path.join(workspaceRoot, "src/App.tsx"),
        }),
        expect.objectContaining({
          kind: "selector-usage",
          filePath: path.join(workspaceRoot, "src/Button.module.scss"),
        }),
      ]),
    );
    expect(snapshot.output.rewritePlans).toEqual([]);
  });

  it("captures query analysis metadata for local flow finite sets", async () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        "import classNames from 'classnames/bind';",
        "import styles from './Button.module.scss';",
        "const cx = classNames.bind(styles);",
        "export function App(enabled: boolean) {",
        "  const size = enabled ? 'small' : 'large';",
        "  return <div className={cx(size)} />;",
        "}",
        "",
      ].join("\n"),
      "src/Button.module.scss": ".small {}",
    });

    const { sourceFiles, styleFiles } = await resolveWorkspaceCheckFiles({ workspaceRoot });
    const styleHost = createWorkspaceStyleHost({
      styleFiles,
      classnameTransform: "asIs",
    });
    styleHost.preloadStyleDocuments();
    const analysisHost = createWorkspaceAnalysisHost({
      workspaceRoot,
      classnameTransform: "asIs",
      pathAlias: {},
      styleDocumentForPath: styleHost.styleDocumentForPath,
    });
    const sourceDocuments = collectSourceDocuments(sourceFiles, analysisHost.analysisCache);
    const command = await runWorkspaceCheckCommand({
      workspace: { workspaceRoot },
      filters: {
        preset: "changed-source",
        category: "source",
        severity: "all",
        includeBundles: ["source-missing"],
        includeCodes: [],
        excludeCodes: [],
      },
    });

    const snapshot = await buildCheckerEngineParitySnapshotV1({
      workspaceRoot,
      classnameTransform: "asIs",
      pathAlias: {},
      sourceDocuments,
      styleFiles,
      analysisCache: analysisHost.analysisCache,
      styleDocumentForPath: styleHost.styleDocumentForPath,
      typeResolver: analysisHost.typeResolver,
      checkerReport: command.checkerReport,
      semanticReferenceIndex: analysisHost.semanticReferenceIndex,
      styleDependencyGraph: styleHost.styleDependencyGraph,
    });

    expect(snapshot.output.queryResults).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: "expression-semantics",
          filePath: path.join(workspaceRoot, "src/App.tsx"),
          payload: expect.objectContaining({
            valueCertainty: "inferred",
            valueCertaintyShapeLabel: "bounded finite (2)",
            valueCertaintyReason: "analysis preserved multiple finite candidate values",
          }),
        }),
        expect.objectContaining({
          kind: "source-expression-resolution",
          filePath: path.join(workspaceRoot, "src/App.tsx"),
          payload: expect.objectContaining({
            valueCertainty: "inferred",
            valueCertaintyShapeLabel: "bounded finite (2)",
            valueCertaintyReason: "analysis preserved multiple finite candidate values",
          }),
        }),
      ]),
    );
  });
});

function makeWorkspace(files: Record<string, string>): string {
  const root = mkdtempSync(path.join(os.tmpdir(), "engine-parity-v1-"));
  tempDirs.push(root);
  for (const [relativePath, content] of Object.entries(files)) {
    const absolutePath = path.join(root, relativePath);
    mkdirSync(path.dirname(absolutePath), { recursive: true });
    writeFileSync(absolutePath, content, "utf8");
  }
  return root;
}
