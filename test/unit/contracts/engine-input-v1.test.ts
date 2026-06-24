import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { buildEngineInputV1Async } from "../../../server/engine-host-node/src/historical/engine-input-v1";
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

describe("buildEngineInputV1", () => {
  it("assembles normalized source/style facts and type facts", async () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        "import classNames from 'classnames/bind';",
        "import styles from './Button.module.scss';",
        "const cx = classNames.bind(styles);",
        "export function App({ size }: { size: 'primary' | 'secondary' }) {",
        "  return cx(size);",
        "}",
        "",
      ].join("\n"),
      "src/Button.module.scss": ".primary {}\n.secondary {}",
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

    const input = await buildEngineInputV1Async({
      workspaceRoot,
      classnameTransform: "asIs",
      pathAlias: {},
      sourceDocuments,
      styleFiles,
      analysisCache: analysisHost.analysisCache,
      styleDocumentForPath: styleHost.styleDocumentForPath,
      typeResolver: analysisHost.typeResolver,
    });

    expect(input.version).toBe("1");
    expect(input.workspace).toEqual({
      root: workspaceRoot,
      classnameTransform: "asIs",
      settingsKey: "transform:asIs;alias:",
    });
    expect(input.sources).toHaveLength(1);
    expect(input.styles).toHaveLength(1);
    expect(input.sources[0]?.bindingGraph.declarations.length).toBeGreaterThan(0);
    expect(input.typeFacts).toHaveLength(1);
    expect(input.typeFacts[0]).toMatchObject({
      filePath: path.join(workspaceRoot, "src/App.tsx"),
      expressionId: expect.any(String),
    });
    expect(["unknown", "exact", "finiteSet", "prefix", "top"]).toContain(
      input.typeFacts[0]?.facts.kind,
    );
  });
});

function makeWorkspace(files: Record<string, string>): string {
  const root = mkdtempSync(path.join(os.tmpdir(), "engine-input-v1-"));
  tempDirs.push(root);
  for (const [relativePath, content] of Object.entries(files)) {
    const absolutePath = path.join(root, relativePath);
    mkdirSync(path.dirname(absolutePath), { recursive: true });
    writeFileSync(absolutePath, content, "utf8");
  }
  return root;
}
