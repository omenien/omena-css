import path from "node:path";
import type { ContractParityEntry } from "./contract-parity-corpus-v1";

const workspaceRoot = process.cwd();

export interface OmenaSemanticObservationCorpusEntry {
  readonly label: string;
  readonly styleFilePath: string;
  readonly contract: ContractParityEntry;
  readonly expected: {
    readonly selectorIdentityStatus: "ready" | "partial" | "gap";
    readonly sourceEvidenceStatus: "ready" | "partial" | "gap";
    readonly downstreamReadinessStatus: "ready" | "partial" | "gap";
    readonly minSelectorCount: number;
    readonly minExpressionCount: number;
    readonly ready: boolean;
    readonly publishReady: boolean;
    readonly publishBlockingGaps: readonly string[];
    readonly observationGaps: readonly string[];
  };
}

export const OMENA_SEMANTIC_OBSERVATION_CORPUS: readonly OmenaSemanticObservationCorpusEntry[] = [
  {
    label: "bem-marker-parent-chain",
    styleFilePath: path.join(
      workspaceRoot,
      "test/_fixtures/omena-semantic-observation/bem-parent-chain/BemParentChain.module.scss",
    ),
    contract: {
      label: "bem-marker-parent-chain-omena-semantic-observation",
      contractVersion: "2",
      workspace: {
        workspaceRoot,
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/omena-semantic-observation/bem-parent-chain/BemParentChain.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/omena-semantic-observation/bem-parent-chain/BemParentChain.module.scss",
          ),
        ],
      },
      filters: {
        preset: "changed-source",
        category: "source",
        severity: "all",
        includeBundles: ["source-missing"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
    expected: {
      selectorIdentityStatus: "ready",
      sourceEvidenceStatus: "gap",
      downstreamReadinessStatus: "gap",
      minSelectorCount: 3,
      minExpressionCount: 0,
      ready: false,
      publishReady: true,
      publishBlockingGaps: [],
      observationGaps: ["sourceEvidence", "downstreamReadiness"],
    },
  },
  {
    label: "literal-union-type-fact",
    styleFilePath: path.join(
      workspaceRoot,
      "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
    ),
    contract: {
      label: "literal-union-omena-semantic-observation",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/literal-union",
        ),
        sourceFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.ts",
          ),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/literal-union/src/App.module.scss",
          ),
        ],
      },
      filters: {
        preset: "changed-source",
        category: "source",
        severity: "all",
        includeBundles: ["source-missing"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
    expected: {
      selectorIdentityStatus: "ready",
      sourceEvidenceStatus: "ready",
      downstreamReadinessStatus: "ready",
      minSelectorCount: 2,
      minExpressionCount: 1,
      ready: true,
      publishReady: true,
      publishBlockingGaps: [],
      observationGaps: [],
    },
  },
  {
    label: "composite-template-type-fact",
    styleFilePath: path.join(
      workspaceRoot,
      "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
    ),
    contract: {
      label: "composite-template-omena-semantic-observation",
      contractVersion: "2",
      workspace: {
        workspaceRoot: path.join(
          workspaceRoot,
          "test/_fixtures/type-fact-backend-parity/composite",
        ),
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/type-fact-backend-parity/composite/src/App.ts"),
        ],
        styleFilePaths: [
          path.join(
            workspaceRoot,
            "test/_fixtures/type-fact-backend-parity/composite/src/App.module.scss",
          ),
        ],
      },
      filters: {
        preset: "changed-source",
        category: "source",
        severity: "all",
        includeBundles: ["source-missing"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
    expected: {
      selectorIdentityStatus: "ready",
      sourceEvidenceStatus: "ready",
      downstreamReadinessStatus: "ready",
      minSelectorCount: 10,
      minExpressionCount: 1,
      ready: true,
      publishReady: true,
      publishBlockingGaps: [],
      observationGaps: [],
    },
  },
  {
    label: "real-project-button-variants",
    styleFilePath: path.join(
      workspaceRoot,
      "test/_fixtures/real-project-corpus/ButtonVariants.module.scss",
    ),
    contract: {
      label: "button-variants-omena-semantic-observation",
      contractVersion: "2",
      workspace: {
        workspaceRoot,
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/real-project-corpus/ButtonVariants.tsx"),
        ],
        styleFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/real-project-corpus/ButtonVariants.module.scss"),
        ],
      },
      filters: {
        preset: "ci",
        category: "all",
        severity: "all",
        includeBundles: ["ci-default"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
    expected: {
      selectorIdentityStatus: "ready",
      sourceEvidenceStatus: "partial",
      downstreamReadinessStatus: "gap",
      minSelectorCount: 10,
      minExpressionCount: 1,
      ready: false,
      publishReady: true,
      publishBlockingGaps: [],
      observationGaps: ["sourceEvidence", "downstreamReadiness"],
    },
  },
  {
    label: "real-project-less-analytics-grid",
    styleFilePath: path.join(
      workspaceRoot,
      "test/_fixtures/real-project-corpus/AnalyticsGrid.module.less",
    ),
    contract: {
      label: "analytics-grid-less-omena-semantic-observation",
      contractVersion: "2",
      workspace: {
        workspaceRoot,
        sourceFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/real-project-corpus/AnalyticsGrid.tsx"),
        ],
        styleFilePaths: [
          path.join(workspaceRoot, "test/_fixtures/real-project-corpus/AnalyticsGrid.module.less"),
        ],
      },
      filters: {
        preset: "ci",
        category: "all",
        severity: "all",
        includeBundles: ["ci-default"],
        includeCodes: [],
        excludeCodes: [],
      },
    },
    expected: {
      selectorIdentityStatus: "ready",
      sourceEvidenceStatus: "gap",
      downstreamReadinessStatus: "gap",
      minSelectorCount: 4,
      minExpressionCount: 0,
      ready: false,
      publishReady: true,
      publishBlockingGaps: [],
      observationGaps: ["sourceEvidence", "downstreamReadiness"],
    },
  },
] as const;
