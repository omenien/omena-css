const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const { pathToFileURL } = require("node:url");
const { SourceMapGenerator } = require("source-map-js");

const DEFAULT_INCLUDE = /\.module\.(css|scss|less)$/;
const CSS_MODULE_PATH = /\.module\.(css|scss|less)$/;
const REPO_ROOT = path.resolve(__dirname, "../..");
const ABSENT_JSON_ARGUMENT = "";

const MINIFY_PASS_IDS = Object.freeze(require("./semantic-minify-pass-ids.json"));

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

  const moduleInterface = CSS_MODULE_PATH.test(filePath)
    ? await resolveCssModuleInterface(engine, filePath, sources, packageManifests, state)
    : null;

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
    ...(moduleInterface
      ? {
          classMap: moduleInterface.classMap,
          namedExports: moduleInterface.namedExports,
          typescriptDeclaration: moduleInterface.typescriptDeclaration,
          moduleInterface,
        }
      : {}),
  };
}

async function resolveCssModuleInterface(engine, filePath, sources, packageManifests, state) {
  if (
    typeof engine.bundlerHostCapabilities !== "function" ||
    typeof engine.resolveCssModule !== "function"
  ) {
    throw new Error("[omena-css] loaded engine is missing the bundler host protocol.");
  }
  const capabilities = await engine.bundlerHostCapabilities();
  if (!capabilities.capabilities?.includes("semanticClassMap")) {
    throw new Error("[omena-css] loaded engine does not advertise semantic class maps.");
  }
  const response = await engine.resolveCssModule({
    snapshotId: { value: state.generations.get(filePath) ?? 0 },
    stylePath: path.resolve(filePath),
    styleSources: sources,
    packageManifests,
  });
  if (response.protocolVersion !== capabilities.protocolVersion) {
    throw new Error("[omena-css] bundler host protocol version mismatch.");
  }
  if (!response.ready) {
    const detail = (response.diagnostics ?? [])
      .map((diagnostic) => `${diagnostic.code}: ${diagnostic.message}`)
      .join("; ");
    throw new Error(`[omena-css] CSS Module interface is not ready${detail ? `: ${detail}` : "."}`);
  }
  return response;
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
  if (options.bundle || sources.some((source) => CSS_MODULE_PATH.test(source.stylePath))) {
    await appendBundlePassIds(passIds, engine, sources);
  }
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
    return selectBuildConfig(JSON.parse(await fs.promises.readFile(configPath, "utf8")));
  }
  if (configPath.endsWith(".toml")) {
    return selectBuildConfig(parseFlatToml(await fs.promises.readFile(configPath, "utf8")));
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
    "omena.toml",
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
  if (typeof config === "function") return selectBuildConfig(config());
  return selectBuildConfig(config);
}

function selectBuildConfig(config) {
  if (!config || typeof config !== "object") return {};
  return config.build && typeof config.build === "object" ? config.build : config;
}

function parseFlatToml(source) {
  const config = {};
  let current = config;
  for (const rawLine of source.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) continue;
    const sectionMatch = /^\[([A-Za-z0-9_.-]+)\]$/.exec(line);
    if (sectionMatch) {
      current = selectTomlSection(config, sectionMatch[1]);
      continue;
    }
    const match = /^([A-Za-z0-9_-]+)\s*=\s*(.+)$/.exec(line);
    if (!match) continue;
    const key = normalizeTomlKey(match[1]);
    current[key] = parseTomlValue(match[2].trim());
  }
  return config;
}

function selectTomlSection(config, rawSectionName) {
  const parts = rawSectionName.split(".").map(normalizeTomlKey);
  let current = config;
  for (const part of parts) {
    current[part] = current[part] && typeof current[part] === "object" ? current[part] : {};
    current = current[part];
  }
  return current;
}

function normalizeTomlKey(key) {
  return key.replace(/-([a-z])/g, (_, char) => char.toUpperCase());
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
  const localNapiBinding = loadOptionalCjs(localNapiPackagePath());
  if (localNapiBinding) return normalizeEngine(localNapiBinding, "napi");

  if (options.wasmFallback !== false) {
    const localWasmBinding = await loadOptionalEsm(localWasmPackagePath());
    if (localWasmBinding) return normalizeEngine(localWasmBinding, "wasm");
  }

  const installedNapiBinding = loadOptionalCjs("@omena/napi");
  if (installedNapiBinding) return normalizeEngine(installedNapiBinding, "napi");

  if (options.wasmFallback !== false) {
    const installedWasmBinding = await loadOptionalEsm("@omena/wasm");
    if (installedWasmBinding) return normalizeEngine(installedWasmBinding, "wasm");
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
      ...(typeof binding.bundlerHostCapabilitiesJson === "function" &&
      typeof binding.resolveCssModuleForBundlerHostJson === "function"
        ? {
            async bundlerHostCapabilities() {
              return JSON.parse(await binding.bundlerHostCapabilitiesJson());
            },
            async resolveCssModule(input) {
              return JSON.parse(
                await binding.resolveCssModuleForBundlerHostJson(JSON.stringify(input)),
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
      ...(typeof binding.bundlerHostCapabilities === "function" &&
      typeof binding.resolveCssModuleForBundlerHost === "function"
        ? {
            async bundlerHostCapabilities() {
              return binding.bundlerHostCapabilities();
            },
            async resolveCssModule(input) {
              return binding.resolveCssModuleForBundlerHost(input);
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
  matchesInclude,
  normalizeFilePath,
  rebuildAndCache,
  resolveEffectiveOptions,
  runOmenaBuild,
  summarizeCache,
};
