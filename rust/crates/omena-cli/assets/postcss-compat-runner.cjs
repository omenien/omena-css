"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { createRequire } = require("node:module");

function packageVersion(requireFromProject, packageName) {
  try {
    return requireFromProject(`${packageName}/package.json`).version;
  } catch {
    let current = path.dirname(requireFromProject.resolve(packageName));
    while (current !== path.dirname(current)) {
      const manifestPath = path.join(current, "package.json");
      if (fs.existsSync(manifestPath)) {
        const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
        if (manifest.name === packageName) return manifest.version;
      }
      current = path.dirname(current);
    }
    throw new Error(`cannot resolve package metadata for ${packageName}`);
  }
}

async function main() {
  const request = JSON.parse(fs.readFileSync(0, "utf8"));
  const requireFromProject = createRequire(path.join(request.projectRoot, "package.json"));
  const postcss = requireFromProject("postcss");
  const pluginFactory = requireFromProject(request.packageName);
  const version = packageVersion(requireFromProject, request.packageName);
  if (version !== request.expectedVersion) {
    throw new Error(
      `plugin version mismatch for ${request.packageName}: expected ${request.expectedVersion}, found ${version}`,
    );
  }

  const config = JSON.parse(request.configJson);
  const result = await postcss([pluginFactory(config)]).process(request.sourceCss, {
    from: request.sourcePath,
    map: false,
  });
  process.stdout.write(
    JSON.stringify({
      schemaVersion: "0",
      outputCss: result.css,
      pluginVersion: version,
      warningCount: result.warnings().length,
    }),
  );
}

main().catch((error) => {
  process.stderr.write(`${error && error.stack ? error.stack : String(error)}\n`);
  process.exitCode = 1;
});
