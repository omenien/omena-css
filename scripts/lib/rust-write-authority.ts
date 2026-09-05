import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

export function compareCodePoint(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

/** Minimum capability set sealed when this authority was born. It may only grow. */
export const FS_CAPABILITY_BIRTH_SET = [
  "std::fs::copy",
  "std::fs::create_dir",
  "std::fs::create_dir_all",
  "std::fs::hard_link",
  "std::fs::remove_dir",
  "std::fs::remove_dir_all",
  "std::fs::remove_file",
  "std::fs::rename",
  "std::fs::set_permissions",
  "std::fs::soft_link",
  "std::fs::write",
  "std::fs::DirBuilder::create",
  "std::fs::File::create",
  "std::fs::File::create_new",
  "std::fs::File::set_len",
  "std::fs::File::set_modified",
  "std::fs::File::set_permissions",
  "std::fs::File::set_times",
  "std::fs::OpenOptions::open",
  "std::os::unix::fs::chown",
  "std::os::unix::fs::chroot",
  "std::os::unix::fs::fchown",
  "std::os::unix::fs::lchown",
  "std::os::unix::fs::symlink",
  "std::os::windows::fs::symlink_dir",
  "std::os::windows::fs::symlink_file",
] as const;

export interface ClippyDisallowedMethod {
  readonly path: string;
  readonly allowInvalid: boolean;
}

export function readClippyDisallowedMethods(repoRoot: string): ClippyDisallowedMethod[] {
  const source = readFileSync(path.join(repoRoot, "rust/clippy.toml"), "utf8");
  const header = source.match(/\bdisallowed-methods\s*=\s*\[/u);
  assert.ok(header?.index !== undefined, "rust/clippy.toml lacks disallowed-methods");
  const open = source.indexOf("[", header.index);
  const close = matchingRustDelimiter(source, open, "[", "]");
  const rows = source
    .slice(open + 1, close)
    .split("\n")
    .filter((line) => /\bpath\s*=\s*"/u.test(line))
    .map((line) => {
      const methodPath = line.match(/\bpath\s*=\s*"([^"]+)"/u)?.[1];
      assert.ok(methodPath, `disallowed method path is not resolvable: ${line}`);
      return {
        path: methodPath,
        allowInvalid: /\ballow-invalid\s*=\s*true\b/u.test(line),
      };
    });
  const duplicates = rows
    .map(({ path: methodPath }) => methodPath)
    .filter((methodPath, index, all) => all.indexOf(methodPath) !== index);
  assert.deepEqual(duplicates, [], `duplicate disallowed method paths: ${duplicates.join(", ")}`);
  return rows;
}

export interface CargoTarget {
  readonly name: string;
  readonly kind: readonly string[];
  readonly crate_types: readonly string[];
  readonly src_path: string;
}

export interface CargoPackage {
  readonly name: string;
  readonly manifest_path: string;
  readonly publish: readonly string[] | null;
  readonly targets: readonly CargoTarget[];
  readonly features: Readonly<Record<string, readonly string[]>>;
}

export interface WriteScope {
  readonly members: readonly string[];
  readonly scope: readonly string[];
  readonly refusals: readonly string[];
  readonly deliveryOnly: readonly string[];
  readonly libTargetCount: number;
  readonly binTargetCount: number;
}

export interface WriteAuthorityArgv {
  readonly cargoArgs: readonly string[];
  readonly scope: WriteScope;
}

export function isCargoLibraryTarget(target: CargoTarget): boolean {
  return target.kind.some((kind) =>
    new Set(["lib", "proc-macro", "cdylib", "rlib", "staticlib", "dylib"]).has(kind),
  );
}

export interface CargoMetadata {
  readonly packages: readonly CargoPackage[];
}

export function readCargoMetadata(repoRoot: string): CargoMetadata {
  return JSON.parse(
    execFileSync(
      "cargo",
      ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
      { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
    ),
  ) as CargoMetadata;
}

function isPublishable(pkg: CargoPackage): boolean {
  return !(Array.isArray(pkg.publish) && pkg.publish.length === 0);
}

function shipsProductBytes(pkg: CargoPackage): boolean {
  return pkg.targets.some((target) => target.crate_types.includes("cdylib"));
}

/**
 * The write authority covers every publishable workspace member and every
 * non-publishable member that emits a delivery artifact. The refusal complement
 * is computed from cargo metadata; callers never maintain a crate-name list.
 */
export function deriveWriteScope(repoRoot: string): WriteScope {
  const metadata = readCargoMetadata(repoRoot);
  const members = metadata.packages.map(({ name }) => name).toSorted();
  const deliveryOnly = metadata.packages
    .filter((pkg) => !isPublishable(pkg) && shipsProductBytes(pkg))
    .map(({ name }) => name)
    .toSorted();
  const scope = metadata.packages
    .filter((pkg) => isPublishable(pkg) || shipsProductBytes(pkg))
    .map(({ name }) => name)
    .toSorted();
  const scopeSet = new Set(scope);
  const refusals = members.filter((name) => !scopeSet.has(name));

  assert.deepEqual(
    [...scope, ...refusals].toSorted(),
    members,
    "write scope and refusal complement must partition cargo workspace members",
  );
  assert.equal(
    new Set([...scope, ...refusals]).size,
    members.length,
    "write scope and refusal complement must be disjoint",
  );
  for (const pkg of metadata.packages) {
    assert.ok(
      isPublishable(pkg) || shipsProductBytes(pkg) || refusals.includes(pkg.name),
      `unclassified crate ${pkg.name}`,
    );
  }

  const selected = metadata.packages.filter(({ name }) => scopeSet.has(name));
  return {
    members,
    scope,
    refusals,
    deliveryOnly,
    libTargetCount: selected.filter((pkg) => pkg.targets.some(isCargoLibraryTarget)).length,
    binTargetCount: selected.reduce(
      (count, pkg) => count + pkg.targets.filter((target) => target.kind.includes("bin")).length,
      0,
    ),
  };
}

function baseCargoArgs(scope: WriteScope): string[] {
  return [
    "clippy",
    "--manifest-path",
    "rust/Cargo.toml",
    "--workspace",
    ...scope.refusals.flatMap((crate) => ["--exclude", crate]),
    "--all-features",
    "--lib",
    "--bins",
  ];
}

export function banGateArgv(repoRoot: string): WriteAuthorityArgv {
  const scope = deriveWriteScope(repoRoot);
  return {
    cargoArgs: [...baseCargoArgs(scope), "--", "-D", "clippy::disallowed_methods"],
    scope,
  };
}

export function forceWarnCensusArgv(repoRoot: string): WriteAuthorityArgv {
  const scope = deriveWriteScope(repoRoot);
  return {
    cargoArgs: [
      ...baseCargoArgs(scope),
      "--message-format=json",
      "--",
      "--force-warn",
      "clippy::disallowed_methods",
    ],
    scope,
  };
}

export function manifestPackageBySourceFile(
  repoRoot: string,
  absoluteFile: string,
): CargoPackage | undefined {
  const metadata = readCargoMetadata(repoRoot);
  const normalized = path.resolve(absoluteFile);
  return metadata.packages
    .filter((pkg) => normalized.startsWith(`${path.dirname(pkg.manifest_path)}${path.sep}`))
    .toSorted((left, right) => right.manifest_path.length - left.manifest_path.length)[0];
}

export interface RustPublicItem {
  readonly path: string;
  readonly sourceFile: string;
  readonly line: number;
  readonly stability: "stable" | "unstable";
}

interface ItemContext {
  readonly path: string;
  readonly stability: "stable" | "unstable" | undefined;
  readonly publicTrait: boolean;
  readonly inherentImpl: boolean;
}

function lineNumber(source: string, offset: number): number {
  return source.slice(0, offset).split("\n").length;
}

export function maskRustCommentsAndLiterals(source: string, preserveLiterals = false): string {
  const output = source.split("");
  const blank = (start: number, end: number): void => {
    for (let index = start; index < end; index += 1) {
      if (output[index] !== "\n" && output[index] !== "\r") output[index] = " ";
    }
  };
  let cursor = 0;
  while (cursor < source.length) {
    if (source.startsWith("//", cursor)) {
      const end = source.indexOf("\n", cursor + 2);
      const stop = end < 0 ? source.length : end;
      blank(cursor, stop);
      cursor = stop;
      continue;
    }
    if (source.startsWith("/*", cursor)) {
      const start = cursor;
      let depth = 1;
      cursor += 2;
      while (cursor < source.length && depth > 0) {
        if (source.startsWith("/*", cursor)) {
          depth += 1;
          cursor += 2;
        } else if (source.startsWith("*/", cursor)) {
          depth -= 1;
          cursor += 2;
        } else {
          cursor += 1;
        }
      }
      blank(start, cursor);
      continue;
    }
    const raw = source.slice(cursor).match(/^(?:br|r)(#*)"/u);
    if (raw) {
      const start = cursor;
      const terminator = `"${raw[1] ?? ""}`;
      cursor += raw[0].length;
      const end = source.indexOf(terminator, cursor);
      cursor = end < 0 ? source.length : end + terminator.length;
      if (!preserveLiterals) blank(start, cursor);
      continue;
    }
    if (source[cursor] === '"') {
      const start = cursor;
      cursor += 1;
      let escaped = false;
      while (cursor < source.length) {
        const current = source[cursor++]!;
        if (escaped) escaped = false;
        else if (current === "\\") escaped = true;
        else if (current === '"') break;
      }
      if (!preserveLiterals) blank(start, cursor);
      continue;
    }
    const character = source
      .slice(cursor)
      .match(/^'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f_]{1,6}\}|.)|[^'\\\r\n])'/u);
    if (character) {
      const start = cursor;
      cursor += character[0].length;
      if (!preserveLiterals) blank(start, cursor);
      continue;
    }
    cursor += 1;
  }
  return output.join("");
}

export function matchingRustDelimiter(
  source: string,
  open: number,
  left: string,
  right: string,
): number {
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === left) depth += 1;
    else if (source[index] === right) {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  assert.fail(`unterminated ${left}${right} region at byte ${open}`);
}

function stabilityFromAttributes(attributes: readonly string[]): "stable" | "unstable" | undefined {
  if (attributes.some((attribute) => /#!?\s*\[\s*unstable\b/u.test(attribute))) {
    return "unstable";
  }
  if (attributes.some((attribute) => /#!?\s*\[\s*stable\b/u.test(attribute))) {
    return "stable";
  }
  return undefined;
}

function declarationEnd(source: string, start: number, limit: number): number {
  let round = 0;
  let square = 0;
  let angle = 0;
  for (let cursor = start; cursor < limit; cursor += 1) {
    const current = source[cursor]!;
    if (current === "(") round += 1;
    else if (current === ")") round -= 1;
    else if (current === "[") square += 1;
    else if (current === "]") square -= 1;
    else if (current === "<") angle += 1;
    else if (current === ">") angle = Math.max(0, angle - 1);
    else if ((current === "{" || current === ";") && round === 0 && square === 0 && angle === 0) {
      return cursor;
    }
  }
  return limit;
}

function normalizeImplTarget(header: string): string | undefined {
  const withoutImpl = header.replace(/^\s*(?:unsafe\s+)?impl\b/u, "").trim();
  let rest = withoutImpl;
  if (rest.startsWith("<")) {
    const close = matchingRustDelimiter(rest, 0, "<", ">");
    rest = rest.slice(close + 1).trim();
  }
  const beforeWhere = rest.split(/\s+where\b/u)[0]!.trim();
  if (/\s+for\s+/u.test(beforeWhere)) return undefined;
  const name = beforeWhere.match(/(?:^|::)([A-Z][A-Za-z0-9_]*)\s*(?:<|$)/u)?.[1];
  return name;
}

/**
 * Enumerate source-declared public items while applying the stability inheritance
 * used by rustc. Public trait members are included even though Rust omits `pub`
 * from their declarations; trait-impl methods are not separate public items.
 */
export function enumerateRustPublicItems(
  absoluteFile: string,
  publicModulePath: string,
): RustPublicItem[] {
  const source = readFileSync(absoluteFile, "utf8");
  const code = maskRustCommentsAndLiterals(source);
  const fileAttributes = [...source.matchAll(/^\s*#!\s*\[(?:stable|unstable)\b[^\]]*\]/gmu)].map(
    ([attribute]) => attribute,
  );
  const fileStability = stabilityFromAttributes(fileAttributes);
  assert.ok(fileStability, `rust-src file lacks a stability root: ${absoluteFile}`);
  const items: RustPublicItem[] = [];

  const parseBlock = (start: number, end: number, context: ItemContext): void => {
    let cursor = start;
    let pendingAttributes: string[] = [];
    while (cursor < end) {
      while (/\s/u.test(code[cursor] ?? "")) cursor += 1;
      if (cursor >= end) break;
      if (code[cursor] === "#" && code[cursor + 1] === "[") {
        const close = matchingRustDelimiter(code, cursor + 1, "[", "]");
        pendingAttributes.push(source.slice(cursor, close + 1));
        cursor = close + 1;
        continue;
      }

      const remaining = code.slice(cursor, end);
      const impl = remaining.match(/^(?:unsafe\s+)?impl\b/u);
      if (impl) {
        const terminator = declarationEnd(code, cursor, end);
        assert.equal(code[terminator], "{", `impl lacks a body in ${absoluteFile}`);
        const close = matchingRustDelimiter(code, terminator, "{", "}");
        const target = normalizeImplTarget(code.slice(cursor, terminator));
        if (target) {
          parseBlock(terminator + 1, close, {
            path: `${publicModulePath}::${target}`,
            stability: stabilityFromAttributes(pendingAttributes) ?? context.stability,
            publicTrait: false,
            inherentImpl: true,
          });
        }
        pendingAttributes = [];
        cursor = close + 1;
        continue;
      }

      const functionDeclaration = remaining.match(
        /^(pub(?:\s*\([^)]*\))?\s+)?(?:(?:const|async|unsafe)\s+)*(?:extern\s+"[^"]+"\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\b/u,
      );
      const namedDeclaration = remaining.match(
        /^(pub(?:\s*\([^)]*\))?\s+)?(?:unsafe\s+)?(struct|enum|union|trait|type|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)\b/u,
      );
      const declaration = functionDeclaration ?? namedDeclaration;
      if (declaration) {
        const explicitPublic = declaration[1]?.trim() === "pub";
        const publicItem = explicitPublic || context.publicTrait;
        const kind = functionDeclaration ? "fn" : namedDeclaration![2]!;
        const name = functionDeclaration ? functionDeclaration[2]! : namedDeclaration![3]!;
        const stability = stabilityFromAttributes(pendingAttributes) ?? context.stability;
        const terminator = declarationEnd(code, cursor, end);
        if (publicItem) {
          assert.ok(
            stability,
            `public rust-src item lacks inherited stability: ${publicModulePath}::${name}`,
          );
          items.push({
            path: `${context.path}::${name}`,
            sourceFile: absoluteFile,
            line: lineNumber(source, cursor),
            stability,
          });
        }
        if (code[terminator] === "{") {
          const close = matchingRustDelimiter(code, terminator, "{", "}");
          if (kind === "trait" && publicItem) {
            parseBlock(terminator + 1, close, {
              path: `${context.path}::${name}`,
              stability,
              publicTrait: true,
              inherentImpl: false,
            });
          }
          cursor = close + 1;
        } else {
          cursor = Math.min(terminator + 1, end);
        }
        pendingAttributes = [];
        continue;
      }

      // Unknown root-level constructs (imports, private helpers, macros) are not
      // public API. Skip their complete statement/body so nested local items cannot
      // be mistaken for module items.
      const terminator = declarationEnd(code, cursor, end);
      pendingAttributes = [];
      if (terminator >= end) break;
      if (code[terminator] === "{") {
        cursor = matchingRustDelimiter(code, terminator, "{", "}") + 1;
      } else {
        cursor = terminator + 1;
      }
    }
  };

  parseBlock(0, source.length, {
    path: publicModulePath,
    stability: fileStability,
    publicTrait: false,
    inherentImpl: false,
  });
  const duplicatePaths = items
    .map(({ path: itemPath }) => itemPath)
    .filter((itemPath, index, all) => all.indexOf(itemPath) !== index);
  assert.deepEqual(
    duplicatePaths,
    [],
    `duplicate rust-src public paths: ${duplicatePaths.join(", ")}`,
  );
  return items.toSorted((left, right) => compareCodePoint(left.path, right.path));
}

export interface RustNamedFunction {
  readonly name: string;
  readonly shortName: string;
  readonly start: number;
  readonly bodyStart: number;
  readonly end: number;
  readonly line: number;
}

/** Return every named function region, including inherent methods and nested helpers. */
export function rustNamedFunctions(source: string, modulePath: string): RustNamedFunction[] {
  const code = maskRustCommentsAndLiterals(source);
  const implementations: Array<{ label: string; start: number; end: number }> = [];
  for (const match of code.matchAll(/\bimpl\b([^;{]*)\{/gu)) {
    const start = match.index ?? 0;
    const open = start + match[0].lastIndexOf("{");
    const target = normalizeImplTarget(code.slice(start, open));
    if (!target) continue;
    implementations.push({
      label: target,
      start,
      end: matchingRustDelimiter(code, open, "{", "}") + 1,
    });
  }
  const functions: RustNamedFunction[] = [];
  const declaration =
    /^[\t ]*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+([a-z][a-z0-9_]*)[^;{]*\{/gmu;
  for (const match of code.matchAll(declaration)) {
    const start = match.index ?? 0;
    const bodyStart = start + match[0].lastIndexOf("{");
    const end = matchingRustDelimiter(code, bodyStart, "{", "}") + 1;
    const implementation = implementations.findLast(
      (candidate) => candidate.start < start && end < candidate.end,
    );
    const shortName = match[1]!;
    functions.push({
      name: `${modulePath}::${implementation ? `${implementation.label}::` : ""}${shortName}`,
      shortName,
      start,
      bodyStart,
      end,
      line: lineNumber(source, start),
    });
  }
  return functions.toSorted((left, right) => left.start - right.start);
}
