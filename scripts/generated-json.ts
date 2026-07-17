import { strict as assert } from "node:assert";

import { format } from "oxfmt";

export async function formatGeneratedJson(fileName: string, value: unknown): Promise<string> {
  const result = await format(fileName, JSON.stringify(value, null, 2), {
    insertFinalNewline: true,
    printWidth: 100,
    tabWidth: 2,
    useTabs: false,
  });
  assert.equal(result.errors.length, 0, `failed to format generated JSON for ${fileName}`);
  return result.code;
}
