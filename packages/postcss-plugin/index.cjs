const path = require("node:path");
const postcss = require("postcss");
const { SourceMapConsumer, SourceMapGenerator } = require("source-map-js");
const {
  DEFAULT_INCLUDE,
  MINIFY_PASS_IDS,
  TREE_SHAKE_PASS_IDS,
  createOmenaBuildState,
  matchesInclude,
  normalizeFilePath,
  rebuildAndCache,
  resolveEffectiveOptions,
} = require("@omena/css-build-adapter");

function omenaPostcss(options = {}) {
  const pluginName = options.name ?? "omena-css";
  const state = createOmenaBuildState(options);

  return {
    postcssPlugin: pluginName,
    async Once(root, { result }) {
      const filePath = resolvePostcssInputPath(root, result, options);
      if (!filePath) return;

      const effectiveOptions = {
        ...(await resolveEffectiveOptions(options, state)),
        moduleInterface: false,
      };
      const include = effectiveOptions.include ?? DEFAULT_INCLUDE;
      if (!matchesInclude(filePath, include)) return;

      const source = root.source?.input?.css ?? root.toString();
      const output = await rebuildAndCache(filePath, source, effectiveOptions, state);
      const previousMap = readPreviousSourceMap(root);
      const composition = composeWithPreviousSourceMap(output.map, filePath, previousMap);
      replacePostcssRoot(root, output.code, filePath, composition.sourceMap);
      result.messages.push({
        type: "omena-css",
        plugin: pluginName,
        file: filePath,
        summary: output.summary,
        sourceMap: composition.sourceMap,
        sourceMapSources: composition.sourceMap?.sources ?? [],
        upstreamMapApplied: composition.upstreamMapApplied,
      });
    },
  };
}

omenaPostcss.postcss = true;

function readPreviousSourceMap(root) {
  const previousMap = root.source?.input?.map;
  if (!previousMap) return null;
  if (typeof previousMap.text === "string") return previousMap.text;
  if (typeof previousMap.consumer === "function") {
    return SourceMapGenerator.fromSourceMap(previousMap.consumer()).toString();
  }
  return null;
}

function composeWithPreviousSourceMap(sourceMap, filePath, previousMap) {
  if (!sourceMap || !previousMap) {
    return { sourceMap, upstreamMapApplied: false };
  }

  const sourceFile = selectDownstreamSourceFile(sourceMap, filePath);
  if (!sourceFile) {
    return { sourceMap, upstreamMapApplied: false };
  }

  try {
    const downstream = new SourceMapConsumer(sourceMap);
    const upstream = new SourceMapConsumer(previousMap);
    const generator = SourceMapGenerator.fromSourceMap(downstream);
    generator.applySourceMap(upstream, sourceFile);
    const composed = generator.toJSON();
    return {
      sourceMap: {
        ...sourceMap,
        ...composed,
        x_omenaPostcssUpstreamMapApplied: true,
      },
      upstreamMapApplied: true,
    };
  } catch {
    return { sourceMap, upstreamMapApplied: false };
  }
}

function selectDownstreamSourceFile(sourceMap, filePath) {
  const target = normalizeSourcePathForCompare(filePath);
  return (
    sourceMap.sources?.find((source) => normalizeSourcePathForCompare(source) === target) ??
    sourceMap.sources?.[0] ??
    null
  );
}

function normalizeSourcePathForCompare(sourcePath) {
  if (!sourcePath) return "";
  if (!path.isAbsolute(sourcePath)) return path.normalize(sourcePath);
  try {
    return require("node:fs").realpathSync.native(sourcePath);
  } catch {
    return path.resolve(sourcePath);
  }
}

function resolvePostcssInputPath(root, result, options) {
  const rawPath = options.from ?? root.source?.input?.file ?? result.opts.from;
  if (!rawPath || rawPath === "<input css>") return null;
  return normalizeFilePath(path.resolve(options.cwd ?? process.cwd(), rawPath));
}

function replacePostcssRoot(root, css, filePath, sourceMap) {
  const parseOptions = sourceMap
    ? {
        from: filePath,
        map: {
          prev: sourceMap,
          inline: false,
          annotation: false,
        },
      }
    : { from: filePath };
  const nextRoot = postcss.parse(css, parseOptions);
  root.removeAll();
  root.raws = { ...nextRoot.raws };
  root.source = nextRoot.source;
  root.append(nextRoot.nodes);
}

module.exports = {
  MINIFY_PASS_IDS,
  TREE_SHAKE_PASS_IDS,
  omenaPostcss,
  default: omenaPostcss,
};
