import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

/**
 * Editor-side mirror of the CLI's `read_external_sifs` / `omena.lock`
 * loading (`omena-cli/src/main.rs`). The engine wire
 * (`StyleDiagnosticsForFileInputV0`) carries `externalSifs` / `externalMode`;
 * this loader sources those SIFs from a workspace `omena.lock` plus the
 * cached SIF JSON artifacts it references, so the live LSP can leave
 * `Ignored` mode without a CLI invocation.
 *
 * Best-effort by design: a missing lockfile, an unreadable SIF artifact, or
 * malformed JSON yields an empty SIF set (i.e. today's `Ignored` behaviour),
 * never a thrown error on the diagnostics hot path.
 */

const OMENA_LOCK_BASENAME = "omena.lock";

/**
 * One external SIF forwarded to the engine wire. The `sif` payload is the
 * verbatim cached SIF JSON object; the Rust side deserialises it into
 * `OmenaSifV1`, so this layer treats it as opaque JSON.
 */
export interface QueryExternalSifInputV0 {
  readonly canonicalUrl: string;
  readonly sif: unknown;
}

interface OmenaLockSifEntryV1Json {
  readonly canonicalUrl?: unknown;
  readonly sifPath?: unknown;
}

interface OmenaLockV1Json {
  readonly entries?: unknown;
}

interface CachedSifV1Json {
  readonly canonicalUrl?: unknown;
}

export interface ExternalSifLoaderDeps {
  readonly fileExists?: (filePath: string) => boolean;
  readonly readFile?: (filePath: string) => string;
}

/**
 * Resolve a lock entry's `sifPath` the same way the CLI does
 * (`resolve_lock_relative_path`): absolute paths pass through; relative
 * paths join against the lockfile's directory.
 */
function resolveLockRelativePath(lockfilePath: string, entryPath: string): string {
  if (path.isAbsolute(entryPath)) return entryPath;
  return path.join(path.dirname(lockfilePath), entryPath);
}

/**
 * Load every external SIF declared by `<workspaceRoot>/omena.lock`,
 * mirroring the CLI's `--external sif --sif <file>` path. Returns an empty
 * array (today's `Ignored` behaviour) when the lockfile is absent or when no
 * usable SIF entries can be read.
 */
export function loadExternalSifsForWorkspace(
  workspaceRoot: string | undefined,
  deps: ExternalSifLoaderDeps = {},
): readonly QueryExternalSifInputV0[] {
  if (!workspaceRoot) return [];
  const fileExists = deps.fileExists ?? existsSync;
  const readFile = deps.readFile ?? ((filePath: string) => readFileSync(filePath, "utf8"));

  const lockfilePath = path.join(workspaceRoot, OMENA_LOCK_BASENAME);
  if (!fileExists(lockfilePath)) return [];

  const lock = parseJson<OmenaLockV1Json>(() => readFile(lockfilePath));
  if (!lock || !Array.isArray(lock.entries)) return [];

  const sifs: QueryExternalSifInputV0[] = [];
  for (const rawEntry of lock.entries) {
    const entry = rawEntry as OmenaLockSifEntryV1Json;
    if (typeof entry?.canonicalUrl !== "string" || typeof entry?.sifPath !== "string") continue;
    const sifPath = resolveLockRelativePath(lockfilePath, entry.sifPath);
    if (!fileExists(sifPath)) continue;
    const sif = parseJson<CachedSifV1Json>(() => readFile(sifPath));
    if (!sif || typeof sif !== "object") continue;
    sifs.push({ canonicalUrl: entry.canonicalUrl, sif });
  }
  return sifs;
}

function parseJson<T>(read: () => string): T | null {
  try {
    return JSON.parse(read()) as T;
  } catch {
    return null;
  }
}
