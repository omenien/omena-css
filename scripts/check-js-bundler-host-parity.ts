import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

type StyleSource = { readonly stylePath: string; readonly styleSource: string };
type ParityCase = {
  readonly id: string;
  readonly targetPath: string;
  readonly sources: readonly StyleSource[];
  readonly emittedCss: string;
  readonly legacyRegexMap: Readonly<Record<string, string>>;
  readonly expectedClassExports: Readonly<Record<string, string>>;
  readonly expectedValueExports: Readonly<Record<string, string>>;
  readonly expectedDiagnosticCodes?: readonly string[];
};
type InterfaceModule = {
  readonly stylePath: string;
  readonly classExports: readonly {
    readonly name: string;
    readonly namedExport?: string;
    readonly emittedClasses: readonly string[];
  }[];
  readonly icssExports: readonly {
    readonly name: string;
    readonly namedExport?: string;
    readonly value: string;
  }[];
};

const repoRoot = path.resolve(import.meta.dirname, "..");
const rustRoot = path.join(repoRoot, "rust");
const fixturePath = path.join(repoRoot, "scripts/fixtures/css-module-host-parity/cases.json");
const require = createRequire(import.meta.url);
const { createOmenaBuildState, rebuildAndCache } =
  require("../packages/css-build-adapter/index.cjs") as {
    createOmenaBuildState(options: Record<string, unknown>): unknown;
    rebuildAndCache(
      filePath: string,
      source: string,
      options: Record<string, unknown>,
      state: unknown,
    ): Promise<{
      readonly classExports: Readonly<Record<string, string>>;
      readonly valueExports: Readonly<Record<string, string>>;
      readonly namedExports: readonly NamedExport[];
      readonly moduleInterface: {
        readonly diagnostics: readonly { readonly code: string }[];
      };
      readonly typescriptDeclaration: string;
    }>;
  };
const { omenaCss } = require("../packages/vite-plugin/index.cjs") as {
  omenaCss(options: Record<string, unknown>): {
    configResolved(config: { readonly root: string; readonly command: string }): void;
    resolveId(id: string, importer?: string): Promise<string | null>;
    load: {
      call(
        context: Record<string, unknown>,
        id: string,
      ): Promise<string | { readonly code: string; readonly map: unknown } | null>;
    };
  };
};

const injectDroppedComposesEdge = process.argv.includes("--inject-dropped-composes-edge");
const injectRenamedDevClass = process.argv.includes("--inject-renamed-dev-class");
const injectMergedExportOracle = process.argv.includes("--inject-merged-export-oracle");
const injectSuppressedCollisionDiagnostic = process.argv.includes(
  "--inject-suppressed-collision-diagnostic",
);
const injectValueBindingClassFamily = process.argv.includes("--inject-value-binding-class-family");
const injectMergedDefaultExport = process.argv.includes("--inject-merged-default-export");
const fixtures = (JSON.parse(fs.readFileSync(fixturePath, "utf8")) as readonly ParityCase[]).map(
  applyFixtureFault,
);
const cliPath = path.join(rustRoot, "target/debug/omena");
const boundaryRunnerPath = path.join(rustRoot, "target/debug/engine-shadow-runner");

main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});

async function main() {
  assert.ok(fixtures.length >= 6, "bundler host parity requires at least six fixtures");
  assert.deepEqual(
    new Set(fixtures.map((fixture) => path.extname(fixture.targetPath))),
    new Set([".css", ".scss", ".less"]),
  );
  run("cargo", [
    "build",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-cli",
    "-p",
    "engine-shadow-runner",
  ]);

  const outcomes = [];
  for (const fixture of fixtures) {
    outcomes.push(await verifyFixture(fixture));
  }

  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "js-bundler-host.parity",
        fixtureCount: outcomes.length,
        outcomes,
      },
      null,
      2,
    )}\n`,
  );
}

async function verifyFixture(fixture: ParityCase) {
  const root = fs.realpathSync.native(
    fs.mkdtempSync(path.join(os.tmpdir(), `omena-bundler-host-${fixture.id}-`)),
  );
  try {
    const materialized = new Map<string, string>();
    for (const source of fixture.sources) {
      const relativePath = source.stylePath.replace(/^\/workspace\//u, "");
      const filePath = path.join(root, relativePath);
      fs.mkdirSync(path.dirname(filePath), { recursive: true });
      fs.writeFileSync(filePath, source.styleSource, "utf8");
      materialized.set(source.stylePath, fs.realpathSync.native(filePath));
    }
    const targetPath = materialized.get(fixture.targetPath);
    assert.ok(targetPath, `missing target source for ${fixture.id}`);
    const interfacePath = path.join(root, "generated/module-interface.json");
    const declarationRoot = path.join(root, "generated/types");

    run(cliPath, [
      "modules",
      "emit",
      root,
      "--interface-file",
      interfacePath,
      "--declaration-dir",
      declarationRoot,
      "--json",
    ]);
    const bundle = JSON.parse(fs.readFileSync(interfacePath, "utf8")) as {
      readonly modules: readonly InterfaceModule[];
    };
    const module = bundle.modules.find((candidate) => candidate.stylePath === targetPath);
    assert.ok(module, `CLI module-interface artifact omitted ${fixture.id}`);
    const cliClassExports = classExportsFromModule(module);
    const cliValueExports = valueExportsFromModule(module);
    if (injectMergedExportOracle && fixture.id === "css-class-value-collision") {
      Object.assign(cliClassExports, cliValueExports);
    }

    const targetSource = fixture.sources.find((source) => source.stylePath === fixture.targetPath);
    assert.ok(targetSource);
    const otherPaths = fixture.sources
      .filter((source) => source.stylePath !== fixture.targetPath)
      .map((source) => materialized.get(source.stylePath))
      .filter((sourcePath): sourcePath is string => Boolean(sourcePath));
    const engine = createBoundaryEngine(fixture);
    const state = createOmenaBuildState({ cwd: root });
    const adapterOutput = await rebuildAndCache(
      targetPath,
      targetSource.styleSource,
      { cwd: root, configFile: false, engine, sources: otherPaths },
      state,
    );

    assert.deepEqual(canonicalRecord(adapterOutput.classExports), canonicalRecord(cliClassExports));
    assert.deepEqual(
      canonicalRecord(adapterOutput.classExports),
      canonicalRecord(fixture.expectedClassExports),
    );
    assert.deepEqual(canonicalRecord(adapterOutput.valueExports), canonicalRecord(cliValueExports));
    assert.deepEqual(
      canonicalRecord(adapterOutput.valueExports),
      canonicalRecord(fixture.expectedValueExports),
    );
    assert.notDeepEqual(
      canonicalRecord(fixture.legacyRegexMap),
      canonicalRecord(fixture.expectedClassExports),
    );
    const expectedDiagnosticCodes = [...(fixture.expectedDiagnosticCodes ?? [])].toSorted();
    assert.deepEqual(
      adapterOutput.moduleInterface.diagnostics.map(({ code }) => code).toSorted(),
      expectedDiagnosticCodes,
    );
    assertNamedExportFamilies(adapterOutput.namedExports, module);
    assert.deepEqual(engine.classNameRewriteCalls, [
      Object.entries(adapterOutput.classExports).map(([originalName, rewrittenName]) => ({
        originalName,
        rewrittenName: rewrittenName.split(/\s+/u, 1)[0],
      })),
    ]);
    const declarationPath = path.join(
      declarationRoot,
      path.relative(root, targetPath).concat(".d.ts"),
    );
    assert.equal(
      adapterOutput.typescriptDeclaration,
      fs.readFileSync(declarationPath, "utf8"),
      `typed export artifact drifted for ${fixture.id}`,
    );
    const namedExportCount = typecheckConsumer(root, targetPath, declarationPath, module);
    const devRuntime = await verifyDevRuntime(
      root,
      targetPath,
      otherPaths,
      fixture,
      adapterOutput.namedExports,
    );

    return {
      id: fixture.id,
      classExports: canonicalRecord(adapterOutput.classExports),
      valueExports: canonicalRecord(adapterOutput.valueExports),
      diagnosticCodes: expectedDiagnosticCodes,
      classNameRewrites: engine.classNameRewriteCalls[0],
      namedExportCount,
      devRuntime,
      parity: true,
      typescriptConsumer: true,
    };
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

async function verifyDevRuntime(
  root: string,
  targetPath: string,
  otherPaths: readonly string[],
  fixture: ParityCase,
  namedExports: readonly NamedExport[],
) {
  const engine = createBoundaryEngine(fixture);
  const plugin = omenaCss({ cwd: root, configFile: false, engine, sources: otherPaths });
  plugin.configResolved({ root, command: "serve" });
  const runtimeId = await plugin.resolveId(targetPath);
  assert.ok(runtimeId, `Vite dev runtime did not claim ${fixture.id}`);
  const loaded = await plugin.load.call({}, runtimeId);
  assert.ok(loaded && typeof loaded !== "string", `Vite dev runtime did not render ${fixture.id}`);
  let runtimeCode = loaded.code;
  if (injectValueBindingClassFamily && fixture.id === "css-value-only") {
    const mutated = runtimeCode.replaceAll(" = valueExports[", " = classExports[");
    assert.notEqual(mutated, runtimeCode, "value-family mutation must alter a live binding");
    runtimeCode = mutated;
  }
  if (injectMergedDefaultExport && fixture.id === "css-value-only") {
    const mutated = runtimeCode.replace(
      "export default classExports;",
      "export default { ...classExports, ...valueExports };",
    );
    assert.notEqual(mutated, runtimeCode, "merged-default mutation must alter the runtime export");
    runtimeCode = mutated;
  }
  const runtimePath = path.join(root, `generated/${fixture.id}-dev-runtime.mjs`);
  fs.mkdirSync(path.dirname(runtimePath), { recursive: true });
  fs.writeFileSync(runtimePath, runtimeCode, "utf8");
  const runtime = (await import(
    `${pathToFileURL(runtimePath).href}?fixture=${encodeURIComponent(fixture.id)}`
  )) as Record<string, unknown>;
  assert.deepEqual(
    canonicalRecord(runtime.classExports as Readonly<Record<string, string>>),
    canonicalRecord(fixture.expectedClassExports),
    `rendered class family drifted for ${fixture.id}`,
  );
  assert.deepEqual(
    canonicalRecord(runtime.valueExports as Readonly<Record<string, string>>),
    canonicalRecord(fixture.expectedValueExports),
    `rendered value family drifted for ${fixture.id}`,
  );
  assert.strictEqual(
    runtime.default,
    runtime.classExports,
    `default export must be the class family object for ${fixture.id}`,
  );
  for (const valueName of Object.keys(fixture.expectedValueExports)) {
    if (!Object.hasOwn(fixture.expectedClassExports, valueName)) {
      assert.equal(
        Object.hasOwn(runtime.default as object, valueName),
        false,
        `default export leaked ICSS value '${valueName}' for ${fixture.id}`,
      );
    }
  }
  const unambiguous = unambiguousNamedExportEntries(namedExports);
  const runtimeBindings: Record<
    string,
    { readonly kind: NamedExport["kind"]; readonly value: string }
  > = {};
  for (const entry of unambiguous) {
    assert.equal(
      runtime[entry.exportedName],
      entry.value,
      `rendered ${entry.kind} binding '${entry.exportedName}' read the wrong family for ${fixture.id}`,
    );
    runtimeBindings[entry.exportedName] = { kind: entry.kind, value: entry.value };
  }
  return {
    defaultExportClassOnly: true,
    namedBindings: runtimeBindings,
  };
}

function unambiguousNamedExportEntries(namedExports: readonly NamedExport[]) {
  const counts = new Map<string, number>();
  for (const entry of namedExports) {
    counts.set(entry.exportedName, (counts.get(entry.exportedName) ?? 0) + 1);
  }
  return namedExports.filter(
    (entry) =>
      counts.get(entry.exportedName) === 1 &&
      entry.exportedName !== "classExports" &&
      entry.exportedName !== "valueExports",
  );
}

function createBoundaryEngine(fixture: ParityCase) {
  const classNameRewriteCalls: Array<
    Array<{ readonly originalName: string; readonly rewrittenName: string }>
  > = [];
  return {
    classNameRewriteCalls,
    buildSnapshotIdentity(input: {
      readonly targetPath: string;
      readonly styleSources: readonly {
        readonly stylePath: string;
        readonly styleSource: string;
      }[];
    }) {
      const targetSource = input.styleSources.find(
        ({ stylePath }) => stylePath === input.targetPath,
      )?.styleSource;
      return {
        schemaVersion: "0",
        product: "omena-query.build-snapshot-digest",
        contentHashAlgorithm: "blake3",
        digest: `blake3:bundler-parity-${JSON.stringify(input)}`,
        targetSourceDigest: `blake3:bundler-parity-source-${JSON.stringify(targetSource)}`,
      };
    },
    summarizeTransformBundleFromSourceJson: () =>
      JSON.stringify({
        plannedPassIds: ["composes-resolution", "css-modules-class-hashing"],
      }),
    buildStyleSourcesWithContextJson: (
      _targetPath: string,
      _sourcesJson: string,
      _passIds: string[],
      contextJson: string,
    ) => {
      const context = JSON.parse(contextJson) as {
        readonly classNameRewrites?: Array<{
          readonly originalName: string;
          readonly rewrittenName: string;
        }>;
      };
      classNameRewriteCalls.push(context.classNameRewrites ?? []);
      return JSON.stringify({
        execution: {
          outputCss: fixture.emittedCss,
          executedPassIds: ["composes-resolution", "css-modules-class-hashing"],
        },
      });
    },
    bundlerHostCapabilitiesJson: () =>
      JSON.stringify({
        protocolVersion: "0",
        capabilities: [
          "semanticClassExports",
          "typedExportNamespaces",
          "namedExports",
          "composesEdges",
        ],
      }),
    resolveCssModuleForBundlerHostJson: (requestJson: string) => {
      const response = JSON.parse(
        capture(boundaryRunnerPath, ["bundler-host-resolve-module"], requestJson),
      ) as {
        classExports: Record<string, string>;
        valueExports: Record<string, string>;
        namedExports: NamedExport[];
        diagnostics: { code: string }[];
      };
      if (injectRenamedDevClass && fixture.id === "css-local-class") {
        response.classExports.root = `${response.classExports.root} renamed`;
        const named = response.namedExports.find(
          (entry) => entry.kind === "class" && entry.exportedName === "root",
        );
        if (named) named.value = response.classExports.root;
      }
      if (injectSuppressedCollisionDiagnostic && fixture.id === "css-class-value-collision") {
        response.diagnostics = response.diagnostics.filter(
          ({ code }) => code !== "exportNamespaceCollision",
        );
      }
      return JSON.stringify(response);
    },
  };
}

function applyFixtureFault(fixture: ParityCase): ParityCase {
  if (!injectDroppedComposesEdge || fixture.id !== "css-imported-composes") return fixture;
  const sources = fixture.sources.map((source) => ({
    ...source,
    styleSource: source.styleSource.replace(
      /\s*composes:\s*base\s+from\s+["']\.\/base\.module\.css["'];/u,
      "",
    ),
  }));
  assert.notDeepEqual(sources, fixture.sources, "composes fault must alter the fixture source");
  return { ...fixture, sources };
}

function typecheckConsumer(
  root: string,
  targetPath: string,
  declarationPath: string,
  module: InterfaceModule,
) {
  const namedExportCounts = new Map<string, number>();
  for (const entry of [...module.classExports, ...module.icssExports]) {
    if (!entry.namedExport) continue;
    namedExportCounts.set(entry.namedExport, (namedExportCounts.get(entry.namedExport) ?? 0) + 1);
  }
  const namedExports = [...namedExportCounts]
    .filter(([, count]) => count === 1)
    .map(([name]) => name)
    .toSorted();
  const importPath = `./${path
    .relative(root, declarationPath)
    .replaceAll(path.sep, "/")
    .replace(/\.d\.ts$/u, "")}`;
  const classDefaultKey = module.classExports[0]?.name;
  const valueDefaultKey = module.icssExports[0]?.name;
  assert.ok(classDefaultKey ?? valueDefaultKey, `fixture ${targetPath} must expose an export key`);
  const consumerPath = path.join(root, "consumer.ts");
  fs.writeFileSync(
    consumerPath,
    [
      `import styles, { classExports, valueExports${
        namedExports.length > 0 ? `, ${namedExports.join(", ")}` : ""
      } } from ${JSON.stringify(importPath)};`,
      ...(classDefaultKey
        ? [
            `const defaultClassValue: string = styles[${JSON.stringify(classDefaultKey)}];`,
            `const classValue: string = classExports[${JSON.stringify(classDefaultKey)}];`,
          ]
        : []),
      ...(valueDefaultKey
        ? [`const valueValue: string = valueExports[${JSON.stringify(valueDefaultKey)}];`]
        : []),
      ...namedExports.map((name) => `const ${name}Value: string = ${name};`),
      `void [${[
        ...(classDefaultKey ? ["defaultClassValue", "classValue"] : []),
        ...(valueDefaultKey ? ["valueValue"] : []),
        ...namedExports.map((name) => `${name}Value`),
      ].join(", ")}];`,
      "",
    ].join("\n"),
    "utf8",
  );
  run("pnpm", [
    "exec",
    "tsc",
    "--ignoreConfig",
    "--noEmit",
    "--strict",
    "--moduleResolution",
    "Bundler",
    "--module",
    "ESNext",
    "--target",
    "ES2025",
    "--allowArbitraryExtensions",
    "--skipLibCheck",
    consumerPath,
  ]);
  return namedExports.length;
}

function classExportsFromModule(module: InterfaceModule) {
  return Object.fromEntries(
    module.classExports.map((entry) => [entry.name, entry.emittedClasses.join(" ")] as const),
  );
}

function valueExportsFromModule(module: InterfaceModule) {
  return Object.fromEntries(module.icssExports.map((entry) => [entry.name, entry.value] as const));
}

type NamedExport = {
  readonly exportedName: string;
  readonly kind: "class" | "value";
  value: string;
};

function assertNamedExportFamilies(actual: readonly NamedExport[], module: InterfaceModule) {
  const expected = [
    ...module.classExports
      .filter((entry) => entry.namedExport)
      .map((entry) => ({
        exportedName: entry.namedExport!,
        kind: "class" as const,
        value: entry.emittedClasses.join(" "),
      })),
    ...module.icssExports
      .filter((entry) => entry.namedExport)
      .map((entry) => ({
        exportedName: entry.namedExport!,
        kind: "value" as const,
        value: entry.value,
      })),
  ].toSorted((left, right) =>
    left.exportedName < right.exportedName
      ? -1
      : left.exportedName > right.exportedName
        ? 1
        : left.kind < right.kind
          ? -1
          : left.kind > right.kind
            ? 1
            : 0,
  );
  assert.deepEqual(actual, expected);
}

function canonicalRecord(value: Readonly<Record<string, string>>) {
  return Object.fromEntries(
    Object.entries(value).toSorted(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0)),
  );
}

function run(command: string, args: readonly string[]) {
  const result = spawnSync(command, args, { cwd: repoRoot, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr || result.stdout);
}

function capture(command: string, args: readonly string[], input: string) {
  const result = spawnSync(command, args, { cwd: repoRoot, encoding: "utf8", input });
  assert.equal(result.status, 0, result.stderr || result.stdout);
  return result.stdout;
}
