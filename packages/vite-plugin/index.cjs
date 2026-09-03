const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const { SourceMapConsumer } = require("source-map-js");
const {
  DEFAULT_INCLUDE,
  MINIFY_PASS_IDS,
  TREE_SHAKE_PASS_IDS,
  createOmenaBuildState,
  matchesInclude,
  normalizeFilePath,
  rebuildAndCache,
  resolveOmenaSourceContentDigest,
  resolveEffectiveOptions,
  summarizeCache,
} = require("@omena/css-build-adapter");

const VIRTUAL_MODULE_ID = "virtual:omena-css/build-summary";
const RESOLVED_VIRTUAL_MODULE_ID = `\0${VIRTUAL_MODULE_ID}`;
const DEV_RUNTIME_ID_PREFIX = "\0omena-vite-style:";
const DEV_RUNTIME_MARKER = "/* @omena/vite-plugin dev runtime */";
const SOURCE_ADMISSION_INSPECTION = Symbol.for("omena-css.vite.source-admission-inspection");

function omenaCss(options = {}) {
  const pluginName = options.name ?? "omena-css";
  const state = createOmenaBuildState(options);

  return {
    name: pluginName,
    enforce: options.enforce ?? "pre",
    [SOURCE_ADMISSION_INSPECTION]: {
      async inspect({ code, id, diskSource, upstreamMap }) {
        const fileId = cleanViteId(id);
        const effectiveOptions = await resolveEffectiveOptions(options, state);
        const include = effectiveOptions.include ?? DEFAULT_INCLUDE;
        if (!matchesInclude(fileId, include)) {
          return { included: false, provenanceClass: "not-included" };
        }
        if (diskSource === code) {
          return { included: true, provenanceClass: "disk-backed" };
        }
        const usableMap = usableUpstreamSourceMap(
          { getCombinedSourcemap: () => upstreamMap },
          fileId,
          diskSource,
        );
        return {
          included: true,
          provenanceClass: usableMap == null ? "virtual-only" : "virtual-with-map",
        };
      },
    },
    configResolved(config) {
      state.root = options.cwd ?? config.root ?? state.root;
      state.command = config.command ?? state.command;
    },
    configureServer(server) {
      state.server = server;
    },
    async resolveId(id, importer) {
      if (id === VIRTUAL_MODULE_ID) return RESOLVED_VIRTUAL_MODULE_ID;
      const effectiveOptions = await resolveEffectiveOptions(options, state);
      if (!shouldUseDevRuntime(effectiveOptions, state)) return null;
      const resolvedFile = resolveStyleImport(id, importer, effectiveOptions, state);
      if (!resolvedFile) return null;
      const include = effectiveOptions.include ?? DEFAULT_INCLUDE;
      if (!matchesInclude(resolvedFile, include) || !fs.existsSync(resolvedFile)) return null;
      return toDevRuntimeId(resolvedFile);
    },
    async load(id) {
      if (id === RESOLVED_VIRTUAL_MODULE_ID) {
        return `export default ${JSON.stringify(summarizeViteCache(state.cache))};\n`;
      }
      if (!isDevRuntimeId(id)) return null;

      const effectiveOptions = await resolveEffectiveOptions(options, state);
      if (!shouldUseDevRuntime(effectiveOptions, state)) return null;

      const fileId = fromDevRuntimeId(id);
      const source = await fs.promises.readFile(fileId, "utf8");
      const output = await rebuildViteSource(fileId, source, source, null, effectiveOptions, state);
      registerBuildDependencies(this, state, fileId);
      reportBundlerHostDiagnostics(this, output, pluginName);
      return {
        code: renderDevCssModule(fileId, output),
        map: output.map,
      };
    },
    async transform(code, id) {
      if (code.startsWith(DEV_RUNTIME_MARKER)) return null;

      const fileId = cleanViteId(id);
      const effectiveOptions = await resolveEffectiveOptions(options, state);
      const include = effectiveOptions.include ?? DEFAULT_INCLUDE;
      if (!matchesInclude(fileId, include)) return null;
      const diskSource = fs.existsSync(fileId) ? fs.readFileSync(fileId, "utf8") : null;
      if (effectiveOptions.requireDiskSource === true && diskSource !== code) {
        const reason =
          diskSource == null ? "has no corresponding disk source" : "differs from disk source";
        this.warn?.(
          `[${pluginName}] skipped ${fileId} in strict disk-source mode: transform input ${reason}.`,
        );
        return null;
      }

      const upstreamMap =
        diskSource !== code ? usableUpstreamSourceMap(this, fileId, diskSource) : null;
      const output = await rebuildViteSource(
        fileId,
        code,
        diskSource,
        upstreamMap,
        effectiveOptions,
        state,
      );
      registerBuildDependencies(this, state, fileId);
      reportBundlerHostDiagnostics(this, output, pluginName);
      if (output.code === code) return null;
      return {
        code: output.code,
        map: output.map,
      };
    },
    async handleHotUpdate(ctx) {
      const effectiveOptions = await resolveEffectiveOptions(options, state);
      const include = effectiveOptions.include ?? DEFAULT_INCLUDE;
      const changedPath = normalizeFilePath(ctx.file);
      const targetPaths = dependentBuildTargets(state.cache, changedPath);
      if (targetPaths.length === 0 && matchesInclude(changedPath, include)) {
        targetPaths.push(changedPath);
      }
      if (targetPaths.length === 0) return;

      const rebuiltTargets = await Promise.all(
        targetPaths.filter(fs.existsSync).map(async (targetPath) => {
          const source = await fs.promises.readFile(targetPath, "utf8");
          const previousOutput = state.cache.get(targetPath)?.output;
          const output = await rebuildViteSource(
            targetPath,
            source,
            source,
            null,
            effectiveOptions,
            state,
          );
          return { output, previousOutput, targetPath };
        }),
      );
      const invalidatedModules = [];
      for (const { output, previousOutput, targetPath } of rebuiltTargets) {
        registerBuildDependencies(this, state, targetPath);
        reportBundlerHostDiagnostics(this, output, pluginName);

        if (shouldUseDevRuntime(effectiveOptions, state)) {
          const classDecision = classifyCssModuleExportDelta(
            previousOutput?.classExports,
            output.classExports,
          );
          const valueDecision = classifyCssModuleExportDelta(
            previousOutput?.valueExports,
            output.valueExports,
          );
          const decision = combineCssModuleExportDelta(classDecision, valueDecision);
          const runtimeModule = ctx.server?.moduleGraph?.getModuleById?.(
            toDevRuntimeId(targetPath),
          );
          if (decision === "shapeChanged") {
            invalidatedModules.push(...collectAffectedRuntimeModules(runtimeModule));
          } else {
            ctx.server?.ws?.send?.({
              type: "custom",
              event: devRuntimeEventName(targetPath),
              data: devRuntimeUpdatePayload(targetPath, output, decision),
            });
          }
          continue;
        }

        const runtimeModule = ctx.server?.moduleGraph?.getModuleById?.(toDevRuntimeId(targetPath));
        const modules =
          targetPath === changedPath && ctx.modules?.length
            ? ctx.modules
            : [runtimeModule, ctx.server?.moduleGraph?.getModuleById?.(targetPath)].filter(Boolean);
        invalidatedModules.push(...modules);
      }

      const uniqueModules = [...new Set(invalidatedModules)];
      for (const mod of uniqueModules) {
        ctx.server?.moduleGraph?.invalidateModule?.(mod);
      }
      return uniqueModules;
    },
  };
}

async function rebuildViteSource(fileId, source, diskSource, upstreamMap, effectiveOptions, state) {
  const isDiskBacked = diskSource === source;
  const diskDigest = isDiskBacked
    ? null
    : diskSource == null
      ? null
      : await resolveOmenaSourceContentDigest(fileId, diskSource, effectiveOptions, state);
  const classification = isDiskBacked
    ? "disk-backed"
    : upstreamMap == null
      ? "virtual-only"
      : "virtual-with-map";
  const identityContext = isDiskBacked
    ? undefined
    : {
        sourceProvenance: {
          schemaVersion: "0",
          product: "omena-vite.virtual-source-provenance",
          classification,
          diskDigest,
          upstreamMapPresent: upstreamMap != null,
          upstreamMap,
        },
      };
  const output = await rebuildAndCache(fileId, source, effectiveOptions, state, identityContext);
  const entry = state.cache.get(fileId);
  if (entry?.output === output) {
    const inputDigest = entry.buildSnapshotIdentity?.targetSourceDigest;
    const hasInputDigest = typeof inputDigest === "string" && inputDigest.startsWith("blake3:");
    const reason = hasInputDigest ? null : (entry.cacheBypassReason ?? "missingTargetSourceDigest");
    if (!isDiskBacked && !hasInputDigest) {
      throw new Error(
        `[omena-css] ${fileId} is missing the build-snapshot target-source digest (${reason}).`,
      );
    }
    entry.sourceProvenance = {
      schemaVersion: "0",
      product: "omena-vite.virtual-source-provenance",
      classification,
      diskDigest: isDiskBacked ? (hasInputDigest ? inputDigest : null) : diskDigest,
      inputDigest: hasInputDigest ? inputDigest : null,
      reason,
      upstreamMapPresent: upstreamMap != null,
      upstreamMapSources: upstreamMap?.sources ?? [],
    };
  }
  return output;
}

function usableUpstreamSourceMap(pluginContext, fileId, diskSource) {
  if (diskSource == null || typeof pluginContext.getCombinedSourcemap !== "function") return null;
  let candidate;
  try {
    candidate = pluginContext.getCombinedSourcemap();
  } catch {
    return null;
  }
  const map = normalizeSourceMap(candidate);
  if (!map) return null;

  const normalizedFileId = normalizeFilePath(fileId);
  const matchingSourceIndexes = map.sources.flatMap((source, index) =>
    normalizeFilePath(source) === normalizedFileId ? [index] : [],
  );
  if (
    matchingSourceIndexes.length !== 1 ||
    map.sourcesContent?.[matchingSourceIndexes[0]] !== diskSource
  ) {
    return null;
  }

  let hasMappedSegment = false;
  let hasNonIdentitySegment = false;
  const consumer = new SourceMapConsumer(map);
  try {
    consumer.eachMapping((mapping) => {
      if (mapping.originalLine == null || mapping.originalColumn == null) return;
      if (mapping.source == null || normalizeFilePath(mapping.source) !== normalizedFileId) return;
      hasMappedSegment = true;
      if (
        mapping.generatedLine !== mapping.originalLine ||
        mapping.generatedColumn !== mapping.originalColumn
      ) {
        hasNonIdentitySegment = true;
      }
    });
  } finally {
    consumer.destroy?.();
  }
  if (!hasMappedSegment || !hasNonIdentitySegment) return null;
  return {
    ...map,
    file: map.file ?? fileId,
  };
}

function normalizeSourceMap(candidate) {
  if (
    candidate?.version !== 3 ||
    !Array.isArray(candidate.sources) ||
    !Array.isArray(candidate.names) ||
    typeof candidate.mappings !== "string" ||
    candidate.mappings.length === 0
  ) {
    return null;
  }
  return {
    version: 3,
    ...(typeof candidate.file === "string" ? { file: candidate.file } : {}),
    sources: candidate.sources.map(String),
    ...(Array.isArray(candidate.sourcesContent)
      ? {
          sourcesContent: candidate.sourcesContent.map((source) =>
            source == null ? null : String(source),
          ),
        }
      : {}),
    names: candidate.names.map(String),
    mappings: candidate.mappings,
  };
}

function summarizeViteCache(cache) {
  return summarizeCache(cache).map((entry) => ({
    ...entry,
    sourceProvenance: cache.get(entry.filePath)?.sourceProvenance ?? null,
  }));
}

function registerBuildDependencies(pluginContext, state, targetPath) {
  const entry = state.cache.get(targetPath);
  for (const dependencyPath of entry?.dependencyPaths ?? []) {
    if (dependencyPath !== targetPath) pluginContext.addWatchFile?.(dependencyPath);
  }
}

function dependentBuildTargets(cache, changedPath) {
  return [...cache.entries()]
    .filter(([, entry]) => entry.dependencyPaths?.includes(changedPath))
    .map(([targetPath]) => targetPath)
    .toSorted((left, right) => (left < right ? -1 : left > right ? 1 : 0));
}

function shouldUseDevRuntime(options, state) {
  return options.devRuntime !== false && state.command === "serve";
}

function resolveStyleImport(id, importer, options, state) {
  const cleanId = cleanViteId(id);
  if (!cleanId || cleanId.startsWith("\0")) return null;
  if (path.isAbsolute(cleanId)) return normalizeFilePath(cleanId);
  if (cleanId.startsWith("/")) {
    return normalizeFilePath(path.resolve(options.cwd ?? state.root, cleanId.slice(1)));
  }
  if (!importer || importer.startsWith("\0")) return null;
  return normalizeFilePath(path.resolve(path.dirname(cleanViteId(importer)), cleanId));
}

function toDevRuntimeId(filePath) {
  return `${DEV_RUNTIME_ID_PREFIX}${Buffer.from(normalizeFilePath(filePath), "utf8").toString("base64url")}`;
}

function isDevRuntimeId(id) {
  return id.startsWith(DEV_RUNTIME_ID_PREFIX);
}

function fromDevRuntimeId(id) {
  return Buffer.from(id.slice(DEV_RUNTIME_ID_PREFIX.length), "base64url").toString("utf8");
}

function renderDevCssModule(filePath, output) {
  const { code: css, classExports, valueExports, namedExports = [] } = output;
  if (!classExports || !valueExports) {
    throw new Error("[omena-css] dev runtime requires typed CSS Module export namespaces.");
  }
  const styleId = `omena-css:${filePath}`;
  const eventName = devRuntimeEventName(filePath);
  const namedBindings = unambiguousNamedExports(namedExports);
  return [
    DEV_RUNTIME_MARKER,
    `const css = ${JSON.stringify(css)};`,
    `const styleId = ${JSON.stringify(styleId)};`,
    `const classExports = ${JSON.stringify(classExports)};`,
    `const valueExports = ${JSON.stringify(valueExports)};`,
    ...namedBindings.map(
      ({ exportedName, kind }) =>
        `let ${exportedName} = ${kind === "class" ? "classExports" : "valueExports"}[${JSON.stringify(exportedName)}];`,
    ),
    `const eventName = ${JSON.stringify(eventName)};`,
    `function findOmenaStyle() {`,
    `  return Array.from(document.querySelectorAll("style[data-omena-vite-style]")).find((style) => style.getAttribute("data-omena-vite-style") === styleId) ?? null;`,
    `}`,
    `function applyOmenaCss(nextCss) {`,
    `  let style = findOmenaStyle();`,
    `  if (!style) {`,
    `    style = document.createElement("style");`,
    `    style.setAttribute("data-omena-vite-style", styleId);`,
    `    document.head.appendChild(style);`,
    `  }`,
    `  style.textContent = nextCss;`,
    `}`,
    `if (typeof document !== "undefined") applyOmenaCss(css);`,
    `if (import.meta.hot) {`,
    `  import.meta.hot.accept();`,
    `  import.meta.hot.on(eventName, (payload) => {`,
    `    applyOmenaCss(payload.css);`,
    `    for (const key of Object.keys(classExports)) delete classExports[key];`,
    `    Object.assign(classExports, payload.classExports);`,
    `    for (const key of Object.keys(valueExports)) delete valueExports[key];`,
    `    Object.assign(valueExports, payload.valueExports);`,
    ...namedBindings.map(
      ({ exportedName, kind }) =>
        `    ${exportedName} = ${kind === "class" ? "classExports" : "valueExports"}[${JSON.stringify(exportedName)}];`,
    ),
    `  });`,
    `  import.meta.hot.prune(() => {`,
    `    findOmenaStyle()?.remove();`,
    `  });`,
    `}`,
    `export default classExports;`,
    `export { classExports, valueExports${
      namedBindings.length > 0
        ? `, ${namedBindings.map(({ exportedName }) => exportedName).join(", ")}`
        : ""
    } };`,
    ``,
  ].join("\n");
}

function classifyCssModuleExportDelta(previousExports, nextExports) {
  if (!previousExports || !nextExports) return "shapeChanged";
  const previousKeys = Object.keys(previousExports).toSorted();
  const nextKeys = Object.keys(nextExports).toSorted();
  if (
    previousKeys.length !== nextKeys.length ||
    previousKeys.some((key, index) => key !== nextKeys[index])
  ) {
    return "shapeChanged";
  }
  return previousKeys.every((key) => previousExports[key] === nextExports[key])
    ? "styleOnly"
    : "valueChanged";
}

function combineCssModuleExportDelta(left, right) {
  if (left === "shapeChanged" || right === "shapeChanged") return "shapeChanged";
  if (left === "valueChanged" || right === "valueChanged") return "valueChanged";
  return "styleOnly";
}

function unambiguousNamedExports(namedExports) {
  const byName = new Map();
  for (const entry of namedExports) {
    if (byName.has(entry.exportedName)) {
      byName.set(entry.exportedName, null);
    } else {
      byName.set(entry.exportedName, entry);
    }
  }
  return [...byName.values()]
    .filter(Boolean)
    .filter(
      ({ exportedName }) => exportedName !== "classExports" && exportedName !== "valueExports",
    )
    .toSorted((left, right) =>
      left.exportedName < right.exportedName ? -1 : left.exportedName > right.exportedName ? 1 : 0,
    );
}

function reportBundlerHostDiagnostics(context, output, pluginName) {
  for (const diagnostic of output.moduleInterface?.diagnostics ?? []) {
    context.warn?.(`[${pluginName}] ${diagnostic.code}: ${diagnostic.message}`);
  }
}

function collectAffectedRuntimeModules(runtimeModule) {
  if (!runtimeModule) return [];
  const affected = [];
  const pending = [runtimeModule];
  const seen = new Set();
  while (pending.length > 0) {
    const current = pending.shift();
    if (!current || seen.has(current)) continue;
    seen.add(current);
    affected.push(current);
    for (const importer of current.importers ?? []) {
      if (!seen.has(importer)) pending.push(importer);
    }
  }
  return affected;
}

function devRuntimeUpdatePayload(filePath, output, decision) {
  return {
    filePath,
    decision,
    css: output.code,
    classExports: output.classExports,
    valueExports: output.valueExports,
    sourceMapSources: output.map?.sources ?? [],
  };
}

function devRuntimeEventName(filePath) {
  const hash = crypto.createHash("sha256").update(normalizeFilePath(filePath)).digest("hex");
  return `omena-css:update:${hash.slice(0, 16)}`;
}

function cleanViteId(id) {
  return id.split("?", 1)[0];
}

module.exports = {
  MINIFY_PASS_IDS,
  TREE_SHAKE_PASS_IDS,
  VIRTUAL_MODULE_ID,
  classifyCssModuleExportDelta,
  omenaCss,
  default: omenaCss,
};
