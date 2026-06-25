const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const { pathToFileURL } = require("node:url");
const { SourceMapGenerator } = require("source-map-js");

const DEFAULT_INCLUDE = /\.module\.(css|scss)$/;
const REPO_ROOT = path.resolve(__dirname, "../..");
const ABSENT_JSON_ARGUMENT = "";

const MINIFY_PASS_IDS = [
  "comment-strip",
  "whitespace-strip",
  "number-compression",
  "color-compression",
  "shorthand-combining",
  "rule-deduplication",
  "rule-merging",
  "selector-merging",
  "empty-rule-removal",
  "calc-reduction",
  "print-css",
];

const TREE_SHAKE_PASS_IDS = [
  "tree-shake-class",
  "tree-shake-keyframes",
  "tree-shake-value",
  "tree-shake-custom-property",
];

function createOmenaBuildState(options = {}, overrides = {}) {
  return {
    root: options.cwd ?? process.cwd(),
    command: overrides.command ?? "build",
    cache: new Map(),
    generations: new Map(),
    configPromise: null,
    enginePromise: null,
  };
}

async function rebuildAndCache(filePath, source, options, state) {
  const generation = (state.generations.get(filePath) ?? 0) + 1;
  state.generations.set(filePath, generation);
  const cacheKey = buildCacheKey(filePath, source, options);
  const cached = state.cache.get(filePath);
  if (cached?.cacheKey === cacheKey) return cached.output;

  const output = await runOmenaBuild(filePath, source, options, state);
  if (state.generations.get(filePath) === generation) {
    state.cache.set(filePath, {
      cacheKey,
      output,
      summary: output.summary,
      updatedAt: Date.now(),
    });
  }
  return output;
}

async function runOmenaBuild(filePath, source, options, state) {
  const engine = await resolveOmenaEngine(options, state);
  const sources = collectStyleSources(filePath, source, options);
  const packageManifests = collectPackageManifests(options);
  const passIds = await resolvePassIds(options, engine, sources);
  const context = normalizeContext(options);
  const targetOptions = options.targetOptions ?? null;
  const includeSourceMap = options.sourceMap !== false;

  const summary = options.targetQuery
    ? await engine.buildSourcesForTargetQuery({
        targetPath: filePath,
        sources,
        targetQuery: options.targetQuery,
        targetOptions,
        context,
        packageManifests,
      })
    : options.bundle && typeof engine.buildBundleSources === "function"
      ? await engine.buildBundleSources({
          targetPath: filePath,
          sources,
          passIds,
          context,
          packageManifests,
          bundleEntryStylePaths: [],
        })
      : await engine.buildSources({
          targetPath: filePath,
          sources,
          passIds,
          context,
          packageManifests,
        });

  const outputCss = summary.outputCss ?? summary.execution?.outputCss;
  if (typeof outputCss !== "string") {
    throw new Error("[omena-css] invalid omena build summary: missing execution.outputCss");
  }

  return {
    code: outputCss,
    map: includeSourceMap
      ? (summary.sourceMapV3 ?? fallbackSourceMap(filePath, source, summary))
      : null,
    summary,
  };
}

function collectStyleSources(filePath, source, options) {
  const targetPath = path.resolve(filePath);
  const sources = [{ stylePath: targetPath, styleSource: source }];
  const seen = new Set([targetPath]);
  for (const sourcePath of options.sources ?? []) {
    const absolutePath = path.resolve(options.cwd ?? process.cwd(), sourcePath);
    if (seen.has(absolutePath)) continue;
    seen.add(absolutePath);
    sources.push({
      stylePath: absolutePath,
      styleSource: fs.readFileSync(absolutePath, "utf8"),
    });
  }
  return sources;
}

function collectPackageManifests(options) {
  return (options.packageManifests ?? []).map((manifestPath) => {
    const absolutePath = path.resolve(options.cwd ?? process.cwd(), manifestPath);
    return {
      packageJsonPath: absolutePath,
      packageJsonSource: fs.readFileSync(absolutePath, "utf8"),
    };
  });
}

async function resolvePassIds(options, engine, sources) {
  const passIds = [...(options.passes ?? [])];
  if (options.treeShake) appendPassIds(passIds, TREE_SHAKE_PASS_IDS);
  if (options.bundle) await appendBundlePassIds(passIds, engine, sources);
  if (options.minify) appendPassIds(passIds, MINIFY_PASS_IDS);
  return passIds;
}

async function appendBundlePassIds(passIds, engine, sources) {
  if (typeof engine.summarizeBundleSource !== "function") {
    throw new Error("[omena-css] loaded engine is missing summarizeTransformBundleFromSource.");
  }
  for (const source of sources) {
    const summary = await engine.summarizeBundleSource(source);
    appendPassIds(passIds, summary.plannedPassIds ?? summary.planned_pass_ids ?? []);
  }
}

function appendPassIds(passIds, preset) {
  for (const passId of preset) {
    if (!passIds.includes(passId)) passIds.push(passId);
  }
}

function normalizeContext(options) {
  const context = { ...options.context };
  if (options.closedStyleWorld || options.treeShake) {
    context.closedStyleWorld = true;
  }
  return Object.keys(context).length > 0 ? context : null;
}

async function resolveEffectiveOptions(options, state) {
  if (!state.configPromise) {
    state.configPromise = loadOmenaConfig(options, state);
  }
  const configOptions = await state.configPromise;
  return mergeOptions(configOptions, options, state);
}

function mergeOptions(configOptions, explicitOptions, state) {
  const merged = { ...configOptions, ...explicitOptions };
  merged.cwd = explicitOptions.cwd ?? configOptions.cwd ?? state.root;
  return merged;
}

async function loadOmenaConfig(options, state) {
  if (options.configFile === false) return {};
  const configPath =
    typeof options.configFile === "string"
      ? path.resolve(options.cwd ?? state.root, options.configFile)
      : findOmenaConfig(options.cwd ?? state.root);
  if (!configPath) return {};

  if (configPath.endsWith(".json")) {
    return JSON.parse(await fs.promises.readFile(configPath, "utf8"));
  }
  if (configPath.endsWith(".toml")) {
    return parseFlatToml(await fs.promises.readFile(configPath, "utf8"));
  }
  if (configPath.endsWith(".cjs")) {
    return normalizeConfigExport(require(configPath));
  }
  if (configPath.endsWith(".js") || configPath.endsWith(".mjs")) {
    return normalizeConfigExport(await import(pathToFileURL(configPath).href));
  }
  if (configPath.endsWith(".ts")) {
    return loadTypeScriptConfigWithVite(configPath, options.cwd ?? state.root);
  }
  return {};
}

function findOmenaConfig(root) {
  for (const fileName of [
    "omena.config.ts",
    "omena.config.mjs",
    "omena.config.js",
    "omena.config.cjs",
    "omena.config.json",
    "omena.config.toml",
  ]) {
    const candidate = path.join(root, fileName);
    if (fs.existsSync(candidate)) return candidate;
  }
  return null;
}

async function loadTypeScriptConfigWithVite(configPath, root) {
  let vite;
  try {
    vite = await import("vite");
  } catch {
    throw new Error(
      `[omena-css] ${path.basename(configPath)} requires vite to load TypeScript config; use JS/JSON/TOML config or install vite.`,
    );
  }
  const result = await vite.loadConfigFromFile(
    { command: "build", mode: process.env.NODE_ENV ?? "development" },
    configPath,
    root,
  );
  return normalizeConfigExport(result?.config ?? {});
}

function normalizeConfigExport(value) {
  const config = value?.default ?? value;
  if (typeof config === "function") return config();
  return config && typeof config === "object" ? config : {};
}

function parseFlatToml(source) {
  const config = {};
  for (const rawLine of source.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#") || line.startsWith("[")) continue;
    const match = /^([A-Za-z0-9_-]+)\s*=\s*(.+)$/.exec(line);
    if (!match) continue;
    const key = match[1].replace(/-([a-z])/g, (_, char) => char.toUpperCase());
    config[key] = parseTomlValue(match[2].trim());
  }
  return config;
}

function parseTomlValue(value) {
  if (value === "true") return true;
  if (value === "false") return false;
  if (value.startsWith("[") && value.endsWith("]")) {
    const inner = value.slice(1, -1).trim();
    if (!inner) return [];
    return inner.split(",").map((entry) => parseTomlValue(entry.trim()));
  }
  if (
    (value.startsWith('"') && value.endsWith('"')) ||
    (value.startsWith("'") && value.endsWith("'"))
  ) {
    return value.slice(1, -1);
  }
  return value;
}

async function resolveOmenaEngine(options, state) {
  if (options.engine) return normalizeEngine(options.engine, "injected");
  if (!state.enginePromise) {
    state.enginePromise = loadOmenaEngine(options);
  }
  return state.enginePromise;
}

async function loadOmenaEngine(options) {
  const napiBinding = loadOptionalCjs("@omena/napi") ?? loadOptionalCjs(localNapiPackagePath());
  if (napiBinding) return normalizeEngine(napiBinding, "napi");

  if (options.wasmFallback !== false) {
    const wasmBinding =
      (await loadOptionalEsm("@omena/wasm")) ?? (await loadOptionalEsm(localWasmPackagePath()));
    if (wasmBinding) return normalizeEngine(wasmBinding, "wasm");
  }

  throw new Error(
    "[omena-css] Unable to load @omena/napi or @omena/wasm. Install @omena/napi or enable the wasm package; CLI/cargo fallback is intentionally not used on plugin hot paths.",
  );
}

function normalizeEngine(binding, kind) {
  if (typeof binding.buildStyleSourcesWithContextJson === "function") {
    return {
      kind,
      async buildSources(input) {
        return JSON.parse(
          await binding.buildStyleSourcesWithContextJson(
            input.targetPath,
            JSON.stringify(input.sources),
            input.passIds,
            stringifyOptionalJson(input.context),
            stringifyOptionalJson(input.packageManifests),
          ),
        );
      },
      ...(typeof binding.bundleStyleSourcesWithContextJson === "function"
        ? {
            async buildBundleSources(input) {
              return JSON.parse(
                await binding.bundleStyleSourcesWithContextJson(
                  input.targetPath,
                  JSON.stringify(input.sources),
                  input.passIds,
                  stringifyOptionalJson(input.context),
                  stringifyOptionalJson(input.packageManifests),
                  input.bundleEntryStylePaths ?? [],
                ),
              );
            },
          }
        : {}),
      async buildSourcesForTargetQuery(input) {
        return JSON.parse(
          await binding.buildStyleSourcesForTargetQueryWithContextJson(
            input.targetPath,
            JSON.stringify(input.sources),
            input.targetQuery,
            stringifyOptionalJson(input.targetOptions),
            stringifyOptionalJson(input.context),
            stringifyOptionalJson(input.packageManifests),
          ),
        );
      },
      ...(typeof binding.summarizeTransformBundleFromSourceJson === "function"
        ? {
            async summarizeBundleSource(input) {
              return JSON.parse(
                await binding.summarizeTransformBundleFromSourceJson(
                  input.styleSource,
                  input.stylePath,
                ),
              );
            },
          }
        : {}),
    };
  }

  if (typeof binding.buildStyleSourcesWithContext === "function") {
    return {
      kind,
      async buildSources(input) {
        return binding.buildStyleSourcesWithContext(
          input.targetPath,
          input.sources,
          input.passIds,
          input.context,
          input.packageManifests,
        );
      },
      ...(typeof binding.bundleStyleSourcesWithContext === "function"
        ? {
            async buildBundleSources(input) {
              return binding.bundleStyleSourcesWithContext(
                input.targetPath,
                input.sources,
                input.passIds,
                input.context,
                input.packageManifests,
                input.bundleEntryStylePaths ?? [],
              );
            },
          }
        : {}),
      async buildSourcesForTargetQuery(input) {
        return binding.buildStyleSourcesForTargetQueryWithContext(
          input.targetPath,
          input.sources,
          input.targetQuery,
          input.targetOptions,
          input.context,
          input.packageManifests,
        );
      },
      ...(typeof binding.summarizeTransformBundleFromSource === "function"
        ? {
            async summarizeBundleSource(input) {
              return binding.summarizeTransformBundleFromSource(input.styleSource, input.stylePath);
            },
          }
        : {}),
    };
  }

  throw new Error("[omena-css] loaded engine is missing buildStyleSourcesWithContext.");
}

function stringifyOptionalJson(value) {
  if (value == null) return ABSENT_JSON_ARGUMENT;
  return JSON.stringify(value);
}

function loadOptionalCjs(specifier) {
  try {
    return require(specifier);
  } catch {
    return null;
  }
}

async function loadOptionalEsm(specifier) {
  try {
    return await import(specifier);
  } catch {
    return null;
  }
}

function localNapiPackagePath() {
  return path.join(REPO_ROOT, "rust/crates/omena-napi/pkg");
}

function localWasmPackagePath() {
  return pathToFileURL(path.join(REPO_ROOT, "rust/crates/omena-wasm/pkg/omena_wasm.js")).href;
}

function buildCacheKey(filePath, source, options) {
  const hash = crypto.createHash("sha256");
  hash.update(filePath);
  hash.update("\0");
  hash.update(source);
  hash.update("\0");
  hash.update(
    JSON.stringify({
      passes: options.passes ?? [],
      minify: Boolean(options.minify),
      targetQuery: options.targetQuery ?? null,
      targetOptions: options.targetOptions ?? null,
      context: options.context ?? null,
      sources: options.sources ?? [],
      packageManifests: options.packageManifests ?? [],
      sourceMap: options.sourceMap !== false,
      treeShake: Boolean(options.treeShake),
      bundle: Boolean(options.bundle),
      closedStyleWorld: Boolean(options.closedStyleWorld),
      devRuntime: Boolean(options.devRuntime),
    }),
  );
  return hash.digest("hex");
}

function normalizeFilePath(filePath) {
  try {
    return fs.realpathSync.native(filePath);
  } catch {
    return path.resolve(filePath);
  }
}

function extractCssModuleClassMap(css) {
  const classMap = {};
  const pattern = /(^|[^\\\w-])\.(-?[_a-zA-Z][_\w-]*)/g;
  for (const match of css.matchAll(pattern)) {
    const className = match[2];
    classMap[className] = className;
  }
  return classMap;
}

function fallbackSourceMap(filePath, source, summary) {
  const generator = new SourceMapGenerator({
    file: path.basename(filePath),
  });
  generator.setSourceContent(filePath, source);
  const lines = source.split(/\r?\n/);
  for (let index = 0; index < lines.length; index += 1) {
    generator.addMapping({
      generated: { line: index + 1, column: 0 },
      original: { line: index + 1, column: 0 },
      source: filePath,
    });
  }
  const generated = generator.toJSON();
  return {
    version: 3,
    sources: generated.sources ?? [filePath],
    sourcesContent: generated.sourcesContent ?? [source],
    names: generated.names ?? [],
    mappings: generated.mappings ?? "",
    x_omenaFallbackReason: "native build summary omitted sourceMapV3",
    x_omenaPassIds: summary.execution?.executedPassIds ?? [],
  };
}

function summarizeCache(cache) {
  return [...cache.entries()].map(([filePath, entry]) => ({
    filePath,
    updatedAt: entry.updatedAt,
    outputBytes: Buffer.byteLength(entry.output.code),
    sourceMapSources: entry.output.map?.sources ?? [],
    readySurfaces: entry.summary?.readySurfaces ?? [],
  }));
}

function matchesInclude(id, include) {
  if (include instanceof RegExp) return include.test(id);
  if (typeof include === "function") return Boolean(include(id));
  if (Array.isArray(include)) return include.some((entry) => matchesInclude(id, entry));
  if (typeof include === "string") return id.endsWith(include);
  return false;
}

module.exports = {
  DEFAULT_INCLUDE,
  MINIFY_PASS_IDS,
  TREE_SHAKE_PASS_IDS,
  createOmenaBuildState,
  extractCssModuleClassMap,
  matchesInclude,
  normalizeFilePath,
  rebuildAndCache,
  resolveEffectiveOptions,
  runOmenaBuild,
  summarizeCache,
};
