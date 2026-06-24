import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { buildCheckerEngineParitySnapshotV2 } from "../../../server/engine-host-node/src/engine-parity-v2";
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

describe("buildCheckerEngineParitySnapshotV2", () => {
  it("assembles a v2 parity snapshot with constrained query metadata", async () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        "import classNames from 'classnames/bind';",
        "import styles from './Button.module.scss';",
        "const cx = classNames.bind(styles);",
        "const size = true ? 'missing' : 'missing';",
        "const cls = cx(size);",
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

    const snapshot = await buildCheckerEngineParitySnapshotV2({
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

    expect(snapshot.input.version).toBe("2");
    expect(snapshot.output.version).toBe("2");
    expect(snapshot.input.typeFacts[0]?.facts.kind).toMatch(
      /^(exact|finiteSet|unknown|top|constrained)$/,
    );
    expect(snapshot.output.queryResults).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: "expression-semantics",
          filePath: path.join(workspaceRoot, "src/App.tsx"),
          payload: expect.objectContaining({
            valueDomainKind: "exact",
            valueCertaintyShapeKind: "exact",
          }),
        }),
      ]),
    );
  });

  it("emits bundle-1 constrained query payloads for prefix-suffix flow", async () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        'import classNames from "classnames/bind";',
        'import styles from "./Button.module.scss";',
        "",
        "const cx = classNames.bind(styles);",
        "",
        "export function App(variant: string) {",
        '  const className = "btn-" + variant + "-chip";',
        "  return <div className={cx(className)} />;",
        "}",
        "",
      ].join("\n"),
      "src/Button.module.scss": [
        ".btn-idle-chip {}",
        ".btn-busy-chip {}",
        ".btn-error-chip {}",
      ].join("\n"),
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

    const snapshot = await buildCheckerEngineParitySnapshotV2({
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
          payload: expect.objectContaining({
            valueDomainKind: "constrained",
            valueConstraintKind: "prefixSuffix",
            valueCertaintyShapeKind: "constrained",
            valueCertaintyConstraintKind: "prefixSuffix",
            selectorCertaintyShapeKind: "constrained",
            selectorNames: ["btn-idle-chip", "btn-busy-chip", "btn-error-chip"],
          }),
        }),
      ]),
    );
  });

  it("emits bundle-3 composite query payloads for widened flow", async () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        'import classNames from "classnames/bind";',
        'import styles from "./Button.module.scss";',
        "const cx = classNames.bind(styles);",
        "function resolveVariant(value: number) {",
        "  switch (value) {",
        '    case 1: return "btn-primary";',
        '    case 2: return "btn-secondary";',
        '    case 3: return "btn-danger";',
        '    case 4: return "btn-success";',
        '    case 5: return "btn-warning";',
        '    case 6: return "btn-info";',
        '    case 7: return "btn-muted";',
        '    case 8: return "btn-ghost";',
        '    default: return "btn-outline";',
        "  }",
        "}",
        "export function App(value: number) {",
        "  const variant = resolveVariant(value);",
        "  return <div className={cx(variant)} />;",
        "}",
        "",
      ].join("\n"),
      "src/Button.module.scss": ".btn-primary {}",
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

    const snapshot = await buildCheckerEngineParitySnapshotV2({
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
          payload: expect.objectContaining({
            valueDomainKind: "constrained",
            valueConstraintKind: "composite",
            valuePrefix: "btn-",
            valueMinLen: 8,
            valueCharMust: "-bnt",
            valueCharMay: "-abcdefghilmnoprstuwy",
            valueCertaintyShapeKind: "constrained",
            valueCertaintyConstraintKind: "composite",
            selectorCertaintyShapeKind: "constrained",
          }),
        }),
      ]),
    );
  });
});

function makeWorkspace(files: Record<string, string>): string {
  const root = mkdtempSync(path.join(os.tmpdir(), "engine-parity-v2-"));
  tempDirs.push(root);
  for (const [relativePath, content] of Object.entries(files)) {
    const absolutePath = path.join(root, relativePath);
    mkdirSync(path.dirname(absolutePath), { recursive: true });
    writeFileSync(absolutePath, content, "utf8");
  }
  return root;
}
