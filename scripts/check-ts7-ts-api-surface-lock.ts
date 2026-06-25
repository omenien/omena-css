import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface TsApiSurfaceAllowlist {
  readonly allowedTsSymbols: readonly string[];
}

const moduleDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(moduleDir, "..");
const facadePath = path.join(repoRoot, "server/engine-core-ts/src/ts-facade.ts");
const allowlistPath = process.env.OMENA_TS_API_SURFACE_ALLOWLIST
  ? path.resolve(repoRoot, process.env.OMENA_TS_API_SURFACE_ALLOWLIST)
  : path.join(moduleDir, "ts7-ts-api-surface-allowlist.json");

const sourceRoots = ["server", "packages", "scripts"].map((entry) => path.join(repoRoot, entry));
const ignoredSegments = new Set(["dist", "node_modules"]);
const directTypeScriptImportPattern =
  /(?:from\s+["']typescript["']|import\s*\(\s*["']typescript["']\s*\)|require\s*\(\s*["']typescript["']\s*\))/;
const facadeImportPattern =
  /import\s+(?:type\s+)?([A-Za-z_$][\w$]*)(?:\s*,\s*\{[^}]*\})?\s+from\s+["'][^"']*ts-facade["']/g;
const lossyMethodPattern =
  /\.(?:getStart|getEnd|getText)\s*\(|getLineAndCharacterOfPosition\s*\(|getPositionOfLineAndCharacter\s*\(/;

function main(): void {
  const allowlist = readAllowlist();
  const files = sourceRoots.flatMap((root) => collectTsFiles(root));
  const diagnostics: string[] = [];
  const usedSymbols = new Set<string>();

  for (const filePath of files) {
    const source = readFileSync(filePath, "utf8");
    const relativePath = path.relative(repoRoot, filePath);
    const isFacade = path.resolve(filePath) === facadePath;

    if (!isFacade && directTypeScriptImportPattern.test(source)) {
      diagnostics.push(`${relativePath}: imports TypeScript directly instead of ts-facade`);
    }

    if (!isFacade && isEngineCoreSource(filePath) && lossyMethodPattern.test(source)) {
      diagnostics.push(`${relativePath}: uses lossy TypeScript node/source-file methods directly`);
    }

    if (isFacade) {
      for (const symbol of collectTsSymbols(source, "ts")) {
        usedSymbols.add(symbol);
      }
    }

    for (const localName of collectFacadeDefaultImports(source)) {
      for (const symbol of collectTsSymbols(source, localName)) {
        usedSymbols.add(symbol);
      }
    }
  }

  const expectedSymbols = new Set(allowlist.allowedTsSymbols);
  const unexpectedSymbols = [...usedSymbols].filter((symbol) => !expectedSymbols.has(symbol));
  const staleSymbols = allowlist.allowedTsSymbols.filter((symbol) => !usedSymbols.has(symbol));

  if (unexpectedSymbols.length > 0) {
    diagnostics.push(`unexpected ts-facade symbols: ${unexpectedSymbols.join(", ")}`);
  }
  if (staleSymbols.length > 0) {
    diagnostics.push(`stale ts-facade allowlist symbols: ${staleSymbols.join(", ")}`);
  }

  if (diagnostics.length > 0) {
    console.error("TS7 TypeScript API surface lock failed:");
    for (const diagnostic of diagnostics) {
      console.error(`- ${diagnostic}`);
    }
    process.exit(1);
  }

  console.log(`ts7 ts-api surface lock: ok (${allowlist.allowedTsSymbols.length} ts.* symbols)`);
}

function readAllowlist(): TsApiSurfaceAllowlist {
  const parsed = JSON.parse(readFileSync(allowlistPath, "utf8")) as TsApiSurfaceAllowlist;
  if (!Array.isArray(parsed.allowedTsSymbols)) {
    throw new Error("ts7 ts-api surface allowlist must contain allowedTsSymbols");
  }
  return {
    allowedTsSymbols: [...parsed.allowedTsSymbols].toSorted(),
  };
}

function collectTsFiles(root: string): string[] {
  if (!existsSync(root)) return [];
  const entries = readdirSync(root, { withFileTypes: true });
  const files: string[] = [];

  for (const entry of entries) {
    if (ignoredSegments.has(entry.name)) continue;
    const filePath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectTsFiles(filePath));
      continue;
    }
    if (entry.isFile() && filePath.endsWith(".ts")) {
      files.push(filePath);
    }
  }

  return files;
}

function collectFacadeDefaultImports(source: string): readonly string[] {
  const imports: string[] = [];
  for (const match of source.matchAll(facadeImportPattern)) {
    imports.push(match[1]!);
  }
  return imports;
}

function collectTsSymbols(source: string, localName: string): readonly string[] {
  const symbols = new Set<string>();
  const pattern = new RegExp(`\\b${escapeRegExp(localName)}\\.([A-Za-z_$][\\w$]*)`, "g");
  for (const match of source.matchAll(pattern)) {
    symbols.add(match[1]!);
  }
  return [...symbols].toSorted();
}

function isEngineCoreSource(filePath: string): boolean {
  const relativePath = path.relative(repoRoot, filePath);
  return relativePath.startsWith("server/engine-core-ts/src/");
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

main();
