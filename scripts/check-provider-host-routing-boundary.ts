import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import { readFileSync, statSync } from "node:fs";
import path from "node:path";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

interface SourceEdge {
  readonly kind: "import" | "export";
  readonly filePath: string;
  readonly line: number;
  readonly isTypeOnly: boolean;
  readonly specifier: string;
}

const ROOT = process.cwd();
const PROVIDER_ROOT = path.join(ROOT, "server/lsp-server/src/providers");

const REQUIRED_HOST_QUERY_IMPORTS: ReadonlyMap<string, readonly string[]> = new Map([
  ["code-actions.ts", ["code-action-query"]],
  ["completion.ts", ["source-completion-query", "style-completion-query"]],
  ["cursor-dispatch.ts", ["source-cursor-query"]],
  ["definition.ts", ["source-definition-query", "style-definition-query"]],
  ["diagnostics.ts", ["source-diagnostics-query"]],
  ["hover.ts", ["source-hover-query", "style-hover-query"]],
  ["reference-lens.ts", ["style-reference-lens-query"]],
  ["references.ts", ["source-references-query", "style-references-query"]],
  ["rename/index.ts", ["source-rename-query", "style-rename-query"]],
  ["scss-diagnostics.ts", ["style-diagnostics-query"]],
]);

const FROM_EDGE_RE = /\b(import|export)\s+(type\s+)?([\s\S]*?)\s+from\s+["']([^"']+)["']/g;

function main() {
  const providerFiles = listProviderSourceFiles(PROVIDER_ROOT);
  const edges = providerFiles.flatMap((filePath) => readSourceEdges(filePath));
  const errors = [...findForbiddenCoreEdges(edges), ...findMissingHostQueryImports(edges)];

  if (errors.length > 0) {
    for (const error of errors) {
      process.stderr.write(`${error}\n`);
    }
    process.exit(1);
  }

  process.stdout.write(
    [
      "provider host routing boundary ok",
      `providers=${providerFiles.length}`,
      `hostImports=${countHostQueryEdges(edges)}`,
      `checkedEdges=${edges.length}`,
    ].join(" ") + "\n",
  );
}

function listProviderSourceFiles(dirPath: string): string[] {
  const files: string[] = [];
  for (const entry of evidenceScanSurface.readdirSync(dirPath)) {
    const entryPath = path.join(dirPath, entry);
    const stat = statSync(entryPath);
    if (stat.isDirectory()) {
      files.push(...listProviderSourceFiles(entryPath));
    } else if (entryPath.endsWith(".ts")) {
      files.push(entryPath);
    }
  }
  return files.toSorted();
}

function readSourceEdges(filePath: string): SourceEdge[] {
  const source = readFileSync(filePath, "utf8");
  const edges: SourceEdge[] = [];

  for (const match of source.matchAll(FROM_EDGE_RE)) {
    const [, kind, typeKeyword, importClause, specifier] = match;
    if (!kind || !specifier) continue;
    edges.push({
      kind: kind as SourceEdge["kind"],
      filePath,
      line: lineAt(source, match.index ?? 0),
      isTypeOnly: typeKeyword !== undefined || isAllTypeNamedClause(importClause ?? ""),
      specifier,
    });
  }

  return edges;
}

function isAllTypeNamedClause(importClause: string): boolean {
  const trimmed = importClause.trim();
  if (!trimmed.startsWith("{") || !trimmed.endsWith("}")) return false;

  const namedSpecifiers = trimmed
    .slice(1, -1)
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean);

  return namedSpecifiers.length > 0 && namedSpecifiers.every((part) => part.startsWith("type "));
}

function findForbiddenCoreEdges(edges: readonly SourceEdge[]): string[] {
  const errors: string[] = [];

  for (const edge of edges) {
    const corePath = coreSpecifierPath(edge.specifier);
    if (!corePath) continue;

    if (corePath.startsWith("core/query")) {
      errors.push(
        formatEdgeError(edge, "provider must route core query access through engine-host-node"),
      );
      continue;
    }

    if (corePath.startsWith("core/semantic")) {
      errors.push(
        formatEdgeError(edge, "provider must not import semantic graph internals directly"),
      );
      continue;
    }

    if (corePath.startsWith("core/indexing")) {
      errors.push(formatEdgeError(edge, "provider must not import indexing internals directly"));
      continue;
    }

    if (corePath.startsWith("core/ts")) {
      errors.push(
        formatEdgeError(edge, "provider must not import TypeScript resolver internals directly"),
      );
      continue;
    }

    if (corePath.startsWith("core/checker") && !edge.isTypeOnly) {
      errors.push(formatEdgeError(edge, "provider may only type-import checker findings"));
      continue;
    }

    if (corePath.startsWith("core/hir") && !edge.isTypeOnly) {
      errors.push(formatEdgeError(edge, "provider may only type-import HIR shapes"));
    }
  }

  return errors;
}

function findMissingHostQueryImports(edges: readonly SourceEdge[]): string[] {
  const errors: string[] = [];

  for (const [relativeFilePath, requiredModules] of REQUIRED_HOST_QUERY_IMPORTS) {
    const fileEdges = edges.filter(
      (edge) => providerRelativePath(edge.filePath) === relativeFilePath,
    );
    for (const requiredModule of requiredModules) {
      const hasModule = fileEdges.some(
        (edge) =>
          edge.specifier.includes("engine-host-node/src/") &&
          edge.specifier.endsWith(`/src/${requiredModule}`),
      );
      if (!hasModule) {
        errors.push(`${relativeFilePath}: missing engine-host-node route for ${requiredModule}`);
      }
    }
  }

  return errors;
}

function countHostQueryEdges(edges: readonly SourceEdge[]): number {
  return edges.filter((edge) => edge.specifier.includes("engine-host-node/src/")).length;
}

function coreSpecifierPath(specifier: string): string | null {
  const marker = "engine-core-ts/src/";
  const markerIndex = specifier.indexOf(marker);
  return markerIndex === -1 ? null : specifier.slice(markerIndex + marker.length);
}

function formatEdgeError(edge: SourceEdge, message: string): string {
  return `${providerRelativePath(edge.filePath)}:${edge.line}: ${message}: ${edge.specifier}`;
}

function providerRelativePath(filePath: string): string {
  return path.relative(PROVIDER_ROOT, filePath).split(path.sep).join("/");
}

function lineAt(source: string, index: number): number {
  return source.slice(0, index).split("\n").length;
}

main();
