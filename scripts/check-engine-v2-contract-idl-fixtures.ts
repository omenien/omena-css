import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import Ajv2020 from "ajv/dist/2020";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workDir = fs.mkdtempSync(path.join(os.tmpdir(), "omena-engine-v2-idl-fixtures-"));
const schemaDir = path.join(workDir, "schema");
const fixtureDir = path.join(repoRoot, "test/_fixtures/contract-parity-v2");

execFileSync(
  "pnpm",
  [
    "exec",
    "tsp",
    "compile",
    "contracts/engine-v2",
    "--emit",
    "@typespec/json-schema",
    "--option",
    `@typespec/json-schema.emitter-output-dir=${schemaDir}`,
    "--option",
    "@typespec/json-schema.file-type=json",
    "--option",
    "@typespec/json-schema.emitAllModels=true",
    "--option",
    "@typespec/json-schema.polymorphic-models-strategy=oneOf",
    "--option",
    "@typespec/json-schema.seal-object-schemas=true",
  ],
  { cwd: repoRoot, stdio: "inherit" },
);

const ajv = new Ajv2020({ allErrors: true, strict: false });
for (const fileName of fs.readdirSync(schemaDir).filter((name) => name.endsWith(".json"))) {
  const schema = JSON.parse(fs.readFileSync(path.join(schemaDir, fileName), "utf8")) as unknown;
  ajv.addSchema(schema, fileName);
}

const validateInput = requiredValidator("EngineInputV2.json");
const validateOutput = requiredValidator("EngineOutputV2.json");
const validateCodeActionPlan = requiredValidator("OmenaQueryCodeActionPlanV0.json");

const fixtureFiles = fs
  .readdirSync(fixtureDir)
  .filter((fileName) => fileName.endsWith(".json"))
  .toSorted();

const queryKinds = new Set<string>();
let validatedFixtureCount = 0;
for (const fileName of fixtureFiles) {
  const fixture = JSON.parse(fs.readFileSync(path.join(fixtureDir, fileName), "utf8")) as {
    readonly input: unknown;
    readonly output: {
      readonly queryResults?: readonly { readonly kind?: string }[];
    };
  };

  validateOrThrow(validateInput, fixture.input, `${fileName}:input`);
  validateOrThrow(validateOutput, fixture.output, `${fileName}:output`);
  for (const result of fixture.output.queryResults ?? []) {
    if (result.kind) {
      queryKinds.add(result.kind);
    }
  }
  validatedFixtureCount += 1;
}

for (const expected of ["expression-semantics", "source-expression-resolution", "selector-usage"]) {
  assert.ok(queryKinds.has(expected), `contract parity fixtures must cover ${expected}`);
}

validateOrThrow(
  validateCodeActionPlan,
  {
    schemaVersion: "0",
    product: "omena-query.code-actions",
    fileUri: "file:///repo/src/App.module.scss",
    fileKind: "style",
    actionCount: 1,
    actions: [
      {
        title: "Extract CSS custom property",
        kind: "refactor.extract",
        edits: [
          {
            uri: "file:///repo/src/App.module.scss",
            range: {
              start: { line: 0, character: 0 },
              end: { line: 0, character: 5 },
            },
            newText: "var(--token)",
          },
        ],
        source: "omenaQueryStyleExtractCodeActions",
      },
    ],
    readySurfaces: ["productFacingCodeActions"],
  },
  "code-action-query-json-sample",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "engine-v2.contract-idl-fixtures",
      fixtureCount: validatedFixtureCount,
      queryKinds: [...queryKinds].toSorted(),
      schemaDir,
    },
    null,
    2,
  )}\n`,
);

function requiredValidator(schemaName: string) {
  const validate = ajv.getSchema(schemaName);
  assert.ok(validate, `schema not registered: ${schemaName}`);
  return validate;
}

function validateOrThrow(
  validate: ReturnType<typeof requiredValidator>,
  value: unknown,
  label: string,
): void {
  if (validate(value)) {
    return;
  }
  throw new Error(`${label} failed schema validation: ${ajv.errorsText(validate.errors)}`);
}
