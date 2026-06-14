const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const {
  BUNDLE_PASS_IDS,
  DEFAULT_INCLUDE,
  MINIFY_PASS_IDS,
  TREE_SHAKE_PASS_IDS,
  createOmenaBuildState,
  extractCssModuleClassMap,
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
      return {
        code: renderDevCssModule(fileId, output.code),
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
      if (output.code === code) return null;
      return {
        code: output.code,
        map: output.map,
      };
    },
    async handleHotUpdate(ctx) {
      const effectiveOptions = await resolveEffectiveOptions(options, state);
      const include = effectiveOptions.include ?? DEFAULT_INCLUDE;
      if (!matchesInclude(ctx.file, include)) return;

      const fileId = normalizeFilePath(ctx.file);
      const source = await fs.promises.readFile(fileId, "utf8");
      const output = await rebuildAndCache(fileId, source, effectiveOptions, state);

      if (shouldUseDevRuntime(effectiveOptions, state)) {
        ctx.server?.ws?.send?.({
          type: "custom",
          event: devRuntimeEventName(fileId),
          data: devRuntimeUpdatePayload(fileId, output),
        });
        const runtimeModule = ctx.server?.moduleGraph?.getModuleById?.(toDevRuntimeId(fileId));
        if (runtimeModule) {
          ctx.server?.moduleGraph?.invalidateModule?.(runtimeModule);
        }
        return [];
      }

      const runtimeModule = ctx.server?.moduleGraph?.getModuleById?.(toDevRuntimeId(fileId));
      const modules = ctx.modules?.length
        ? ctx.modules
        : [runtimeModule, ctx.server?.moduleGraph?.getModuleById?.(fileId)].filter(Boolean);
      for (const mod of modules) {
        ctx.server?.moduleGraph?.invalidateModule?.(mod);
      }
      return modules;
    },
  };
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

function renderDevCssModule(filePath, css) {
  const styleId = `omena-css:${filePath}`;
  const classMap = extractCssModuleClassMap(css);
  const eventName = devRuntimeEventName(filePath);
  return [
    DEV_RUNTIME_MARKER,
    `const css = ${JSON.stringify(css)};`,
    `const styleId = ${JSON.stringify(styleId)};`,
    `const classMap = ${JSON.stringify(classMap)};`,
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
    `    for (const key of Object.keys(classMap)) delete classMap[key];`,
    `    Object.assign(classMap, payload.classMap);`,
    `  });`,
    `  import.meta.hot.prune(() => {`,
    `    findOmenaStyle()?.remove();`,
    `  });`,
    `}`,
    `export default classMap;`,
    ``,
  ].join("\n");
}

function devRuntimeUpdatePayload(filePath, output) {
  return {
    filePath,
    css: output.code,
    classMap: extractCssModuleClassMap(output.code),
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
  BUNDLE_PASS_IDS,
  MINIFY_PASS_IDS,
  TREE_SHAKE_PASS_IDS,
  VIRTUAL_MODULE_ID,
  omenaCss,
  default: omenaCss,
};
