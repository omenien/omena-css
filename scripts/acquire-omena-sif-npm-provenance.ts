import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

type AcquisitionOptions = {
  readonly packageSpec: string;
  readonly outputPath: string;
  readonly receiptPath: string;
  readonly metadataFile?: string;
};

const options = parseOptions(process.argv.slice(2).filter((argument) => argument !== "--"));
const source = acquireMetadata(options);
let metadata: unknown;
try {
  metadata = JSON.parse(source);
} catch (error) {
  throw new Error(`npm metadata is not valid JSON: ${formatError(error)}`);
}

const normalizedMetadata = `${JSON.stringify(metadata, null, 2)}\n`;
writeText(options.outputPath, normalizedMetadata);
const provenanceCandidateCount = countProvenanceCandidates(metadata);
const receipt = {
  schemaVersion: "1",
  product: "omena-sif.npm-provenance-acquisition",
  acquisitionOwner: options.metadataFile ? "operator-supplied-metadata" : "platform-npm-cli",
  networkAccess: options.metadataFile ? "none" : "npm-cli-owned",
  packageSpec: options.packageSpec,
  npmMetadataPath: options.outputPath,
  provenanceDisposition: provenanceCandidateCount === 0 ? "absent" : "present",
  provenanceCandidateCount,
  command: options.metadataFile ? null : ["npm", "view", options.packageSpec, "--json"],
} as const;
writeText(options.receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
process.stdout.write(`${JSON.stringify(receipt)}\n`);

function parseOptions(arguments_: readonly string[]): AcquisitionOptions {
  let packageSpec: string | undefined;
  let outputPath: string | undefined;
  let receiptPath: string | undefined;
  let metadataFile: string | undefined;
  for (let index = 0; index < arguments_.length; index += 1) {
    const argument = arguments_[index];
    if (argument === "--output") {
      outputPath = requiredOptionValue(arguments_, ++index, argument);
    } else if (argument === "--receipt") {
      receiptPath = requiredOptionValue(arguments_, ++index, argument);
    } else if (argument === "--metadata-file") {
      metadataFile = requiredOptionValue(arguments_, ++index, argument);
    } else if (argument?.startsWith("--")) {
      throw new Error(`unknown option: ${argument}`);
    } else if (argument && packageSpec === undefined) {
      packageSpec = argument;
    } else {
      throw new Error(`unexpected argument: ${argument ?? "<missing>"}`);
    }
  }
  if (!packageSpec) throw new Error("package spec is required");
  if (!outputPath) throw new Error("--output is required");
  return {
    packageSpec,
    outputPath,
    receiptPath: receiptPath ?? `${outputPath}.receipt.json`,
    ...(metadataFile ? { metadataFile } : {}),
  };
}

function requiredOptionValue(arguments_: readonly string[], index: number, option: string): string {
  const value = arguments_[index];
  if (!value || value.startsWith("--")) throw new Error(`${option} requires a value`);
  return value;
}

function acquireMetadata(options: AcquisitionOptions): string {
  if (options.metadataFile) return readFileSync(options.metadataFile, "utf8");
  const result = spawnSync("npm", ["view", options.packageSpec, "--json"], {
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(
      `npm view failed (status=${result.status})\nstdout=${result.stdout}\nstderr=${result.stderr}`,
    );
  }
  return result.stdout;
}

function countProvenanceCandidates(value: unknown): number {
  if (!isRecord(value)) return 0;
  let count = countProvenanceValue(value.dist) + countProvenanceValue(value.attestations);
  if (isRecord(value.versions)) {
    for (const version of Object.values(value.versions)) {
      count += countProvenanceCandidates(version);
    }
  }
  return count;
}

function countProvenanceValue(value: unknown): number {
  if (!isRecord(value)) return 0;
  const attestations = isRecord(value.attestations) ? value.attestations : value;
  const provenance = attestations.provenance;
  if (provenance === undefined || provenance === null) return 0;
  return Array.isArray(provenance) ? provenance.length : 1;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function writeText(outputPath: string, source: string): void {
  mkdirSync(path.dirname(path.resolve(outputPath)), { recursive: true });
  writeFileSync(outputPath, source, "utf8");
}

function formatError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
