const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const {
  DEFAULT_INCLUDE,
  MINIFY_PASS_IDS,
  TREE_SHAKE_PASS_IDS,
  createOmenaBuildState,
  matchesInclude,
  normalizeFilePath,
  rebuildAndCache,
  resolveEffectiveOptions,
  summarizeCache,
} = require("@omena/css-build-adapter");

const VIRTUAL_MODULE_ID = "virtual:omena-css/build-summary";
const RESOLVED_VIRTUAL_MODULE_ID = `\0${VIRTUAL_MODULE_ID}`;
const DEV_RUNTIME_ID_PREFIX = "\0omena-vite-style:";
const DEV_RUNTIME_MARKER = "/* @omena/vite-plugin dev runtime */";

function omenaCss(options = {}) {
  const pluginName = options.name ?? "omena-css";
  const state = createOmenaBuildState(options);

  return {
    name: pluginName,
    enforce: options.enforce ?? "pre",
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
        return `export default ${JSON.stringify(summarizeCache(state.cache))};\n`;
      }
      if (!isDevRuntimeId(id)) return null;

      const effectiveOptions = await resolveEffectiveOptions(options, state);
      if (!shouldUseDevRuntime(effectiveOptions, state)) return null;

      const fileId = fromDevRuntimeId(id);
      const source = await fs.promises.readFile(fileId, "utf8");
      const output = await rebuildAndCache(fileId, source, effectiveOptions, state);
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
      if (!fs.existsSync(fileId)) return null;
      if (effectiveOptions.requireDiskSource !== false) {
        const diskSource = fs.readFileSync(fileId, "utf8");
        if (diskSource !== code) {
          this.warn?.(
            `[${pluginName}] skipped ${fileId}: transform input differs from disk source; set requireDiskSource=false to allow disk-backed transforms.`,
          );
          return null;
        }
      }

      const output = await rebuildAndCache(fileId, code, effectiveOptions, state);
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

      const invalidatedModules = [];
      for (const targetPath of targetPaths) {
        if (!fs.existsSync(targetPath)) continue;
        const source = await fs.promises.readFile(targetPath, "utf8");
        const previousOutput = state.cache.get(targetPath)?.output;
        const output = await rebuildAndCache(targetPath, source, effectiveOptions, state);
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
  const affected = [runtimeModule];
  for (const importer of runtimeModule.importers ?? []) {
    if (!affected.includes(importer)) affected.push(importer);
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
