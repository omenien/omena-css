import path from "node:path";
import type { Utf16CodeUnitLengthV2Json } from "../server/engine-core-ts/src/contracts/engine-v2-input-idl.generated";
import type { ContractParityEntry } from "./contract-parity-corpus-v1";

const workspaceRoot = process.cwd();

export interface ContractParityAuthoredMembershipV2 {
  readonly valueConstraintKind: "prefixSuffix" | "composite";
  readonly valuePrefix: string;
  readonly valueSuffix: string;
  readonly valueMinLen: Utf16CodeUnitLengthV2Json;
  readonly selectorNames: readonly string[];
}

export interface ContractParityEntryV2 extends ContractParityEntry {
  readonly authoredMembership?: ContractParityAuthoredMembershipV2;
}

export const CONTRACT_PARITY_CORPUS_V2: readonly ContractParityEntryV2[] = [
  {
    label: "type-fact-parity-v2",
    contractVersion: "2",
    workspace: {
      workspaceRoot,
      sourceFilePaths: [
        path.join(workspaceRoot, "test/_fixtures/contract-parity/TypeFactParity.tsx"),
      ],
      styleFilePaths: [
        path.join(workspaceRoot, "test/_fixtures/contract-parity/TypeFactParity.module.scss"),
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
  {
    label: "source-prefix-suffix-parity-v2",
    contractVersion: "2",
    workspace: {
      workspaceRoot,
      sourceFilePaths: [
        path.join(workspaceRoot, "test/_fixtures/contract-parity/SourcePrefixSuffixParity.tsx"),
      ],
      styleFilePaths: [
        path.join(
          workspaceRoot,
          "test/_fixtures/contract-parity/SourcePrefixSuffixParity.module.scss",
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
  {
    label: "source-char-inclusion-parity-v2",
    contractVersion: "2",
    workspace: {
      workspaceRoot,
      sourceFilePaths: [
        path.join(workspaceRoot, "test/_fixtures/contract-parity/SourceCharInclusionParity.tsx"),
      ],
      styleFilePaths: [
        path.join(
          workspaceRoot,
          "test/_fixtures/contract-parity/SourceCharInclusionParity.module.scss",
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
  {
    label: "source-prefix-suffix-overlap-parity-v2",
    contractVersion: "2",
    workspace: {
      workspaceRoot,
      sourceFilePaths: [
        path.join(
          workspaceRoot,
          "test/_fixtures/contract-parity/SourcePrefixSuffixOverlapParity.tsx",
        ),
      ],
      styleFilePaths: [
        path.join(
          workspaceRoot,
          "test/_fixtures/contract-parity/SourcePrefixSuffixOverlapParity.module.scss",
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
    authoredMembership: {
      valueConstraintKind: "composite",
      valuePrefix: "ab-",
      valueSuffix: "-cd",
      valueMinLen: 5,
      selectorNames: ["ab-cd", "ab-x-cd", "ab-long-cd"],
    },
  },
  {
    label: "source-composite-parity-v2",
    contractVersion: "2",
    workspace: {
      workspaceRoot,
      sourceFilePaths: [
        path.join(workspaceRoot, "test/_fixtures/contract-parity/SourceCompositeParity.tsx"),
      ],
      styleFilePaths: [
        path.join(
          workspaceRoot,
          "test/_fixtures/contract-parity/SourceCompositeParity.module.scss",
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
  {
    label: "source-unicode-length-parity-v2",
    contractVersion: "2",
    workspace: {
      workspaceRoot,
      sourceFilePaths: [
        path.join(workspaceRoot, "test/_fixtures/contract-parity/SourceUnicodeLengthParity.tsx"),
      ],
      styleFilePaths: [
        path.join(
          workspaceRoot,
          "test/_fixtures/contract-parity/SourceUnicodeLengthParity.module.scss",
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
    authoredMembership: {
      valueConstraintKind: "composite",
      valuePrefix: "카드-",
      valueSuffix: "-활성",
      valueMinLen: 10,
      selectorNames: ["카드-long-활성"],
    },
  },
] as const;
